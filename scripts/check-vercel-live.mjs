// Disposable Vercel project lifecycle for Actions-only acceptance. Credentials
// stay in the environment; the journal contains only target IDs and evidence.
import assert from 'node:assert/strict';
import {randomBytes} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {readFile, rename, writeFile} from 'node:fs/promises';
import {fileURLToPath} from 'node:url';
import {checkDeliverySite} from './check-delivery-http.mjs';

const sessionPattern = /^zellij-live-\d{8}-[0-9a-f]{6}$/;
const teamPattern = /^team_[A-Za-z0-9]{1,123}$/;
const projectPattern = /^prj_[A-Za-z0-9]{1,124}$/;

export function validateJournal(value) {
  assert(value?.version === 1 && sessionPattern.test(value.session), 'Expected a disposable Vercel journal');
  assert(teamPattern.test(value.team), 'Expected an explicit Vercel team ID');
  assert(/^\d+$/.test(value.run ?? '') && /^[0-9a-f]{40}$/.test(value.source ?? ''), 'Expected Actions source/run identities');
  assert(Array.isArray(value.cases), 'Expected acceptance cases');
  if (value.project !== undefined) assert(projectPattern.test(value.project), 'Expected the created Vercel project ID');
  return value;
}

export function verifyProject(project, journal) {
  validateJournal(journal);
  assert(projectPattern.test(project?.id) && project.name === journal.session && project.accountId === journal.team,
    'Vercel project name or team changed; refusing to use it');
  if (journal.project !== undefined) assert.equal(project.id, journal.project, 'Vercel project ID changed');
  return project.id;
}

export function deploymentReceipt(stdout) {
  const match = stdout.match(/Vercel deployment (dpl_[A-Za-z0-9]+) is ready\r?\n\s+(https:\/\/[^\s]+)/);
  assert(match, 'zel did not report a ready Vercel deployment');
  const url = new URL(match[2]);
  assert(url.protocol === 'https:' && /^[a-z0-9-]+\.vercel\.app$/.test(url.hostname)
    && !url.username && !url.password && !url.port && url.pathname === '/' && !url.search && !url.hash,
  'Expected the fixture deployment URL on vercel.app');
  return {deployment:match[1], url:url.href};
}

export function vercelApi(token, team, fetcher = fetch) {
  assert(typeof token === 'string' && /^[\x21-\x7e]+$/.test(token), 'Set VERCEL_TOKEN to a nonempty token without whitespace');
  assert(teamPattern.test(team), 'Expected an explicit Vercel team ID');
  return async (method, path, body, allowMissing = false) => {
    const url = new URL(path, 'https://api.vercel.com');
    assert(url.origin === 'https://api.vercel.com' && !url.username && !url.password && !url.search && !url.hash, 'Unexpected Vercel API path');
    url.searchParams.set('teamId', team);
    let response;
    try {
      response = await fetcher(url, {method, redirect:'error', signal:AbortSignal.timeout(30_000),
        headers:{Authorization:`Bearer ${token}`, 'Content-Type':'application/json'},
        ...(body === undefined ? {} : {body:JSON.stringify(body)})});
    } catch {
      throw new Error(`Vercel ${method} ${path}: request failed; remote outcome may be unknown`);
    }
    if (allowMissing && response.status === 404) return null;
    assert(response.ok, `Vercel ${method} ${path}: HTTP ${response.status}; response body omitted`);
    if (response.status === 204) return null;
    // Do not put provider bodies into exceptions or the evidence journal.
    try {
      const reader = response.body.getReader();
      const chunks = [];
      let length = 0;
      for (;;) {
        const {done, value} = await reader.read();
        if (done) break;
        length += value.length;
        if (length > 2 * 1024 * 1024) { await reader.cancel(); throw new Error(); }
        chunks.push(value);
      }
      const data = JSON.parse(Buffer.concat(chunks).toString('utf8'));
      assert(data && typeof data === 'object' && !Array.isArray(data) && !data.error);
      return data;
    } catch {
      throw new Error(`Vercel ${method} ${path}: invalid or oversized response; body omitted`);
    }
  };
}

export async function createFixture(journal, api, save) {
  validateJournal(journal);
  assert(journal.project === undefined && journal.creation === 'not-started', 'Refusing to reuse a fixture journal');
  const existing = await api('GET', `/v9/projects/${journal.session}`, undefined, true);
  assert(existing === null, 'Fixture name already exists; no existing project will be changed');
  journal.creation = 'requested';
  await save(journal); // Persist the name before the possibly ambiguous POST.
  const project = await api('POST', '/v11/projects', {
    name:journal.session, framework:null, ssoProtection:null, publicSource:false,
  });
  journal.project = verifyProject(project, journal);
  journal.creation = 'confirmed';
  await save(journal);
  // Public previews are intentional only on this freshly created fixture.
  assert(project.ssoProtection === null, 'Vercel preview protection remains enabled; inspect the temporary project');
  return journal;
}

