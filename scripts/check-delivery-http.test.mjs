import assert from 'node:assert/strict';
import { test } from 'node:test';
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { checkDeliverySite } from './check-delivery-http.mjs';

async function site(t, renderer) {
  const portable = renderer === 'portable';
  const home = portable ? '/index.html' : '/manual';
  const next = portable ? '/guide/next.html' : '/manual/next';
  const logo = portable ? '/assets/logo.svg' : '/_zellij/assets/logo.svg';
  const resources = new Map([
    [home, `<html><head><link rel="stylesheet" href="/site.css"></head><body><h1>${portable ? 'Candidate delivery acceptance' : 'Hosted delivery acceptance'}</h1><img alt="Acceptance logo" src="${logo}"></body></html>`],
    [next, portable ? 'zirconium-acceptance' : 'zirconium-hosted-acceptance'],
    [logo, await readFile(new URL(`./fixtures/release-delivery/${renderer}/content/assets/logo.svg`, import.meta.url), 'utf8')],
    ['/site.css', 'body { color: black; }'],
  ]);
  const server = createServer((req, res) => {
    const body = resources.get(req.url);
    res.writeHead(body === undefined ? 404 : 200);
    res.end(body ?? 'missing');
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  t.after(() => { server.closeAllConnections(); return new Promise(resolve => server.close(resolve)); });
  return { base: `http://127.0.0.1:${server.address().port}/`, resources, next, logo };
}

for (const renderer of ['portable', 'hosted']) {
  test(`accepts the complete ${renderer} delivery fixture`, async t => {
    const fixture = await site(t, renderer);
    await checkDeliverySite(fixture.base, renderer);
  });
}

test('rejects a missing page or stylesheet, changed logo bytes, and exposed source files', async t => {
  for (const change of [
    fixture => fixture.resources.delete(fixture.next),
    fixture => fixture.resources.delete('/site.css'),
    fixture => fixture.resources.set(fixture.logo, '<svg>wrong asset</svg>'),
    fixture => fixture.resources.set('/zel.yaml', 'authored source must not be public'),
  ]) {
    const fixture = await site(t, 'portable');
    change(fixture);
    await assert.rejects(checkDeliverySite(fixture.base, 'portable'));
  }
});
