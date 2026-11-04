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
let builder;
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
  // The CLI uses the selected builder but never creates/starts one. Acceptance
  // explicitly provides a disposable Buildx builder with loopback registry access.
  const config = path.join(root, 'buildkit.toml');
  await writeFile(config, `[registry."${registry}"]\n  http = true\n`);
  builder = `zel-multi-${suffix}`;
  docker(['buildx', 'create', '--name', builder, '--driver', 'docker-container', '--driver-opt', 'network=host', '--buildkitd-config', config, '--use']);
  const multi = `${registry}/acceptance/combined:${suffix}`;
  images.push(multi);
  run(['pub', '--tag', multi, '--platform', 'all']);
  const index = await (await fetch(`http://${registry}/v2/acceptance/combined/manifests/${suffix}`, {
    headers: { Accept: 'application/vnd.oci.image.index.v1+json, application/vnd.docker.distribution.manifest.list.v2+json' },
  })).json();
  assert.deepEqual(index.manifests.filter(m => m.platform.os === 'linux').map(m => m.platform.architecture).sort(), ['amd64', 'arm64']);
  for (const arch of ['amd64', 'arm64']) {
    docker(['pull', '--platform', `linux/${arch}`, multi]);
    const id = `zel-combined-${arch}-${suffix}`;
    containers.push(id);
    docker(['run', '-d', '--platform', `linux/${arch}`, '--name', id, '-p', '127.0.0.1::3000', multi]);
    const info = JSON.parse(docker(['inspect', id]))[0];
    const url = `http://127.0.0.1:${info.NetworkSettings.Ports['3000/tcp'][0].HostPort}/next.html`;
    let ok = false;
    for (let n = 0; n < 60; n++) {
      try { const res = await fetch(url); if (res.ok && (await res.text()).includes('Next page')) { ok = true; break; } } catch { /* startup */ }
      await new Promise(resolve => setTimeout(resolve, 250));
    }
    assert(ok, `${arch} combined image failed to serve`);
  }
  console.log('Image platforms: single-platform compatibility and combined Buildx publication/pull/HTTP passed for AMD64 and ARM64.');
} finally {
  if (builder) { try { docker(['buildx', 'rm', builder]); } catch { /* preserve failure */ } }
  for (const container of containers.reverse()) {
    try { docker(['rm', '-f', container]); } catch { /* preserve failure */ }
  }
  for (const image of images) {
    try { docker(['image', 'rm', image]); } catch { /* failed builds have no image */ }
  }
  await rm(root, { recursive: true, force: true });
}
