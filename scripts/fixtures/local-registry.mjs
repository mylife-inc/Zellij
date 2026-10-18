// A disposable loopback registry for acceptance. Nothing is published to an
// external account. The caller owns cleanup through its containers list.
import assert from 'node:assert/strict';

export async function localRegistry(docker, containers, suffix) {
  const name = `zel-publication-registry-${suffix}`;
  containers.push(name);
  docker(['run', '--rm', '-d', '--name', name, '-e', 'OTEL_TRACES_EXPORTER=none', '-p', '127.0.0.1::5000', 'registry:3']);
  const info = JSON.parse(docker(['inspect', name]))[0];
  const port = info.NetworkSettings.Ports['5000/tcp'][0].HostPort;
  const host = `localhost:${port}`;
  let ready = false;
  for (let attempt = 0; attempt < 60; attempt++) {
    try {
      if ((await fetch(`http://127.0.0.1:${port}/v2/`, { signal: AbortSignal.timeout(1000) })).ok) {
        ready = true;
        break;
      }
    } catch { /* startup */ }
    await new Promise(resolve => setTimeout(resolve, 250));
  }
  assert(ready, `Registry did not start: ${docker(['logs', name])}`);
  return host;
}

export function pullPublishedImage(docker, tag) {
  // Remove the local tag so the later HTTP/browser checks run the artifact
  // retrieved from the registry, not just the original local build result.
  docker(['image', 'rm', tag]);
  docker(['pull', tag]);
  const image = JSON.parse(docker(['image', 'inspect', tag]))[0];
  assert(image.RepoDigests.some(digest => digest.startsWith(tag.slice(0, tag.lastIndexOf(':')) + '@sha256:')));
}
