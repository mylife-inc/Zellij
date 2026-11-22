/** Real CLI download/cache/build acceptance; no external tools on the child's PATH. */
import { spawn, execFileSync } from 'node:child_process';
import { createServer } from 'node:http';
import { mkdtemp, mkdir, readFile, writeFile, cp, rm, readdir } from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import assert from 'node:assert/strict';
import { checkStandaloneAuthoring } from './check-standalone-authoring.mjs';
const repo = path.resolve(import.meta.dirname, '..');
const binary = path.resolve(process.argv[2] ?? `cli/target/debug/zel${process.platform === 'win32' ? '.exe' : ''}`);
const target = JSON.parse(execFileSync(binary, ['version', '--json'], { encoding: 'utf8' })).cli.target;
const platform = `${target.includes('windows') ? 'win' : target.includes('darwin') ? 'darwin' : 'linux'}-${target.startsWith('aarch64') ? 'arm64' : 'x64'}`;
const version = (await readFile(path.join(repo, 'cli/Cargo.toml'), 'utf8')).match(/^version = "([^"]+)"/m)[1];
const directory = path.resolve(process.argv[3] ?? 'dist/renderer-packages');
const archive = `zel-renderer-v${version}-${platform}.tar.gz`;
const publicBase = process.env.ZEL_TEST_ENGINE_BASE_URL;
const downloads = new Map(publicBase ? [] : await Promise.all([archive, archive + '.sha256'].map(async name => [name, await readFile(path.join(directory, name))])));
const root = await mkdtemp(path.join(os.tmpdir(), 'zel rich 日本 '));
const requests = [];
const server = createServer((req,res) => {
  requests.push(req.url);const data = downloads.get(decodeURIComponent(req.url.slice(1)));
  res.writeHead(data ? 200 : 404);res.end(data ?? 'not found');
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const env = { ...process.env, PATH: path.join(root, 'empty-path'), ZELLIJ_HOME: path.join(root, 'resources café'), ZEL_ENGINE_BASE_URL: publicBase ?? `http://127.0.0.1:${server.address().port}` };
delete env.ZEL_ENGINE_CACHE;
const engineCache = path.join(env.ZELLIJ_HOME, 'cache/engines');
await mkdir(env.PATH);await cp(path.join(repo, 'examples/showcase/content'), path.join(root, 'content'), { recursive: true });
await writeFile(path.join(root, 'zel.yaml'), 'version: 1\nrenderer: hosted\n');
async function run(args, code = 0, overrides = {}) {
  return new Promise((resolve,reject) => {
    const child = spawn(binary, args, { cwd: root, env: { ...env, ...overrides }, windowsHide: true });let output = '';
    child.stdout.on('data', d => output += d);child.stderr.on('data', d => output += d);
    child.on('error', reject);child.on('exit', status => {
      try { assert.equal(status, code, `${args.join(' ')}\n${output}`);resolve(output); } catch(error) { reject(error); }
    });
  });
}
try {
  assert.match(await run(['build', '--offline'], 2), /not cached/);
  assert.equal(requests.length, 0);
  const doctor = JSON.parse(await run(['doctor', '--json']));
  assert.equal(doctor.runtime.status, 'not_required');assert.equal(doctor.rich_engine.status, 'download_required');
  assert.equal(doctor.rich_engine.cache, path.join(engineCache, `${version}-${platform}`));
  if (!publicBase) {
    const checksum = downloads.get(archive + '.sha256');
    downloads.set(archive + '.sha256', Buffer.from(`${'0'.repeat(64)}  ${archive}\n`));
    assert.match(await run(['build'], 2), /checksum mismatch/);
    assert.equal((await readdir(engineCache)).length, 0);
    downloads.set(archive + '.sha256', checksum); requests.length = 0;
  }
  const coldCheck = JSON.parse(await run(['check', '--target', 'static', '--json']));
  assert.equal(coldCheck.target, 'static'); assert.equal(coldCheck.ok, true);
  await run(['build']);assert.equal(requests.length, publicBase ? 0 : 2);
  const before = await readFile(path.join(root, 'site/index.html'));
  const manifest = JSON.parse(await readFile(path.join(root, 'site/_zellij/file/manifest.json')));
  assert.equal(manifest.pages, 16);
  assert.match(before.toString(), /file\/styles\.css/);
  await new Promise(resolve => server.close(resolve));
  await run(['build', '--offline']);
  assert.match(await run(['build', '--offline'], 2, { ZELLIJ_HOME: path.join(root, 'other-home') }), /not cached/);
  const overrideDoctor = JSON.parse(await run(['doctor', '--json'], 0, { ZELLIJ_HOME: path.join(root, 'other-home'), ZEL_ENGINE_CACHE: engineCache }));
  assert.equal(overrideDoctor.rich_engine.status, 'cached');
  assert.equal(overrideDoctor.rich_engine.cache, path.join(engineCache, `${version}-${platform}`));
  assert.equal(requests.length, publicBase ? 0 : 2);
  assert((await readFile(path.join(root, 'site/index.html'))).equals(before));
  await run(['new', 'docs', 'full', '--full', '--guide']);
  const guide = path.join(root, 'full/content/01-guide/index.mdx');
  await writeFile(guide, await readFile(guide, 'utf8') + '\n## Local maths\n\n$$\n\\sum_{k=1}^{n} k = \\frac{n(n+1)}{2}\n$$\n');
  await run(['build', 'full', '--offline']);
  const complete = JSON.parse(await readFile(path.join(root, 'full/site/_zellij/file/manifest.json')));
  assert.equal(complete.pages, 28, 'Every built-in section and the math guide must render');
  assert.match(await readFile(path.join(root, 'full/site/guide/index.html'), 'utf8'), /<mjx-container/,
    'MathJax must run from the packaged engine without resolving system packages');
  const pageList = path.join(root, 'full/content/pages.yaml');
  const authoredPages = await readFile(pageList, 'utf8');
  await writeFile(pageList, authoredPages.replace('- name: home', '- name: home\n  route: /../escape'));
  assert.match(await run(['build', 'full', '--offline', '--no-check'], 3), /offline route/);
  assert(!(await readdir(path.join(root, 'full'))).includes('escape'), 'An authored route must not write outside the staged output');
  await writeFile(pageList, authoredPages);
  await writeFile(pageList, authoredPages.replace('- name: home', '- name: home\n  route: /index'));
  assert.match(await run(['build', 'full', '--target', 'static', '--offline', '--no-check'], 3), /HTTP output routes collide/);
  await writeFile(pageList, authoredPages);
  await run(['build', '--offline', '--base-path', '/manual', '--site-url', 'https://docs.example.test', '--out', 'prefix-site']);
  const prefixed = await readFile(path.join(root, 'prefix-site/index.html'), 'utf8');
  assert.match(prefixed, /rel="canonical" href="https:\/\/docs\.example\.test\/manual\/"/);
  assert.match(prefixed, /href="\.\/_zellij\/file\/styles\.css"/);
  const sitePath = path.join(root, 'content/site.yaml');
  const sourceSite = await readFile(sitePath, 'utf8');
  // This acceptance runner intentionally has no npm dependencies. Change the
  // known showcase fixture only; fail visibly if its analytics block changes.
  const analyticsBlock = /^analytics:\r?\n  provider: none\s*$/m;
  assert.match(sourceSite, analyticsBlock);
  await writeFile(sitePath, sourceSite.replace(analyticsBlock,
    'analytics:\n  provider: plausible\n  domain: docs.example.test\n'));
  await run(['build', '--target', 'static', '--offline', '--base-path', '/manual', '--site-url', 'https://docs.example.test', '--out', 'http-site']);
  const httpManifest = JSON.parse(await readFile(path.join(root, 'http-site/_zellij/static/manifest.json')));
  assert.equal(httpManifest.target, 'static'); assert.equal(httpManifest.pages, manifest.pages);
  const httpHome = await readFile(path.join(root, 'http-site/index.html'), 'utf8');
  assert.match(httpHome, /rel="canonical" href="https:\/\/docs\.example\.test\/manual\/"/);
  assert.match(httpHome, /href="\/manual\/_zellij\/static\/styles\.css"/);
  assert.match(httpHome, /src="\/manual\/_zellij\/static\/runtime\.js"/);
  assert.match(httpHome, /name="robots" content="index, follow"/);
  assert.match(httpHome, /\$Analytics/);
  assert.doesNotMatch(httpHome, /_next\//);
  for (const route of httpManifest.routes) {
    assert((await readFile(path.join(root, 'http-site', route === '/' ? 'index.html' : route.slice(1) + '.html'))).length > 0);
  }
  const notFound = await readFile(path.join(root, 'http-site/404.html'), 'utf8');
  assert.match(notFound, /name="robots" content="noindex"/);
  assert.match(notFound, /href="\/manual\/"/);
  assert.match(await readFile(path.join(root, 'http-site/sitemap.xml'), 'utf8'), /<changefreq>weekly<\/changefreq>/);
  await run(['build', '--offline', '--out', 'file-with-analytics']);
  assert.doesNotMatch(await readFile(path.join(root, 'file-with-analytics/index.html'), 'utf8'), /\$Analytics/);
  await writeFile(sitePath, sourceSite);
  assert.equal(requests.length, publicBase ? 0 : 2, 'HTTP export reuses the same renderer download');
  const content = path.join(root, 'content/site.yaml');const original = await readFile(content);
  await writeFile(content, 'broken: [');await run(['build', '--offline'], 3);
  assert((await readFile(path.join(root, 'site/index.html'))).equals(before));await writeFile(content, original);
  const cached = path.join(engineCache, `${version}-${platform}`);
  const originalStyles = await readFile(path.join(cached, 'engine/styles.css'));
  await writeFile(path.join(cached, 'engine/styles.css'), 'tampered');
  assert.match(await run(['build', '--offline'], 2), /integrity verification/);
  assert((await readFile(path.join(root, 'site/index.html'))).equals(before));
  // Restore exactly the publisher's bytes, then produce an artifact for file:// QA.
  await writeFile(path.join(cached, 'engine/styles.css'), originalStyles);
  if (process.env.ZEL_STANDALONE_RESULT) {
    await mkdir(process.env.ZEL_STANDALONE_RESULT, { recursive: true });
    await cp(path.join(root, 'prefix-site'), process.env.ZEL_STANDALONE_RESULT, { recursive: true });
  }
  await checkStandaloneAuthoring(binary, env, root);
  assert.equal((await readdir(root)).some(n => n.startsWith('.zel-build-')), false);
  console.log(JSON.stringify({ platform, initialDownloads: requests.length, pages: manifest.pages, completeStarterPages: complete.pages, cachedOfflineBuild: true, cachedHttpBuild: true, failedBuildPreservedSite: true, tamperRejected: true, routeTraversalRejected: true, metadataOverrides: true, systemTools: false }));
} finally { server.close();await rm(root, { recursive: true, force: true }); }
