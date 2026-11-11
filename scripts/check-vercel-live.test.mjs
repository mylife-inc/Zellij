import {test} from 'node:test';
import assert from 'node:assert/strict';
import {createFixture, cleanupFixture, exerciseFixture, deploymentReceipt, validateJournal, vercelApi} from './check-vercel-live.mjs';

const fixture = () => ({version:1, session:'zellij-live-20261110-abcdef', team:'team_fixture', run:'123',
  source:'a'.repeat(40), creation:'not-started', cases:[]});
const project = () => ({id:'prj_fixture', name:fixture().session, accountId:fixture().team, ssoProtection:null});
const confirmed = () => ({...fixture(), creation:'confirmed', project:project().id});
const discard = async () => {};
const receipt = n => `✓ Vercel deployment dpl_fixture${n} is ready\n  https://fixture-${n}.vercel.app/\n`;

test('creation records only a newly returned identity and requests public static previews', async () => {
  const state = fixture(), saved = [], calls = [];
  await createFixture(state, async (...args) => {
    calls.push(args);
    return args[0] === 'GET' ? null : {...project(), arbitraryProviderData:'not retained'};
  }, async value => saved.push(structuredClone(value)));
  assert.deepEqual(calls, [
    ['GET', `/v9/projects/${state.session}`, undefined, true],
    ['POST', '/v11/projects', {name:state.session, framework:null, ssoProtection:null, publicSource:false}],
  ]);
  assert.equal(saved[0].creation, 'requested');
  assert.equal(saved[0].project, undefined);
  assert.equal(saved[1].project, 'prj_fixture');
  assert.equal(saved[1].creation, 'confirmed');
  assert(!JSON.stringify(saved).includes('arbitraryProviderData'));
});

test('a name collision never changes or deletes an existing project', async () => {
  const state = fixture(), calls = [];
  const api = async (...args) => { calls.push(args); return project(); };
  await assert.rejects(createFixture(state, api, discard), /already exists/);
  await cleanupFixture(state, api, discard);
  assert.equal(calls.length, 1);
  assert.equal(state.cleanup.status, 'not-created');
});

test('an uncertain create outcome is journaled and never deleted by guessed name', async () => {
  const state = fixture(), saved = [], calls = [];
  const api = async (...args) => {
    calls.push(args);
    if (args[0] === 'GET') return null;
    throw new Error('connection closed');
  };
  await assert.rejects(createFixture(state, api, async value => saved.push(structuredClone(value))), /connection closed/);
  await assert.rejects(cleanupFixture(state, api, discard), /outcome is unknown/);
  assert.equal(saved[0].creation, 'requested');
  assert.equal(calls.length, 2);
});

test('a confirmed creation is retained for cleanup even if preview access is protected', async () => {
  const state = fixture();
  await assert.rejects(createFixture(state, async method => method === 'GET' ? null : {...project(), ssoProtection:{deploymentType:'all'}}, discard), /protection remains/);
  assert.equal(state.project, 'prj_fixture');
  assert.equal(state.creation, 'confirmed');
});

test('cleanup deletes by confirmed ID and verifies absence', async () => {
  const state = confirmed(), calls = [], saved = [];
  await cleanupFixture(state, async (...args) => {
    calls.push(args);
    return calls.length === 1 ? project() : null;
  }, async value => saved.push(structuredClone(value)));
  assert.deepEqual(calls.map(call => call.slice(0, 2)), [
    ['GET', '/v9/projects/prj_fixture'], ['DELETE', '/v9/projects/prj_fixture'], ['GET', '/v9/projects/prj_fixture'],
  ]);
  assert.equal(saved[0].cleanup.status, 'requested');
  assert.equal(state.cleanup.status, 'deleted');
  await cleanupFixture(state, async () => null, discard);
  assert.equal(state.cleanup.status, 'already-absent');
});

test('cleanup refuses identity drift and reports a project that survives deletion', async () => {
  for (const change of [{name:'production'}, {accountId:'team_other'}, {id:'prj_other'}]) {
    const calls = [];
    await assert.rejects(cleanupFixture(confirmed(), async (...args) => { calls.push(args); return {...project(), ...change}; }, discard), /changed/);
    assert.equal(calls.length, 1);
  }
  const state = confirmed();
  await assert.rejects(cleanupFixture(state, async () => project(), discard), /still exists/);
  assert.equal(state.cleanup.status, 'requested');
});

