// Exercise the installed, self-contained checker and optionally the actual Rust
// command/container. Generated HTML verifies the accepted routes and anchors.
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { copyFile, mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repo = fileURLToPath(new URL('../', import.meta.url));
const root = await mkdtemp(path.join(tmpdir(), 'zellij-validation-'));
const content = path.join(root, 'docs with spaces');
const packaged = path.join(root, 'node_modules/@shebka/zellij');
const cli = path.join(packaged, 'bin/zellij.mjs');
const binary = process.argv[2] ? path.resolve(process.argv[2]) : undefined;
const site = 'name: Validation fixture\n';
const intro = '# Introduction\n\n## Start here\n\n[Details](/manual/nested/detail?mode=full#details)\n\n![Local](/_zellij/assets/caf%C3%A9.svg)\n';
const hero = 'type: hero\nheadline: Validation fixture\nvisual: assets/café.svg\ncta: [{label: Start, href: /manual/intro#start-here}]\n';
const files = {
  'site.yaml': site,
  'pages.yaml': '- name: home\n  sections: [hero]\n',
  'sections/hero.yaml': hero,
  '01-guide/_sidebar.yaml': 'route: /manual\nnav: [intro, nested/detail]\n',
  '01-guide/intro.mdx': intro,
  '01-guide/nested/detail.mdx': '# Nested\n\n## Details\n\n[Back](../intro#start-here)\n',
  'assets/café.svg': '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><circle cx="5" cy="5" r="4"/></svg>',
};
try {
  for (const [name, body] of Object.entries(files)) {
    await mkdir(path.dirname(path.join(content, name)), { recursive: true });
    await writeFile(path.join(content, name), body);
  }
  await mkdir(path.join(packaged, 'bin'), { recursive: true });
  await mkdir(path.join(packaged, 'dist/cli'), { recursive: true });
  await writeFile(path.join(packaged, 'package.json'), '{"type":"module"}');
  await copyFile(path.join(repo, 'engine/packages/zellij/bin/zellij.mjs'), cli);
  await copyFile(path.join(repo, 'engine/packages/zellij/dist/cli/check-bundle.js'), path.join(packaged, 'dist/cli/check-bundle.js'));
  const commands = [[process.execPath, [cli, 'check']]];
  if (binary) commands.push([binary, ['check']]);
  function check(args, success, matches = []) {
    for (const [program, prefix] of commands) {
      const result = spawnSync(program, [...prefix, ...args, content], { encoding: 'utf8', cwd: root });
      if (result.error) throw result.error;
      const output = result.stdout + result.stderr;
      assert.equal(result.status === 0, success, `${program}: unexpected exit ${result.status}\n${output}`);
      for (const match of matches) assert.match(output, match);
    }
  }
  check(['--deep', '--strict'], true);
  for (const flag of ['--depp', '--strict=false']) {
    const result = spawnSync(process.execPath, [cli, 'check', content, flag]);
    assert.notEqual(result.status, 0, `Unknown flag ${flag} must fail`);
  }
  if (binary) {
    const output = path.join(root, 'site');
    execFileSync(binary, ['build', content, '--out', output], { cwd: root, stdio: 'inherit' });
    const html = await readFile(path.join(output, 'manual/intro.html'), 'utf8');
    assert(html.includes('id="start-here"'));
    assert(html.includes('/manual/nested/detail?mode=full#details'));
    assert(html.includes('/_zellij/assets/caf%C3%A9.svg'));
    assert((await readFile(path.join(output, 'manual/nested/detail.html'), 'utf8')).includes('id="details"'));
    assert.equal(await readFile(path.join(output, '_zellij/assets/café.svg'), 'utf8'), files['assets/café.svg']);
  }
  await writeFile(path.join(content, 'sections/hero.yaml'), hero.replace('/manual/intro#start-here', '/missing').replace('assets/café.svg', 'assets/missing.svg'));
  await writeFile(path.join(content, '01-guide/intro.mdx'), intro + '\n<Callout>\n');
  check([], true);
  check(['--deep'], false, [/sections\/hero.yaml/, /no page resolves/, /cannot read asset/, /invalid MDX syntax/]);
  await writeFile(path.join(content, 'sections/hero.yaml'), hero);
  await writeFile(path.join(content, '01-guide/intro.mdx'), intro);
  await writeFile(path.join(content, 'site.yaml'), site + 'futureOption: true\n');
  check([], true);
  check(['--strict'], false, [/unknown key/, /Strict check failed/]);
  await writeFile(path.join(content, 'site.yaml'), site);
  check(['--deep', '--strict'], true);
  console.log(`Deep/strict validation passed through the isolated package${binary ? ', Rust container command and actual export' : ''}.`);
} finally {
  await rm(root, { recursive: true, force: true });
}
