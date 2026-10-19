// Exercise a real foreign-architecture build, registry round trip and HTTP
// serving. CI enables ARM64 emulation explicitly; this script never installs it.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { localRegistry } from './fixtures/local-registry.mjs';

const zel = path.resolve(process.argv[2] ?? 'cli/target/debug/zel');
const root = await mkdtemp(path.join(tmpdir(), 'zellij-platform-'));
const suffix = `${process.pid}-${Date.now()}`;
const containers = [];
const images = [];
const docker = args => execFileSync('docker', args, { encoding: 'utf8', timeout: 120_000 }).trim();
const run = args => execFileSync(zel, args, { cwd: root, stdio: 'inherit', timeout: 300_000 });
try {
  await mkdir(path.join(root, 'content'));
  await writeFile(path.join(root, 'content/index.md'), '# ARM64 publication\n\n[Next](next.md)\n');
  await writeFile(path.join(root, 'content/next.md'), '# Next page\n');
  const registry = await localRegistry(docker, containers, suffix);
  const tag = `${registry}/acceptance/platform:${suffix}`;
  images.push(tag);
  await writeFile(path.join(root, 'zel.yaml'), `version: 1\nrenderer: portable\nimage:\n  name: '${tag}'\n  platform: linux/arm64\npublish: {action: push, image: static}\n`);
  run(['pub']);
  docker(['image', 'rm', tag]);
  docker(['pull', '--platform', 'linux/arm64', tag]);
  const image = JSON.parse(docker(['image', 'inspect', tag]))[0];
  assert.equal(image.Os, 'linux');
  assert.equal(image.Architecture, 'arm64');
  assert(image.RepoDigests.some(digest => digest.includes('@sha256:')));
  const container = `zel-platform-${suffix}`;
  containers.push(container);
  docker(['run', '-d', '--platform', 'linux/arm64', '--name', container, '-p', '127.0.0.1::3000', tag]);
  const details = JSON.parse(docker(['inspect', container]))[0];
  const port = details.NetworkSettings.Ports['3000/tcp'][0].HostPort;
  let ready = false;
  for (let i = 0; i < 120; i++) {
    try {
      const response = await fetch(`http://127.0.0.1:${port}/index.html`, { signal: AbortSignal.timeout(1000) });
      if (response.ok && (await response.text()).includes('ARM64 publication')) { ready = true; break; }
    } catch { /* startup */ }
    await new Promise(resolve => setTimeout(resolve, 250));
  }
  assert(ready, `ARM64 server failed: ${docker(['logs', container])}`);
  const next = await fetch(`http://127.0.0.1:${port}/next.html`, { signal: AbortSignal.timeout(5000) });
  assert.equal(next.status, 200);
  assert((await next.text()).includes('Next page'));
  // An explicit override must win over saved ARM64, without publishing.
  const local = `zel-platform-override:${suffix}`;
  images.push(local);
  run(['si', '--tag', local, '--platform', 'linux/amd64']);
  assert.equal(JSON.parse(docker(['image', 'inspect', local]))[0].Architecture, 'amd64');
  console.log('Image platforms: ARM64 build/push/pull/HTTP and AMD64 override passed.');
} finally {
  for (const container of containers.reverse()) {
    try { docker(['rm', '-f', container]); } catch { /* preserve failure */ }
  }
  for (const image of images) {
    try { docker(['image', 'rm', image]); } catch { /* failed builds have no image */ }
  }
  await rm(root, { recursive: true, force: true });
}
