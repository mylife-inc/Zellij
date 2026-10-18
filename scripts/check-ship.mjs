// Import -> one build -> publication, using only a disposable loopback registry.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { chromium } from '@playwright/test';
import { localRegistry, pullPublishedImage } from './fixtures/local-registry.mjs';

const zel = path.resolve(process.argv[2]);
const root = await mkdtemp(path.join(tmpdir(), 'zellij-ship-'));
const suffix = `${process.pid}-${Date.now()}`;
const containers = [], images = [];
const docker = args => execFileSync('docker', args, { encoding: 'utf8', timeout: 120000 }).trim();
const run = args => execFileSync(zel, args, { cwd: root, stdio: 'inherit', timeout: 600000 });
const source = {
  'mkdocs.yml': 'site_name: Imported manual\nnav: [index.md, next.md]\n',
  'docs/index.md': '# Imported manual\n\n[Next page](next.md#details)\n\n![Logo](logo.svg)\n',
  'docs/next.md': '# Second page\n\n## Details\n\n[Home](index.md)\n',
  'docs/logo.svg': '<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20"><circle cx="10" cy="10" r="9"/></svg>',
};
try {
  for (const [relative, body] of Object.entries(source)) {
    const file = path.join(root, 'source', relative);
    await mkdir(path.dirname(file), { recursive: true });
    await writeFile(file, body);
  }
  const registry = await localRegistry(docker, containers, suffix);
  for (const kind of ['static', 'next']) {
    const tag = `${registry}/acceptance/ship-${kind}:${suffix}`;
    images.push(tag);
    run(['ship', 'source', '--out', `manual ${kind}`, '--image', kind, '--tag', tag]);
    pullPublishedImage(docker, tag);
    const name = `zel-ship-${kind}-${suffix}`;
    containers.push(name);
    docker(['run', '-d', '--name', name, '-p', '127.0.0.1::3000', tag]);
    const info = JSON.parse(docker(['inspect', name]))[0];
    const origin = `http://127.0.0.1:${info.NetworkSettings.Ports['3000/tcp'][0].HostPort}`;
    let ready = false;
    for (let n = 0; n < 120; n++) {
      try { if ((await fetch(`${origin}/guide`, { signal: AbortSignal.timeout(1000) })).ok) { ready = true; break; } } catch { /* startup */ }
      await new Promise(resolve => setTimeout(resolve, 250));
    }
    assert(ready, `Server did not start: ${docker(['logs', name])}`);
    const browser = await chromium.launch();
    try {
      const page = await browser.newPage();
      const errors = [];
      page.on('pageerror', error => errors.push(error.message));
      page.on('response', res => { if (res.status() >= 400) errors.push(`${res.status()} ${res.url()}`); });
      await page.goto(`${origin}/guide`);
      await page.getByRole('heading', { name: 'Imported manual', exact: true }).waitFor();
      const logo = page.locator('article img[alt="Logo"]');
      assert(await logo.evaluate(img => img.complete && img.naturalWidth > 0));
      await page.locator('article').getByRole('link', { name: 'Next page', exact: true }).click();
      await page.waitForURL(`${origin}/guide/next#details`);
      await page.reload();
      await page.getByRole('heading', { name: 'Second page', exact: true }).waitFor();
      assert.deepEqual(errors, []);
    } finally { await browser.close(); }
    assert.equal(await (await fetch(`${origin}/_zellij/assets/imported/docs/logo.svg`)).text(), source['docs/logo.svg']);
    for (const url of ['/zel.yaml', '/site.yaml', '/docs/index.md']) assert.equal((await fetch(origin + url)).status, 404);
    for (const [relative, body] of Object.entries(source)) assert.equal(await readFile(path.join(root, 'source', relative), 'utf8'), body);
    run(['pub', `manual ${kind}`, '--dry-run']);
    console.log(`ship ${kind}: imported, published, pulled and verified navigation/assets; source unchanged.`);
  }
} finally {
  for (const name of containers) { try { docker(['rm', '-f', name]); } catch { /* original failure */ } }
  for (const tag of images) { try { docker(['image', 'rm', tag]); } catch { /* failed build */ } }
  await rm(root, { recursive: true, force: true });
}
