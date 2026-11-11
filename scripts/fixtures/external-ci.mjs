// GitHub-hosted acceptance only. Replay generated GitLab/CircleCI shell jobs
// with the candidate binary and a loopback registry, not published rc assets.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdir, readFile, readdir, rm, writeFile } from 'node:fs/promises';
import path from 'node:path';
import YAML from 'yaml';
import { releasePolicy } from '../release-policy.mjs';

const jobScript = (doc, system, job) => system === 'gitlab'
  ? doc[job].script[0]
  : doc.jobs[job].steps.find(step => step.run)?.run.command;
function replaceOnce(script, before, after) {
  assert.equal(script.split(before).length, 2, `Expected exactly one fixture replacement: ${before}`);
  return script.replace(before, after);
}
async function execute(script, root, env) {
  execFileSync('bash', ['-c', script], { cwd: root, env: { ...process.env, ...env }, stdio: 'inherit', timeout: 600000 });
}
export async function externalNative({ root, zel, arch, registry, artifact, suffix, docker, images }) {
  const reported = JSON.parse(execFileSync(zel, ['version', '--json'], { encoding: 'utf8' }));
  const { version } = releasePolicy(`v${reported.cli.version}`);
  await writeFile(path.join(root, 'zel.yaml'), 'version: 1\nrenderer: portable\nimage: {name: registry.example/acceptance/external:final, platform: all}\npublish: {action: push, image: static}\n');
  for (const system of ['gitlab', 'circle']) {
    execFileSync(zel, ['ci', '--system', system, '--force'], { cwd: root, stdio: 'inherit' });
    const file = system === 'gitlab' ? '.gitlab-ci.yml' : '.circleci/config.yml';
    const doc = YAML.parse(await readFile(path.join(root, file), 'utf8'));
    await writeFile(path.join(artifact, `${system}.workflow.json`), JSON.stringify(doc));
    let script = jobScript(doc, system, `zel_publish_${arch}`);
    assert(script.includes('curl --fail') && script.includes('sh "$ZEL_WORK/install.sh"'));
    // Replace only the pinned download/install block. The remainder (checks,
    // login, native build, push, digest validation, receipt, cleanup) is emitted.
    const start = script.indexOf('export ZEL_VERSION=');
    const end = script.indexOf('export SITE_ROOT=');
    assert(start > 0 && end > start);
    script = script.slice(0, start) + script.slice(end);
    script = replaceOnce(script, `export ZELLIJ_IMAGE='ghcr.io/mylife-inc/zellij-base:${version}'`, `export ZELLIJ_IMAGE='${process.env.ZELLIJ_IMAGE}'`);
    script = script.replaceAll('registry.example/acceptance/external', `${registry}/acceptance/external`);
    script = replaceOnce(script, "export ZEL_LOGIN_REGISTRY='registry.example'", `export ZEL_LOGIN_REGISTRY='${registry}'`);
    await execute(script, root, { PATH: `${path.dirname(zel)}:${process.env.PATH}`, CI_BUILDS_DIR: path.dirname(root), CI_PIPELINE_ID: suffix, CI_JOB_ID: system, CIRCLE_WORKFLOW_ID: suffix, CIRCLE_BUILD_NUM: system, REGISTRY_USERNAME: 'fixture', REGISTRY_PASSWORD: 'fixture' });
    const receiptDir = (await readdir(root)).filter(name => name.startsWith('.zel-ci-'));
    assert.equal(receiptDir.length, 1);
    const digest = (await readFile(path.join(root, receiptDir[0], `${arch}.digest`), 'utf8')).trim();
    assert.match(digest, /@sha256:[a-f0-9]{64}$/);
    const tag = `${registry}/acceptance/external:zel-${suffix}-${system}-${arch}`;
    images.push(tag);
    assert.equal(JSON.parse(docker(['image', 'inspect', tag]))[0].Architecture, arch);
    assert.equal(docker(['image', 'inspect', '--format', '{{index .RepoDigests 0}}', tag]), digest);
    await rm(path.join(root, receiptDir[0]), { recursive: true });
  }
}
export async function externalManifest({ root, registry, artifact, digests }) {
  for (const system of ['gitlab', 'circle']) {
    const doc = JSON.parse(await readFile(path.join(artifact, 'amd64', `${system}.workflow.json`), 'utf8'));
    let script = jobScript(doc, system, 'zel_manifest');
    // Existing tested per-architecture static images have been pushed into this
    // job's registry. Use their immutable receipts; still run emitted validation.
    script = script.replaceAll('registry.example/acceptance/external', `${registry}/acceptance/static`);
    script = replaceOnce(script, "export ZEL_LOGIN_REGISTRY='registry.example'", `export ZEL_LOGIN_REGISTRY='${registry}'`);
    const receipt = script.match(/'(\.zel-ci-[a-f0-9]+)\/amd64\.digest'/)?.[1];
    assert(receipt);
    await mkdir(path.join(root, receipt), { recursive: true });
    for (const arch of ['amd64', 'arm64']) await writeFile(path.join(root, receipt, `${arch}.digest`), `${digests[arch]}\n`);
    await execute(script, root, { CI_BUILDS_DIR: path.dirname(root), REGISTRY_USERNAME: 'fixture', REGISTRY_PASSWORD: 'fixture' });
    await rm(path.join(root, receipt), { recursive: true });
  }
}
