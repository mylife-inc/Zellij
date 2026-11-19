// Production publication for this repository, independent of CLI releases.
// Build/browser validation runs in a separate job without Netlify credentials.
import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { readFile, writeFile, appendFile } from 'node:fs/promises';
import { promisify } from 'node:util';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

export const repository = 'mylife-inc/Zellij';
export const siteId = '91774174-d4c2-41b5-aa4b-ac61eeadbf51';
export const domain = 'zellij.shebka.net';
export const routes = ['/', '/cli', '/cli/build', '/cli/templates', '/cli/books', '/sections/compose'];
const run = promisify(execFile);
const shaPattern = /^[a-f0-9]{40}$/;
const titlePattern = /^Zellij documentation ([a-f0-9]{40})$/;

export function assertContext(env) {
  assert.equal(env.GITHUB_REPOSITORY, repository, 'Production publication belongs to mylife-inc/Zellij');
  assert.equal(env.GITHUB_REF, 'refs/heads/main', 'Only main can publish production documentation');
  assert(['push', 'workflow_dispatch'].includes(env.GITHUB_EVENT_NAME), 'Unsupported publication event');
  assert(shaPattern.test(env.GITHUB_SHA ?? ''), 'Missing source commit');
  assert(env.NETLIFY_AUTH_TOKEN, 'Set the dedicated NETLIFY_DOCS_PRODUCTION_TOKEN Actions secret');
  assert(env.GH_TOKEN, 'Missing read-only GitHub Actions token');
  assert(env.DOCS_OUTPUT && env.NETLIFY_CLI, 'Missing checked artifact or deployment client');
}

export function assertSite(site) {
  assert.equal(site.id, siteId, 'Unexpected Netlify project');
  assert.equal(site.custom_domain, domain, 'Production domain does not match the intended project');
  assert(site.published_deploy?.id, 'Expected an existing production deployment');
  assert(!site.published_deploy.locked, 'Production deployment is locked');
}

export function mainContent(html) {
  const content = html.match(/<main(?:\s[^>]*)?>([\s\S]*?)<\/main>/i)?.[1];
  assert(content, 'Expected documentation main content, not a challenge or error page');
  return content;
}

// Response bodies can include account data. Fail with status/path only.
export function apiClient(base, token, fetcher = fetch) {
  return async (route, method = 'GET') => {
    const response = await fetcher(base + route, {
      method, redirect: 'error', signal: AbortSignal.timeout(60_000),
      headers: { authorization: `Bearer ${token}`, accept: 'application/json' },
    });
    assert(response.ok, `${method} ${new URL(base).hostname}${route} returned HTTP ${response.status}`);
    return response.json();
  };
}

export async function checkPublished(origin, output, fetcher = fetch) {
  for (const route of routes) {
    const file = route === '/' ? 'index.html' : `${route.slice(1)}.html`;
    const expected = mainContent(await readFile(path.join(output, file), 'utf8'));
    const response = await fetcher(origin + route, { redirect: 'error', signal: AbortSignal.timeout(30_000) });
    assert.equal(response.status, 200, `${origin}${route}: expected HTTP 200`);
    assert.equal(mainContent(await response.text()), expected, `${route}: hosted content differs from the checked artifact`);
  }
  for (const file of ['favicon.ico', '_zellij/assets/logo.png']) {
    const response = await fetcher(`${origin}/${file}`, { redirect: 'error', signal: AbortSignal.timeout(30_000) });
    assert.equal(response.status, 200, `${file}: expected HTTP 200`);
    assert(Buffer.from(await response.arrayBuffer()).equals(await readFile(path.join(output, file))), `${file}: hosted bytes changed`);
  }
}

