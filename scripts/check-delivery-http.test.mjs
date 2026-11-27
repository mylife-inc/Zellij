import assert from 'node:assert/strict';
import { test } from 'node:test';
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { checkDeliverySite } from './check-delivery-http.mjs';
import yaml from 'js-yaml';
import { imageInspectionMetadata, inspectionImage } from './delivery-acceptance-image.mjs';

for (const kind of ['static', 'next']) {
  test(`${kind} inspection is tied to the configured, generated publication`, async () => {
    const cases = JSON.parse(await readFile(new URL('./fixtures/release-delivery/cases.json', import.meta.url)));
    const entry = cases.find(entry => entry.name === kind);
    const bytes = await readFile(new URL(`./fixtures/release-delivery/${entry.config}`, import.meta.url));
    const config = yaml.load(bytes.toString());
    const workflow = yaml.load(await readFile(new URL(`../.github/workflows/${entry.workflow}`, import.meta.url), 'utf8'));
    const expected = imageInspectionMetadata(entry, config, workflow, bytes);
    assert.deepEqual({ image: entry.image, configSha256: entry.configSha256 }, expected);
    assert.equal(inspectionImage(entry, bytes), config.image.name);

    // The historical inspector silently checked rc.18 after publication moved
    // to rc.19. Both independent drift paths must fail before pulling an image.
    assert.throws(() => imageInspectionMetadata({ ...entry, image: expected.image.replace(/:[^:]+$/, ':0.2.0-rc.18') }, config, workflow, bytes), /inspection target is stale/);
    assert.throws(() => inspectionImage(entry, Buffer.concat([bytes, Buffer.from('\n# changed configuration\n')])), /stale/);
    assert.throws(() => inspectionImage({ ...entry, configSha256: undefined }, bytes), /stale/);
    assert.throws(() => inspectionImage({ ...entry, image: 'ghcrXio/other/image:old' }, bytes), /Unexpected/);
    const changed = structuredClone(workflow);
    changed.jobs.publish.steps.find(step => step.env?.IMAGE_TAG).env.IMAGE_TAG += '-wrong';
    assert.throws(() => imageInspectionMetadata(entry, config, changed, bytes), /Generated publication target/);
  });
}

async function site(t, renderer) {
  const portable = renderer === 'portable';
  const home = portable ? '/' : '/manual';
  const next = portable ? '/guide/next.html' : '/manual/next';
  const logo = portable ? '/assets/logo.svg' : '/_zellij/assets/logo.svg';
  const resources = new Map([
    [home, `<html><head><link rel="stylesheet" href="/site.css"></head><body><h1>${portable ? 'Candidate delivery acceptance' : 'Hosted delivery acceptance'}</h1><img alt="Acceptance logo" src="${logo}"></body></html>`],
    [next, portable ? 'zirconium-acceptance' : 'zirconium-hosted-acceptance'],
    [logo, await readFile(new URL(`./fixtures/release-delivery/${renderer}/content/assets/logo.svg`, import.meta.url), 'utf8')],
    ['/site.css', 'body { color: black; }'],
  ]);
  if (portable) resources.set('/index.html', resources.get('/'));
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

test('accepts a portable Next wrapper whose home is / and has no /index.html route', async t => {
  const fixture = await site(t, 'portable');
  fixture.resources.delete('/index.html');
  assert.equal((await fetch(new URL('index.html', fixture.base))).status, 404);
  await checkDeliverySite(fixture.base, 'portable');
});

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
