// Run only on hosted Linux acceptance runners. Pull first; never build an image.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { readFile } from 'node:fs/promises';
import { setTimeout as delay } from 'node:timers/promises';
import { checkDeliverySite } from './check-delivery-http.mjs';

const kind = process.argv[2];
assert(['static', 'next'].includes(kind), 'Choose static or next');
const cases = JSON.parse(await readFile(new URL('./fixtures/release-delivery/cases.json', import.meta.url)));
const image = cases.find(entry => entry.name === kind).image;
const docker = args => execFileSync('docker', args, { encoding: 'utf8', timeout: 180000 }).trim();
const name = `zel-published-${kind}-${process.pid}`;
docker(['pull', image]);
const info = JSON.parse(docker(['image', 'inspect', image]))[0];
assert.equal(info.Architecture, 'amd64');
assert.equal(info.Config.User, kind === 'static' ? 'nginx' : 'nextjs');
assert(info.RepoDigests.length > 0, 'A registry digest must identify the pulled image');
let started = false;
try {
  docker(['run', '-d', '--name', name, '-p', '127.0.0.1::3000', image]);
  started = true;
  const container = JSON.parse(docker(['inspect', name]))[0];
  const origin = `http://127.0.0.1:${container.NetworkSettings.Ports['3000/tcp'][0].HostPort}/`;
  let ready = false;
  let lastFailure = 'no response';
  for (let attempt = 0; attempt < 120; attempt++) {
    try {
      const response = await fetch(origin, { signal: AbortSignal.timeout(1000) });
      await response.arrayBuffer();
      if (response.ok) { ready = true; break; }
      lastFailure = `HTTP ${response.status}`;
    } catch (error) { lastFailure = error.message; }
    await delay(250);
  }
  assert(ready, `Published container did not become ready (${lastFailure})`);
  docker(['exec', name, 'sh', '-c', info.Config.Healthcheck.Test[1]]);
  await checkDeliverySite(origin, 'portable');
  console.log(`${image}: fresh pull and HTTP acceptance passed (${info.RepoDigests.join(', ')})`);
} catch (error) {
  // This container contains only the public fixture, with no credentials.
  // Keep useful startup evidence before removing it on failed acceptance.
  if (started) {
    try { console.error(docker(['logs', '--tail', '80', name])); } catch { /* Retain the original failure. */ }
  }
  throw error;
} finally {
  if (started) docker(['rm', '--force', name]);
}
