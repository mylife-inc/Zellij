// CI only. Native jobs replay generated publication commands against a local
// registry, save tested images, and the final job replays the generated merge.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdir, mkdtemp, readFile, rm, writeFile, appendFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import YAML from 'yaml';
import { localRegistry } from './fixtures/local-registry.mjs';
import { externalNative, externalManifest } from './fixtures/external-ci.mjs';

const [mode, directory] = process.argv.slice(2);
assert(['amd64', 'arm64', 'combine'].includes(mode) && directory);
const artifact = path.resolve(directory);
const zel = path.resolve('cli/target/debug/zel');
const root = await mkdtemp(path.join(tmpdir(), 'zel-multi-ci-'));
const containers = [], images = [], sizes = [];
const suffix = `${process.pid}-${Date.now()}`;
const docker = args => execFileSync('docker', args, { encoding: 'utf8', timeout: 120000 }).trim();
const run = args => execFileSync(zel, args, { cwd: root, stdio: 'inherit', timeout: 600000 });
const write = async (file, content) => { const target = path.join(root, file); await mkdir(path.dirname(target), { recursive: true }); await writeFile(target, content); };
async function script(step, env) {
  // Every user-controlled value stays in the environment, including paths.
  execFileSync('bash', ['-c', step.run], { cwd: root, env: { ...process.env, ...env }, stdio: 'inherit', timeout: 600000 });
}
// Docker's Size field differs between classic and containerd image stores.
// Registry layer descriptors measure compressed bytes consistently on both.
async function compressedLayers(registry, repository, reference, arch) {
  const response = await fetch(`http://${registry}/v2/${repository}/manifests/${reference}`, {
    headers: { Accept: 'application/vnd.oci.image.index.v1+json, application/vnd.oci.image.manifest.v1+json, application/vnd.docker.distribution.manifest.list.v2+json, application/vnd.docker.distribution.manifest.v2+json' },
    signal: AbortSignal.timeout(10000),
  });
  assert(response.ok, `Registry manifest HTTP ${response.status}`);
  const manifest = await response.json();
  if (manifest.manifests) {
    const platform = manifest.manifests.find(m => m.platform.os === 'linux' && m.platform.architecture === arch);
    assert(platform, `Missing ${arch} image`);
    return compressedLayers(registry, repository, platform.digest, arch);
  }
  assert(Array.isArray(manifest.layers));
  return manifest.layers.reduce((total, layer) => total + layer.size, 0);
}
async function serve(tag, kind) {
  const name = `zel-multi-${kind}-${suffix}`;
  containers.push(name);
  docker(['run', '-d', '--name', name, '-p', '127.0.0.1::3000', tag]);
  const info = JSON.parse(docker(['inspect', name]))[0];
  const origin = `http://127.0.0.1:${info.NetworkSettings.Ports['3000/tcp'][0].HostPort}`;
  const home = kind === 'next' ? '/' : '/index.html';
  let ready = false;
  let lastStatus;
  for (let n = 0; n < 120; n++) {
    try {
      const res = await fetch(`${origin}${home}`, { signal: AbortSignal.timeout(1000) });
      lastStatus = res.status;
      if (res.ok && (await res.text()).includes('Native publication')) { ready = true; break; }
    } catch { /* startup */ }
    await new Promise(resolve => setTimeout(resolve, 250));
  }
  assert(ready, `${kind} home ${home}, last HTTP ${lastStatus}: ${docker(['logs', name])}`);
  assert((await (await fetch(`${origin}/next.html`)).text()).includes('Second document'));
  assert.equal((await fetch(`${origin}/index.md`)).status, 404);
  docker(['exec', name, 'sh', '-ec', info.Config.Healthcheck.Test[1]]);
  docker(['exec', name, 'sh', '-ec', 'test -z "$(ls -A /var/cache/apk)"']);
  if (kind === 'static') {
    docker(['exec', name, 'sh', '-ec', 'test ! -e /usr/sbin/nginx-debug']);
    assert.equal(info.Config.StopSignal, 'SIGQUIT');
    assert.deepEqual(info.Config.Entrypoint, ['/docker-entrypoint.sh']);
  }
  if (kind === 'next') docker(['exec', name, 'sh', '-ec', 'node --version; ! command -v npm; ! command -v yarn; test ! -d /usr/local/include/node']);
}
try {
  await mkdir(artifact, { recursive: true });
  const registry = await localRegistry(docker, containers, suffix);
  if (mode !== 'combine') {
    execFileSync('git', ['init', '--quiet'], { cwd: root });
    await write('content/index.md', '# Native publication\n\n[Next](next.md)\n');
    await write('content/next.md', '# Second document\n');
    const saved = [];
    for (const kind of ['static', 'next']) {
      await write('zel.yaml', `version: 1\nrenderer: portable\nimage: {name: registry.example/acceptance/${kind}:final, platform: all}\npublish: {action: push, image: ${kind}}\n`);
      run(['ci', '--force', '--features', 'none', '--build-arg', 'LABEL=literal 日本語']);
      const workflow = YAML.parse(await readFile(path.join(root, '.github/workflows/docs.yml'), 'utf8'));
      const job = workflow.jobs[`publish_${mode}`];
      const step = job.steps.at(-1);
      const output = path.join(root, `output-${kind}`);
      const env = { ...workflow.env, ...job.env, ...step.env, IMAGE_REPOSITORY: `${registry}/acceptance/${kind}`, GITHUB_RUN_ID: suffix, GITHUB_RUN_ATTEMPT: '1', GITHUB_OUTPUT: output, PATH: `${path.dirname(zel)}:${process.env.PATH}`, ZELLIJ_IMAGE: process.env.ZELLIJ_IMAGE };
      assert.equal(env.DOCKER_CONTEXT, 'default');
      const preflight = job.steps.find(step => step.name === 'Check the configured Docker runtime');
      assert(preflight, 'Native publication needs its generated runtime preflight');
      await script(preflight, env);
      await script(step, env);
      const digest = (await readFile(output, 'utf8')).trim().slice('image='.length);
      assert.match(digest, /@sha256:[a-f0-9]{64}$/);
      const built = `${env.IMAGE_REPOSITORY}:zel-${suffix}-1-${mode}`;
      images.push(built);
      docker(['image', 'rm', built]);
      docker(['pull', digest]);
      const tag = `zel-multi-acceptance-${kind}:${mode}`;
      docker(['tag', digest, tag]); images.push(tag); saved.push(tag);
      const info = JSON.parse(docker(['image', 'inspect', tag]))[0];
      assert.equal(info.Architecture, mode);
      assert(!info.Config.Env.some(value => value.startsWith('LABEL=')));
      await serve(tag, kind);
      // Same application payload on the former runtime base, so measured
      // savings reflect runtime trimming rather than a smaller fixture site.
      const baseline = `zel-multi-baseline-${kind}:${mode}`; images.push(baseline);
      const base = kind === 'static' ? 'nginxinc/nginx-unprivileged:stable-alpine' : 'node:24-alpine';
      const payload = kind === 'static' ? '/usr/share/nginx/html' : '/app';
      execFileSync('docker', ['build', '-f', '-', '-t', baseline, root], { input: `FROM ${base}\nCOPY --from=${tag} ${payload}/ ${payload}/\n`, stdio: ['pipe', 'inherit', 'inherit'], timeout: 300000 });
      const oldSize = JSON.parse(docker(['image', 'inspect', baseline]))[0].Size;
      assert(info.Size < oldSize - 1000000, `${kind}: expected a smaller runtime, got ${info.Size} versus ${oldSize}`);
      const baselineTag = `${registry}/acceptance/${kind}:baseline-${mode}`; images.push(baselineTag);
      docker(['tag', baseline, baselineTag]); docker(['push', baselineTag]);
      const currentCompressedBytes = await compressedLayers(registry, `acceptance/${kind}`, digest.split('@')[1], mode);
      const previousCompressedBytes = await compressedLayers(registry, `acceptance/${kind}`, `baseline-${mode}`, mode);
      assert(currentCompressedBytes < previousCompressedBytes - 1000000, `${kind}: compressed layers did not shrink`);
      sizes.push({ kind, arch: mode, dockerReportedBytes: info.Size, previousDockerReportedBytes: oldSize, currentCompressedBytes, previousCompressedBytes, savedCompressedBytes: previousCompressedBytes - currentCompressedBytes });
      await writeFile(path.join(artifact, `${kind}.workflow.json`), JSON.stringify(workflow));
    }
    await externalNative({ root, zel, arch: mode, registry, artifact, suffix, docker, images });
    docker(['save', '--output', path.join(artifact, 'images.tar'), ...saved]);
    await writeFile(path.join(artifact, 'sizes.json'), JSON.stringify(sizes, null, 2));
  } else {
    for (const arch of ['amd64', 'arm64']) {
      docker(['load', '--input', path.join(artifact, arch, 'images.tar')]);
      sizes.push(...JSON.parse(await readFile(path.join(artifact, arch, 'sizes.json'), 'utf8')));
    }
    for (const kind of ['static', 'next']) {
      const digests = {};
      const platformDigests = {};
      for (const arch of ['amd64', 'arm64']) {
        const local = `zel-multi-acceptance-${kind}:${arch}`; images.push(local);
        const tag = `${registry}/acceptance/${kind}:${arch}`; images.push(tag);
        docker(['tag', local, tag]); docker(['push', tag]);
        digests[arch] = JSON.parse(docker(['image', 'inspect', tag]))[0].RepoDigests.find(d => d.startsWith(`${registry}/acceptance/${kind}@`));
        assert(digests[arch]);
        const digest = digests[arch].split('@')[1];
        const manifest = await (await fetch(`http://${registry}/v2/acceptance/${kind}/manifests/${digest}`, {
          headers: { Accept: 'application/vnd.oci.image.index.v1+json, application/vnd.oci.image.manifest.v1+json, application/vnd.docker.distribution.manifest.list.v2+json, application/vnd.docker.distribution.manifest.v2+json' },
        })).json();
        platformDigests[arch] = manifest.manifests?.find(m => m.platform.os === 'linux' && m.platform.architecture === arch)?.digest ?? digest;
      }
      const workflow = JSON.parse(await readFile(path.join(artifact, 'amd64', `${kind}.workflow.json`), 'utf8'));
      const job = workflow.jobs.publish;
      const step = job.steps.at(-1);
      const env = { ...job.env, ...step.env, IMAGE_TAG: `${registry}/acceptance/${kind}:final`, AMD64_IMAGE: digests.amd64, ARM64_IMAGE: digests.arm64 };
      assert.equal(env.DOCKER_CONTEXT, 'default');
      const preflight = job.steps.find(step => step.name === 'Check the configured Docker runtime');
      assert(preflight, 'Manifest publication needs its generated runtime preflight');
      await script(preflight, env);
      await script(step, env);
      if (kind === 'static') await externalManifest({ root, registry, artifact, digests });
      const index = await (await fetch(`http://${registry}/v2/acceptance/${kind}/manifests/final`, { headers: { Accept: 'application/vnd.oci.image.index.v1+json, application/vnd.docker.distribution.manifest.list.v2+json' } })).json();
      const platforms = index.manifests.filter(m => m.platform.os === 'linux');
      assert.deepEqual(platforms.map(m => `${m.platform.os}/${m.platform.architecture}`).sort(), ['linux/amd64', 'linux/arm64']);
      for (const item of platforms) assert.equal(item.digest, platformDigests[item.platform.architecture]);
    }
  }
  console.log(JSON.stringify(sizes, null, 2));
  if (process.env.GITHUB_STEP_SUMMARY) await appendFile(process.env.GITHUB_STEP_SUMMARY, `\nRuntime image size comparison (identical site payload). Compressed sizes sum registry layer bytes, excluding small manifests/configuration and attestations. Docker-reported sizes use each runner's image-store accounting.\n\n\`\`\`json\n${JSON.stringify(sizes, null, 2)}\n\`\`\`\n`);
} finally {
  for (const container of containers.reverse()) { try { docker(['rm', '-f', container]); } catch { /* original failure wins */ } }
  for (const image of images) { try { docker(['image', 'rm', image]); } catch { /* failed builds */ } }
  await rm(root, { recursive: true, force: true });
}
