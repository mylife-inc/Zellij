// Real image acceptance: both renderers, including hosted basePath, run on an
// unprivileged HTTP server. No registry pushes or external deployment.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { chromium } from '@playwright/test';

const zel = path.resolve(process.argv[2] ?? 'cli/target/debug/zel');
const root = await mkdtemp(path.join(tmpdir(), 'zellij-static-images-'));
const suffix = `${process.pid}-${Date.now()}`;
const images = [];
const containers = [];
const docker = (args) => execFileSync('docker', args, { encoding: 'utf8', timeout: 120_000 }).trim();
const run = (args) => execFileSync(zel, args, { cwd: root, stdio: 'inherit', timeout: 600_000 });
const asset = '<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20"><circle cx="10" cy="10" r="9"/></svg>';
async function write(relative, body) {
  const file = path.join(root, relative);
  await mkdir(path.dirname(file), { recursive: true });
  await writeFile(file, body);
}
async function response(origin, url, status = 200) {
  const res = await fetch(`${origin}${url}`, { signal: AbortSignal.timeout(5000), redirect: 'manual' });
  assert.equal(res.status, status, `${url}: ${res.status}`);
  return res;
}
async function resources(origin, html) {
  const urls = new Set([...html.matchAll(/(?:src|href)="([^"]+)"/g)]
    .map((m) => m[1].replaceAll('&amp;', '&')).filter((url) => url.startsWith('/') && !url.startsWith('//')));
  assert(urls.size > 0);
  for (const url of urls) {
    const res = await fetch(new URL(url, origin), { signal: AbortSignal.timeout(5000) });
    assert.equal(res.status, 200, `Missing generated resource ${url}`);
    await res.arrayBuffer();
  }
}
async function serve(tag, kind, prefix) {
  const name = `zel-static-${kind}-${suffix}`;
  containers.push(name);
  docker(['run', '-d', '--name', name, '--read-only', '--tmpfs', '/tmp', '-p', '127.0.0.1::3000', tag]);
  const details = JSON.parse(docker(['inspect', name]))[0];
  assert.equal(details.Config.User, 'nginx');
  const port = details.NetworkSettings.Ports['3000/tcp'][0].HostPort;
  const origin = `http://127.0.0.1:${port}`;
  let ready = false;
  for (let attempt = 0; attempt < 60; attempt++) {
    try {
      const res = await fetch(`${origin}${prefix}/index.html`, { signal: AbortSignal.timeout(1000) });
      if (res.status === 200) { ready = true; break; }
    } catch { /* Wait for NGINX to start. */ }
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  assert(ready, `Server did not become ready: ${docker(['logs', name])}`);
  docker(['exec', name, 'sh', '-c', details.Config.Healthcheck.Test[1]]);
  const html = await (await response(origin, `${prefix}/index.html`)).text();
  await response(origin, `${prefix}/missing-route`, 404);
  await response(origin, `${prefix}/.zellij-output`, 404);
  await response(origin, `${prefix}/zel.yaml`, 404);
  await response(origin, `${prefix}/site.yaml`, 404);
  await response(origin, `${prefix}/index.md`, 404);
  if (kind === 'portable') {
    assert(html.includes('Portable images'));
    assert(html.includes('nested/page.html#details'));
    assert(html.includes('https://company.example'));
    const page = await (await response(origin, '/nested/page.html?source=image')).text();
    assert(page.includes('id="details"'));
    await response(origin, '/nested/page');
    const redirect = await response(origin, '/nested', 301);
    assert.equal(redirect.headers.get('location'), '/nested/');
    assert((await (await response(origin, '/nested/')).text()).includes('Nested index'));
    assert.equal(await (await response(origin, '/assets/logo.svg')).text(), asset);
    const pdf = await response(origin, '/assets/Manual%20copy.pdf');
    assert.equal(pdf.headers.get('content-type'), 'application/pdf');
    await response(origin, '/unreferenced.txt', 404);
    // Portable references are relative, so resolve them against the actual page.
    for (const m of html.matchAll(/(?:src|href)="([^"]+)"/g)) {
      if (/^(?:https?:|#)/.test(m[1])) continue;
      assert.equal((await fetch(new URL(m[1], `${origin}/index.html`))).status, 200, m[1]);
    }
  } else {
    const manual = await (await response(origin, `${prefix}/manual`)).text();
    assert(manual.includes('Hosted images'));
    assert((await (await response(origin, `${prefix}/manual/next`)).text()).includes('More documentation'));
    assert.equal(await (await response(origin, `${prefix}/_zellij/assets/logo.svg`)).text(), asset);
    const search = await (await response(origin, `${prefix}/_zellij/search-index.json`)).json();
    assert(search.some((entry) => entry.route === '/manual/next'));
    if (prefix) await response(origin, '/manual', 404);
    await resources(origin, manual);
    // Next serializes an unprefixed redirect digest in exported HTML; its
    // browser router applies basePath. Verify navigation, not that wire format.
    const browser = await chromium.launch();
    try {
      const page = await browser.newPage();
      const failures = [];
      page.on('pageerror', (error) => failures.push(error.message));
      page.on('response', (res) => { if (res.status() >= 400) failures.push(`${res.status()} ${res.url()}`); });
      await page.goto(`${origin}${prefix}/`);
      await page.waitForURL(`${origin}${prefix}/manual`);
      await page.getByRole('link', { name: 'Next', exact: true }).click();
      await page.waitForURL(`${origin}${prefix}/manual/next`);
      await page.reload();
      assert((await page.textContent('body')).includes('More documentation'));
      assert.deepEqual(failures, [], 'Browser errors or missing resources');
    } finally {
      await browser.close();
    }
  }
  console.log(`${kind}: routes, assets, health check and source isolation passed (${prefix || '/'}).`);
}

try {
  await write('docs/zellij/zel.yaml', 'version: 1\nrenderer: portable\nname: Portable images\ntheme: forest\nbrand:\n  logo: {src: assets/logo.svg, alt: Company, href: https://company.example}\nnav:\n  - {page: index.md, label: Welcome}\n  - label: Guide\n    children:\n      - {page: nested/page.md}\n');
  await write('docs/zellij/content/index.md', '# Portable images\n\n[Details](nested/page.md#details)\n\n[PDF](assets/Manual%20copy.pdf)\n');
  await write('docs/zellij/content/nested/page.md', '# Another page\n\n## Details\n\n[Home](../index.md)\n');
  await write('docs/zellij/content/nested/index.md', '# Nested index\n');
  await write('docs/zellij/content/assets/logo.svg', asset);
  await write('docs/zellij/content/assets/Manual copy.pdf', '%PDF-1.4\nfixture\n');
  await write('docs/zellij/content/unreferenced.txt', 'not public');
  await write('docs/zellij/site/index.html', 'previous output');
  const portable = `zellij-static-portable:${suffix}`;
  images.push(portable);
  run(['si', '--tag', portable]);
  assert.equal(await readFile(path.join(root, 'docs/zellij/site/index.html'), 'utf8'), 'previous output');
  await serve(portable, 'portable', '');

  // Two hosted builds cover both deployment roots using the same source.
  await write('hosted/zel.yaml', 'version: 1\nrenderer: hosted\n');
  await write('hosted/content/01-guide/_sidebar.yaml', 'title: Manual\nroute: /manual\nnav: [index, next]\n');
  await write('hosted/content/01-guide/index.mdx', '# Hosted images\n\n[Next](/manual/next)\n\n![Logo](assets/logo.svg)\n');
  await write('hosted/content/01-guide/next.mdx', '# Next page\n\nMore documentation.\n');
  await write('hosted/content/assets/logo.svg', asset);
  for (const [label, prefix] of [['root', ''], ['prefix', '/products/.docs']]) {
    await write('hosted/content/site.yaml', `name: Hosted images\nhome: 01-guide\nbasePath: '${prefix}'\nnav: { search: true, themeToggle: true }\n`);
    const tag = `zellij-static-hosted-${label}:${suffix}`;
    images.push(tag);
    run(['image', 'static', 'hosted', '--tag', tag]);
    await serve(tag, 'hosted-' + label, prefix);
  }
} finally {
  for (const name of containers) {
    try { docker(['rm', '-f', name]); } catch { /* Keep the original failure. */ }
  }
  for (const tag of images) {
    try { docker(['image', 'rm', tag]); } catch { /* Failed builds have no tag. */ }
  }
  await rm(root, { recursive: true, force: true });
}
