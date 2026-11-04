// Real engine acceptance, run in CI only. No provider deployment is involved.
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { mkdtemp, mkdir, readFile, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
const zel = path.resolve('cli/target/debug/zel');
const root = await mkdtemp(path.join(tmpdir(), 'zellij-hosted-overrides-'));
const oldImage = `zel-old-overrides:${process.pid}-${Date.now()}`;
const source = 'name: Hosted overrides\nbasePath: /authored\nsiteUrl: https://authored.example\n';
const siteFile = path.join(root, 'content/site.yaml');
const output = path.join(root, 'site');
const run = (flags, extra = {}) => spawnSync(zel, ['build', '.', '--features', 'none', ...flags], { cwd: root, env: { ...process.env, ...extra }, encoding: 'utf8', timeout: 600000 });
const success = result => assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
const read = file => readFile(path.join(output, file), 'utf8');
try {
  await mkdir(path.join(root, 'content/manual'), { recursive: true });
  await writeFile(path.join(root, 'zel.yaml'), 'version: 1\nrenderer: hosted\n');
  await writeFile(siteFile, source);
  await writeFile(path.join(root, 'content/manual/_sidebar.yaml'), 'title: Manual\nnav: [index, next]\n');
  await writeFile(path.join(root, 'content/manual/index.mdx'), '# Hosted overrides\n\n[Next](/manual/next)\n');
  await writeFile(path.join(root, 'content/manual/next.mdx'), '# Next page\n');
  success(run(['--base-path', '/products/manual', '--site-url', 'https://docs.example']));
  const html = await read('manual.html');
  assert(html.includes('href="https://docs.example/products/manual/manual"'));
  assert(html.includes('/products/manual/_next/'));
  assert((await read('sitemap.xml')).includes('<loc>https://docs.example/products/manual/manual</loc>'));
  assert((await read('robots.txt')).includes('https://docs.example/products/manual/sitemap.xml'));
  assert.equal(await readFile(siteFile, 'utf8'), source);

  await writeFile(siteFile, 'name: []\n');
  const invalid = run(['--no-check', '--base-path', '/']);
  assert.notEqual(invalid.status, 0, invalid.stdout);
  assert.match(invalid.stdout + invalid.stderr, /Invalid|invalid/);
  assert.equal(await read('manual.html'), html, 'Preparation failure preserves the last export');
  await writeFile(siteFile, source);

  execFileSync('docker', ['build', '--build-arg', `BASE=${process.env.ZELLIJ_IMAGE || 'zellij-ci'}`, '-t', oldImage, '-'], {
    input: 'ARG BASE\nFROM ${BASE}\nRUN node -e \'const fs=require("fs");const p="/opt/zellij/package.json";const v=JSON.parse(fs.readFileSync(p));delete v.zellij.buildOverridesVersion;fs.writeFileSync(p,JSON.stringify(v));\'\n',
    stdio: ['pipe', 'inherit', 'inherit'], timeout: 120000,
  });
  const old = run(['--no-check'], { ZELLIJ_IMAGE: oldImage });
  assert.notEqual(old.status, 0);
  assert.match(old.stdout + old.stderr, /does not support build overrides/);
  assert.equal(await read('manual.html'), html, 'Incompatible engine preserves the last export');

  success(run(['--base-path', '/', '--no-check']));
  const atRoot = await read('manual.html');
  assert(atRoot.includes('href="https://authored.example/manual"'));
  assert(atRoot.includes('src="/_next/'));
  assert(!atRoot.includes('/authored/_next/'));
  assert.equal(await readFile(siteFile, 'utf8'), source);
  console.log('Real hosted overrides: canonical/sitemap/robots, root reset, no-check validation, old engine and output preservation passed.');
} finally {
  try { execFileSync('docker', ['image', 'rm', oldImage], { stdio: 'pipe' }); } catch { /* Build might not have run. */ }
  await rm(root, { recursive: true, force: true });
}