test('malformed journals fail before any provider call', async () => {
  for (const change of [{version:2}, {session:'production'}, {team:'../team'}, {source:'branch'}, {run:'123;command'}, {project:'../project'}]) {
    const state = {...confirmed(), ...change};
    assert.throws(() => validateJournal(state));
    await assert.rejects(cleanupFixture(state, () => assert.fail('provider called'), discard));
  }
});

test('exercise uses current zel for both renderers and both preview/production modes', async () => {
  const state = confirmed(), commands = [], checked = [], saved = [];
  await exerciseFixture(state, async () => project(), async args => {
    commands.push(args); return receipt(commands.length);
  }, async (...args) => checked.push(args), async value => saved.push(structuredClone(value)));
  assert.equal(commands.length, 4);
  assert.deepEqual(commands.map(args => args.includes('--prod')), [false, false, true, true]);
  assert.deepEqual(checked.map(([, renderer]) => renderer), ['portable', 'hosted', 'portable', 'hosted']);
  for (const args of commands) {
    assert.equal(args[0], 'vc');
    assert.equal(args[4], 'prj_fixture');
  }
  assert.deepEqual(state.cases.map(item => item.status), ['passed', 'passed', 'passed', 'passed']);
  assert(saved.some(s => s.cases.at(-1)?.deployment === 'dpl_fixture1' && s.cases.at(-1).status === 'started'));
  await assert.rejects(exerciseFixture(state, () => assert.fail('provider called'), discard, discard, discard), /replay/);
});

test('a failed public HTTP check preserves receipts and still permits project cleanup', async () => {
  const state = confirmed();
  let count = 0;
  await assert.rejects(exerciseFixture(state, async () => project(), async () => receipt(++count),
    async () => { if (count === 2) throw new Error('HTTP 401'); }, discard), /HTTP 401/);
  assert.deepEqual(state.cases.map(item => item.status), ['passed', 'failed']);
  assert.equal(state.cases[1].deployment, 'dpl_fixture2');
  let requests = 0;
  await cleanupFixture(state, async () => ++requests === 1 ? project() : null, discard);
  assert.equal(state.cleanup.status, 'deleted');
});

test('receipts require a ready deployment and an unambiguous public fixture URL', () => {
  assert.deepEqual(deploymentReceipt(receipt(1)), {deployment:'dpl_fixture1', url:'https://fixture-1.vercel.app/'});
  for (const url of ['https://production.example/', 'http://fixture.vercel.app/', 'https://user:secret@fixture.vercel.app/',
    'https://fixture.vercel.app:123/', 'https://fixture.vercel.app/?token=secret', 'https://fixture.vercel.app/path/',
    'https://fixture.vercel.app/#anchor', 'https://fixture.vercel.app.attacker.test/']) {
    assert.throws(() => deploymentReceipt(receipt(1).replace('https://fixture-1.vercel.app/', url)));
  }
  assert.throws(() => deploymentReceipt(receipt(1).replace('is ready', 'is building')));
});

test('API requests bind the team, refuse redirects and do not expose provider bodies or transport errors', async () => {
  const requests = [];
  const api = vercelApi('fixture-secret', 'team_fixture', async (url, options) => {
    requests.push({url, options}); return new Response(JSON.stringify(project()));
  });
  await api('GET', '/v9/projects/prj_fixture');
  assert.equal(requests[0].url.href, 'https://api.vercel.com/v9/projects/prj_fixture?teamId=team_fixture');
  assert.equal(requests[0].options.redirect, 'error');
  assert.equal(requests[0].options.headers.Authorization, 'Bearer fixture-secret');
  await assert.rejects(api('GET', 'https://attacker.test/'), /Unexpected/);
  for (const fetcher of [
    async () => new Response('fixture-secret', {status:403}),
    async () => new Response('fixture-secret'),
    async () => new Response(JSON.stringify({error:{message:'fixture-secret'}})),
    async () => new Response('x'.repeat(2 * 1024 * 1024 + 1)),
    async () => { throw new Error('fixture-secret'); },
  ]) {
    await assert.rejects(vercelApi('fixture-secret', 'team_fixture', fetcher)('GET', '/v9/projects/prj_fixture'),
      error => !error.message.includes('fixture-secret'));
  }
  const missing = vercelApi('fixture-secret', 'team_fixture', async () => new Response('', {status:404}));
  assert.equal(await missing('GET', '/v9/projects/prj_fixture', undefined, true), null);
  await assert.rejects(missing('GET', '/v9/projects/prj_fixture'), /HTTP 404/);
});
