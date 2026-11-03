// Regenerate manual acceptance workflows with the current generator, while
// pinning execution to the public version saved in each fixture configuration.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { cp, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import yaml from 'js-yaml';

const root = fileURLToPath(new URL('../', import.meta.url));
const binary = path.resolve(process.argv[2] || 'cli/target/debug/zel');
const write = process.argv[3] === '--write';
assert(process.argv.length <= 4 && (!process.argv[3] || write), 'Usage: prepare-delivery-acceptance.mjs <zel> [--write]');
const fixture = 'scripts/fixtures/release-delivery';
const cases = JSON.parse(await readFile(path.join(root, fixture, 'cases.json'), 'utf8'));
const temp = await mkdtemp(path.join(tmpdir(), 'zel-generated-delivery-'));
const env = { ...process.env };
delete env.NETLIFY_SITE_ID;
delete env.NETLIFY_AUTH_TOKEN;
try {
  await cp(path.join(root, fixture), path.join(temp, fixture), { recursive: true });
  execFileSync('git', ['init', '--quiet'], { cwd: temp });
  const outputs = new Set();
  const groups = new Set();
  const prepared = [];
  for (const entry of cases) {
    assert(/^acceptance-[a-z-]+\.yml$/.test(entry.workflow), 'Only acceptance workflows may be generated');
    assert(/^(portable|hosted)\/[a-z-]+\.yaml$/.test(entry.config), 'Expected fixture configuration');
    assert(!outputs.has(entry.workflow), 'Duplicate workflow output');
    outputs.add(entry.workflow);
    execFileSync(binary, ['ci', '--config', `${fixture}/${entry.config}`], { cwd: temp, env, encoding: 'utf8' });
    const relative = `.github/workflows/${entry.workflow}`;
    const generated = await readFile(path.join(temp, relative));
    const workflow = yaml.load(generated.toString());
    assert.deepEqual(Object.keys(workflow.on), ['pull_request', 'workflow_dispatch'], 'Acceptance only verifies PRs or runs manually');
    if (workflow.jobs.publish) {
      assert.equal(workflow.jobs.publish.if, "${{ (github.event_name == 'workflow_dispatch' && github.ref == 'refs/heads/main') }}", 'Publication requires a manual main run');
    }
    const group = workflow.concurrency.group.toLowerCase();
    assert(!groups.has(group), 'Independent acceptance workflows must not share a queue');
    groups.add(group);
    for (const job of Object.values(workflow.jobs)) {
      for (const step of job.steps) assert(!step.run?.includes('--prod'), 'Acceptance must create draft previews');
    }
    prepared.push({ relative, generated });
    if (!write) assert.deepEqual(await readFile(path.join(root, relative)), generated, `${relative} is stale; regenerate with --write`);
    console.log(`${entry.workflow}: ${write ? 'generated' : 'matches generator'} (PR verification; manual publication)`);
  }
  if (write) for (const { relative, generated } of prepared) await writeFile(path.join(root, relative), generated);
} finally {
  await rm(temp, { recursive: true, force: true });
}