export async function publish({ env = process.env, netlify, github, deploy, check = checkPublished, save = async () => {} } = {}) {
  assertContext(env);
  netlify ??= apiClient('https://api.netlify.com/api/v1', env.NETLIFY_AUTH_TOKEN);
  github ??= apiClient(`https://api.github.com/repos/${repository}`, env.GH_TOKEN);
  deploy ??= async () => {
    // No --prod: the ready draft is checked before the one promotion request.
    // Run outside the checkout so local build/config discovery cannot redirect it.
    let result;
    try {
      result = await run(env.NETLIFY_CLI, ['deploy', '--no-build', '--draft', '--site', siteId,
        '--dir', path.resolve(env.DOCS_OUTPUT), '--message', `Zellij documentation ${env.GITHUB_SHA}`, '--json'],
      { cwd: env.RUNNER_TEMP, env, timeout: 8 * 60_000, maxBuffer: 4 * 1024 * 1024 });
    } catch {
      throw new Error('Netlify draft upload failed; production was not promoted. Check Netlify deploy logs.');
    }
    return JSON.parse(result.stdout);
  };
  const source = env.GITHUB_SHA;
  const verifyMain = async () => {
    const head = await github('/git/ref/heads/main');
    assert(shaPattern.test(head.object?.sha ?? ''), 'Invalid main branch reference');
    const relation = await github(`/compare/${source}...${head.object.sha}`);
    assert(['ahead', 'identical'].includes(relation.status), 'Source commit is no longer on main');
  };
  await verifyMain();
  const original = await netlify(`/sites/${siteId}`);
  assertSite(original);
  const publishedSource = original.published_deploy.title?.match(titlePattern)?.[1];
  if (publishedSource) {
    const relation = await github(`/compare/${publishedSource}...${source}`);
    if (relation.status === 'behind') {
      const receipt = { skipped: true, reason: 'A newer source commit is already published', source, current_deploy: original.published_deploy.id };
      await save(receipt); return receipt;
    }
    assert(['ahead', 'identical'].includes(relation.status), 'Published history diverged; review before replacing production');
  }
  const draft = await deploy();
  assert.equal(draft.site_id, siteId, 'Draft uploaded to an unexpected project');
  assert(/^[a-f0-9]{24}$/.test(draft.deploy_id ?? ''), 'Invalid draft deployment ID');
  const candidate = await netlify(`/deploys/${draft.deploy_id}`);
  assert.equal(candidate.site_id, siteId);
  assert.equal(candidate.state, 'ready', 'Draft is not ready');
  assert.deepEqual(candidate.required ?? [], []);
  assert.deepEqual(candidate.required_functions ?? [], []);
  assert.equal(candidate.title, `Zellij documentation ${source}`, 'Draft source receipt mismatch');
  const preview = new URL(draft.deploy_url);
  assert.equal(preview.protocol, 'https:');
  assert.equal(preview.hostname, `${draft.deploy_id}--${original.name}.netlify.app`, 'Unexpected draft origin');
  assert.equal(preview.port, '', 'Unexpected draft port');
  assert.equal(preview.pathname, '/');
  assert(!preview.username && !preview.password && !preview.search && !preview.hash);
  await check(preview.origin, env.DOCS_OUTPUT);
  await verifyMain();
  const beforePromotion = await netlify(`/sites/${siteId}`);
  assertSite(beforePromotion);
  assert.equal(beforePromotion.published_deploy.id, original.published_deploy.id,
    'Production changed during validation; draft retained without promotion');
  const receipt = { source, site_id: siteId, domain, deploy_id: draft.deploy_id,
    previous_deploy_id: original.published_deploy.id, preview: preview.origin, state: 'draft-verified' };
  await save(receipt);
  // Never retry mutations automatically: an uncertain response needs inspection.
  await netlify(`/sites/${siteId}/deploys/${draft.deploy_id}/restore`, 'POST');
  const live = await netlify(`/sites/${siteId}`);
  assertSite(live);
  assert.equal(live.published_deploy.id, draft.deploy_id, 'Production promotion could not be confirmed');
  receipt.state = 'published';
  receipt.published_at = live.published_deploy.published_at;
  await save(receipt);
  await check(`https://${domain}`, env.DOCS_OUTPUT);
  receipt.state = 'verified';
  await save(receipt);
  return receipt;
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  try {
    const receipt = await publish({ save: async receipt => {
      await writeFile(process.env.DOCS_RECEIPT, JSON.stringify(receipt, null, 2) + '\n');
      if (process.env.GITHUB_STEP_SUMMARY) await appendFile(process.env.GITHUB_STEP_SUMMARY,
        `Documentation: **${receipt.state ?? 'skipped'}** — source ${receipt.source}\n\n` +
        (receipt.deploy_id ? `Deploy: ${receipt.deploy_id}; previous deploy: ${receipt.previous_deploy_id}.\n\n` : `${receipt.reason}.\n\n`));
    } });
    console.log(JSON.stringify(receipt));
  } catch (error) {
    // Do not serialize request/child-process objects or credential-bearing env.
    console.error(error.message); process.exitCode = 1;
  }
}