export async function cleanupFixture(journal, api, save) {
  validateJournal(journal);
  if (!journal.project) {
    assert(journal.creation === 'not-started',
      'Project creation outcome is unknown; inspect the recorded team/name before manual cleanup');
    journal.cleanup = {status:'not-created'};
    await save(journal);
    return;
  }
  assert(journal.creation === 'confirmed', 'Cleanup requires a confirmed creation receipt');
  const path = `/v9/projects/${journal.project}`;
  const project = await api('GET', path, undefined, true);
  if (project !== null) {
    verifyProject(project, journal);
    journal.cleanup = {status:'requested'};
    await save(journal);
    await api('DELETE', path, undefined, true);
    assert.equal(await api('GET', path, undefined, true), null, 'Temporary Vercel project still exists after deletion');
  }
  journal.cleanup = {status:project === null ? 'already-absent' : 'deleted', verifiedAt:new Date().toISOString()};
  await save(journal);
}

export async function exerciseFixture(journal, api, zel, check, save) {
  validateJournal(journal);
  assert(journal.creation === 'confirmed' && journal.project && !journal.cleanup, 'Expected a fresh confirmed fixture');
  assert.equal(journal.cases.length, 0, 'Refusing to replay tests in the same journal');
  const project = await api('GET', `/v9/projects/${journal.project}`);
  verifyProject(project, journal);
  assert(project.ssoProtection === null, 'Fixture preview protection must be disabled');
  for (const production of [false, true]) {
    for (const renderer of ['portable', 'hosted']) {
      const item = {renderer, mode:production ? 'production' : 'preview', status:'started'};
      journal.cases.push(item);
      await save(journal);
      try {
        const args = ['vc', '--config', `scripts/fixtures/release-delivery/${renderer}/${renderer}.yaml`, '--site', journal.project];
        if (production) args.push('--prod');
        const stdout = await zel(args);
        Object.assign(item, deploymentReceipt(stdout));
        await save(journal); // Record the deployment before checking public HTTP.
        await check(item.url, renderer);
        item.status = 'passed';
      } catch (error) {
        item.status = 'failed';
        await save(journal);
        throw error;
      }
      await save(journal);
    }
  }
}

async function waitForHttp(url, renderer) {
  const deadline = Date.now() + 180_000;
  for (;;) {
    try { await checkDeliverySite(url, renderer); return; }
    catch (error) {
      if (Date.now() >= deadline) throw error;
      console.log(`Waiting for ${renderer} HTTP acceptance at ${url}: ${error.message}`);
      await new Promise(resolve => setTimeout(resolve, 5000));
    }
  }
}

async function main(action, path, binary) {
  assert(['create', 'exercise', 'cleanup'].includes(action) && path, 'Pass create|exercise|cleanup and a journal path');
  assert(process.env.GITHUB_ACTIONS === 'true' && process.env.GITHUB_REPOSITORY === 'mylife-inc/Zellij', 'Run this lifecycle in Zellij GitHub Actions');
  const api = vercelApi(process.env.VERCEL_TOKEN, process.env.VERCEL_TEAM_ID);
  const save = async journal => {
    await writeFile(`${path}.tmp`, JSON.stringify(journal, null, 2) + '\n');
    await rename(`${path}.tmp`, path);
  };
  let journal;
  if (action === 'create') {
    journal = validateJournal({version:1,
      session:`zellij-live-${new Date().toISOString().slice(0, 10).replaceAll('-', '')}-${randomBytes(3).toString('hex')}`,
      team:process.env.VERCEL_TEAM_ID, run:process.env.GITHUB_RUN_ID, source:process.env.GITHUB_SHA,
      creation:'not-started', startedAt:new Date().toISOString(), cases:[]});
    await writeFile(path, JSON.stringify(journal, null, 2) + '\n', {flag:'wx'});
    await createFixture(journal, api, save);
  } else {
    journal = validateJournal(JSON.parse(await readFile(path, 'utf8')));
    assert(journal.team === process.env.VERCEL_TEAM_ID && journal.run === process.env.GITHUB_RUN_ID
      && journal.source === process.env.GITHUB_SHA, 'Journal belongs to another team or Actions run');
    if (action === 'cleanup') await cleanupFixture(journal, api, save);
    else {
      assert(binary, 'Pass the current zel binary');
      await exerciseFixture(journal, api, args => {
        const result = spawnSync(binary, args, {env:{...process.env, NO_COLOR:'1'}, encoding:'utf8',
          maxBuffer:32 * 1024 * 1024, timeout:10 * 60 * 1000});
        process.stdout.write(result.stdout ?? '');
        process.stderr.write(result.stderr ?? '');
        assert.equal(result.status, 0, 'zel vc failed');
        return result.stdout;
      }, waitForHttp, save);
    }
  }
  console.log(`Vercel ${action}: ${journal.session} (${journal.project ?? 'no confirmed project'})`);
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) await main(...process.argv.slice(2));
