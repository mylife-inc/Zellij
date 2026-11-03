// Public-site checks shared by live draft previews and fresh registry pulls.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';

export async function checkDeliverySite(base, renderer) {
  assert(['portable', 'hosted'].includes(renderer), 'Expected portable or hosted renderer');
  const origin = new URL(base);
  assert(['http:', 'https:'].includes(origin.protocol) && !origin.username && !origin.password && !origin.search && !origin.hash, 'Expected a public HTTP(S) site URL');
  if (!origin.pathname.endsWith('/')) origin.pathname += '/';
  async function response(relative, status = 200) {
    const url = new URL(relative, origin);
    assert.equal(url.origin, origin.origin, 'Acceptance requests stay on the selected site');
    const res = await fetch(url, { signal: AbortSignal.timeout(15000) });
    assert.equal(res.status, status, `${url.pathname}: HTTP ${res.status}`);
    return res;
  }
  const home = renderer === 'portable' ? 'index.html' : 'manual';
  const next = renderer === 'portable' ? 'guide/next.html' : 'manual/next';
  const html = await (await response(home)).text();
  assert(html.includes(renderer === 'portable' ? 'Candidate delivery acceptance' : 'Hosted delivery acceptance'));
  const nextHtml = await (await response(next)).text();
  assert(nextHtml.includes(renderer === 'portable' ? 'zirconium-acceptance' : 'zirconium-hosted-acceptance'));
  const image = [...html.matchAll(/<img\b[^>]*>/g)].find(([tag]) => tag.includes('Acceptance logo'))?.[0];
  assert(image, 'Authored logo is rendered');
  const src = image.match(/\bsrc="([^"]+)"/)?.[1].replaceAll('&amp;', '&');
  assert(src, 'Logo has a URL');
  const logoUrl = new URL(src, new URL(home, origin));
  const logo = await (await response(logoUrl.href)).text();
  const expectedLogo = await readFile(new URL(`./fixtures/release-delivery/${renderer}/content/assets/logo.svg`, import.meta.url), 'utf8');
  assert.equal(logo, expectedLogo, 'Published logo bytes match the fixture');
  // Check actual rendered local styles, fonts and scripts, not guessed asset names.
  const resources = new Set([...html.matchAll(/<(?:script|link)\b[^>]*>/g)]
    .filter(([tag]) => !/\brel="(?:preconnect|dns-prefetch)"/.test(tag))
    .flatMap(([tag]) => [...tag.matchAll(/\b(?:src|href)="([^"]+)"/g)].map(match => match[1].replaceAll('&amp;', '&'))));
  for (const resource of resources) {
    const url = new URL(resource, new URL(home, origin));
    if (url.origin !== origin.origin) continue;
    await (await response(url.href)).arrayBuffer();
  }
  for (const source of ['zel.yaml', 'site.yaml', 'index.md', 'content/index.md', 'missing-acceptance-page']) {
    await (await response(source, 404)).arrayBuffer();
  }
  console.log(`${renderer}: published pages, linked assets and source-file isolation passed at ${origin}`);
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  await checkDeliverySite(process.argv[2], process.argv[3]);
}
