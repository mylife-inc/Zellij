// Import -> one build -> publication, using only a disposable loopback registry.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { chromium, expect } from '@playwright/test';
import { localRegistry, pullPublishedImage } from './fixtures/local-registry.mjs';

const zel = path.resolve(process.argv[2]);
const root = await mkdtemp(path.join(tmpdir(), 'zellij-ship-'));
const suffix = `${process.pid}-${Date.now()}`;
const containers = [], images = [];
const buildBase = `zel-ship-build-args:${suffix}`;
const buildLabel = "日本語 '$(touch /tmp/SHOULD_NOT_EXIST)`echo literal` = ";
const docker = args => execFileSync('docker', args, { encoding: 'utf8', timeout: 120000 }).trim();
const run = args => execFileSync(zel, args, { cwd: root, env: { ...process.env, ZELLIJ_IMAGE: buildBase }, stdio: 'inherit', timeout: 600000 });
const source = {
  'mkdocs.yml': 'site_name: Imported manual\nnav: [index.md, next.md]\n',
  'docs/index.md': '# Imported manual\n\n[Next page](next.md#details)\n\n![Logo](logo.svg)\n\n$x^2$\n\n```mermaid\ngraph TD; A-->B;\n```\n',
  'docs/next.md': '# Second page\n\n## Details\n\n[Home](index.md)\n',
  'docs/logo.svg': '<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20"><circle cx="10" cy="10" r="9"/></svg>',
};
try {
  // A custom builder consumes an ARG before rendering. The generated public
  // file proves the real builder received the exact value; no runtime ENV needed.
  await writeFile(path.join(root, 'build-argument-check.cjs'), `
const fs = require('node:fs');
const assert = require('node:assert/strict');
assert.equal(process.env.DOCS_BUILD_LABEL, ${JSON.stringify(buildLabel)});
assert.equal(process.env.EMPTY, '');
assert(!fs.existsSync('/tmp/SHOULD_NOT_EXIST'));
fs.mkdirSync('/app/public', {recursive:true});
fs.writeFileSync('/app/public/build-argument.json', JSON.stringify({label:process.env.DOCS_BUILD_LABEL}));
`);
  images.push(buildBase);
  execFileSync('docker', ['build', '--build-arg', `BASE=${process.env.ZELLIJ_IMAGE || 'zellij-ci'}`, '-f', '-', '-t', buildBase, root], {
    input: `ARG BASE\nFROM \${BASE}\nCOPY build-argument-check.cjs /app/build-argument-check.cjs\nRUN node -e 'const fs=require("fs");const p=JSON.parse(fs.readFileSync("/app/package.json"));p.scripts.prebuild += " && node /app/build-argument-check.cjs";fs.writeFileSync("/app/package.json",JSON.stringify(p));'\n`,
    stdio: ['pipe', 'inherit', 'inherit'], timeout: 120000,
  });
  for (const [relative, body] of Object.entries(source)) {
    const file = path.join(root, 'source', relative);
    await mkdir(path.dirname(file), { recursive: true });
    await writeFile(file, body);
  }
  const registry = await localRegistry(docker, containers, suffix);
  for (const kind of ['static', 'next']) {
    const prefix = `/manual-${kind}`;
    const tag = `${registry}/acceptance/ship-${kind}:${suffix}`;
    images.push(tag);
    run(['ship', 'source', '--out', `manual ${kind}`, '--image', kind, '--tag', tag, ...(kind === 'static' ? ['--assets', 'copy'] : []), '--features', kind === 'static' ? 'all' : 'none', '--base-path', prefix, '--site-url', 'https://imported.example', '--no-check', '--build-arg', 'DOCS_BUILD_LABEL=old', '--build-arg', `DOCS_BUILD_LABEL=${buildLabel}`, '--build-arg', 'EMPTY=']);
    pullPublishedImage(docker, tag);
    const name = `zel-ship-${kind}-${suffix}`;
    containers.push(name);
    docker(['run', '-d', '--name', name, '-p', '127.0.0.1::3000', tag]);
    const info = JSON.parse(docker(['inspect', name]))[0];
    assert(!info.Config.Env.some(value => value.startsWith('DOCS_BUILD_LABEL=') || value.startsWith('EMPTY=')));
    const host = `http://127.0.0.1:${info.NetworkSettings.Ports['3000/tcp'][0].HostPort}`;
    const origin = host + prefix;
    let ready = false;
    for (let n = 0; n < 120; n++) {
      try { if ((await fetch(`${origin}/guide`, { signal: AbortSignal.timeout(1000) })).ok) { ready = true; break; } } catch { /* startup */ }
      await new Promise(resolve => setTimeout(resolve, 250));
    }
    assert(ready, `Server did not start: ${docker(['logs', name])}`);
    assert.deepEqual(await (await fetch(`${origin}/build-argument.json`)).json(), { label: buildLabel });
    docker(['exec', name, 'sh', '-c', info.Config.Healthcheck.Test[1]]);
    const html = await (await fetch(`${origin}/guide`)).text();
    assert(html.includes(`href="https://imported.example${prefix}/guide"`));
    assert((await (await fetch(`${origin}/sitemap.xml`)).text()).includes(`https://imported.example${prefix}/guide`));
    assert.equal((await fetch(`${host}/guide`)).status, 404);
    const browser = await chromium.launch();
    try {
      const page = await browser.newPage();
      const errors = [];
      page.on('pageerror', error => errors.push(error.message));
      page.on('response', res => { if (res.status() >= 400) errors.push(`${res.status()} ${res.url()}`); });
      await page.goto(`${origin}/guide`);
      await page.getByRole('heading', { name: 'Imported manual', exact: true }).waitFor();
      if (kind === 'static') {
        await expect(page.locator('mjx-container svg')).toBeVisible({ timeout: 15000 });
        await expect(page.locator('.zellij-mermaid svg')).toBeVisible({ timeout: 15000 });
      } else {
        await expect(page.locator('mjx-container, .zellij-mermaid')).toHaveCount(0);
        assert((await page.locator('article').innerText()).includes('$x^2$'));
      }
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
    const saved = await readFile(path.join(root, `manual ${kind}`, 'zel.yaml'), 'utf8');
    assert(!/features|base-path|site-url|no-check|imported\.example|DOCS_BUILD_LABEL|build-arg/.test(saved));
    run(['pub', `manual ${kind}`, '--dry-run']);
    console.log(`ship ${kind}: imported, published, pulled and verified navigation/assets; source unchanged.`);
  }
} finally {
  for (const name of containers) { try { docker(['rm', '-f', name]); } catch { /* original failure */ } }
  for (const tag of images) { try { docker(['image', 'rm', tag]); } catch { /* failed build */ } }
  await rm(root, { recursive: true, force: true });
}
