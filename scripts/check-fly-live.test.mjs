import assert from 'node:assert/strict';
import test from 'node:test';
import {cleanupFixture,createFixture,deploymentReceipt,exerciseFixture,flyApi,machineConfig,tokenParts,validateJournal,verifyApp,verifyMachine} from './check-fly-live.mjs';

function journal() {
  return {version:1,session:'zellij-live-20261111-123abc',org:'trial-org',region:'iad',run:'123',source:'a'.repeat(40),creation:'not-started',cases:[]};
}
function fixture() {
  const j = journal(), calls = [], saves = [];
  const image = `registry.fly.io/${j.session}@sha256:${'b'.repeat(64)}`;
  const state = {app:null,machine:null};
  const save = async value => saves.push(structuredClone(value));
  const api = async (method,path,body,allowMissing) => {
    calls.push({method,path,body,allowMissing});
    const appPath = `/v1/apps/${j.session}`;
    if (method === 'POST' && path === '/v1/apps') {
      assert.deepEqual(body,{app_name:j.session,org_slug:j.org,network:j.session});
      state.app = {id:'created-app-id',name:j.session,organization:{slug:j.org}};
      return {id:state.app.id};
    }
    if (method === 'GET' && path === appPath) return state.app;
    if (method === 'POST' && path === '/graphql') {
      assert.deepEqual(body.variables.input,{appId:j.session,type:'shared_v4',region:''});
      return {data:{allocateIpAddress:{app:{name:j.session,sharedIpAddress:'192.0.2.1'}}}};
    }
    if (method === 'POST' && path === `${appPath}/machines`) {
      assert.equal(body.name,'acceptance');
      assert.equal(body.region,j.region);
      assert.equal(body.config.image,image);
      assert.deepEqual(body.config.guest,{cpu_kind:'shared',cpus:1,memory_mb:512});
      assert(!body.config.mounts && !body.config.files && !body.config.init);
      state.machine = {id:'machine-1',name:body.name,region:body.region,config:body.config,state:'started',host_status:'ok',checks:[{status:'passing'}]};
      return state.machine;
    }
    if (method === 'GET' && path === `${appPath}/machines/machine-1`) return state.machine;
    if (method === 'POST' && path === `${appPath}/machines/machine-1/stop`) { state.machine.state = 'stopped'; return null; }
    if (method === 'POST' && path === `${appPath}/machines/machine-1/start`) { state.machine.state = 'started'; return null; }
    if (method === 'DELETE' && path === `${appPath}?force=true`) { state.app = null; state.machine = null; return null; }
    assert.fail(`Unexpected ${method} ${path}`);
  };
  return {j,calls,saves,state,api,save,image,prepare:async () => image,sleep:async () => {}};
}
async function create(f) { await createFixture(f.j,f.api,f.prepare,f.save,f.sleep); }
function receipt(f) { return `✓ Fly app ${f.j.session}: 1 Machines updated with fresh passing health checks\n  ${f.image}\n`; }

test('fresh fixture creation journals ownership, allocates shared IPv4 and bounds one isolated Machine', async () => {
  const f = fixture();
  await create(f);
  assert.equal(f.j.ready,true);
  assert.equal(f.saves[0].creation,'requested');
  assert.equal(f.saves[0].app,undefined);
  assert.equal(f.saves[1].app,'created-app-id');
  assert.equal(f.j.bootstrapImage,f.image);
  assert.equal(f.calls.filter(c => c.path.endsWith('/machines') && c.method === 'POST').length,1);
  await cleanupFixture(f.j,f.api,f.save,f.sleep);
  assert.equal(f.j.cleanup.status,'deleted');
  assert.equal(f.calls.at(-1).method,'GET');
  await cleanupFixture(f.j,f.api,f.save,f.sleep);
  assert.equal(f.j.cleanup.status,'already-absent');
});

test('name collision and ambiguous app creation never authorize deletion by name', async () => {
  const f = fixture();
  f.state.app = {id:'existing-user-app',name:f.j.session,organization:{slug:f.j.org}};
  await assert.rejects(create(f),/already exists/);
  assert(f.calls.every(c => c.method === 'GET'));
  await cleanupFixture(f.j,f.api,f.save,f.sleep);
  assert.equal(f.j.cleanup.status,'not-created');
  const g = fixture();
  const api = async (...args) => {
    const result = await g.api(...args);
    if (args[0] === 'POST' && args[1] === '/v1/apps') throw new Error('unknown outcome');
    return result;
  };
  await assert.rejects(createFixture(g.j,api,g.prepare,g.save,g.sleep),/unknown outcome/);
  await assert.rejects(cleanupFixture(g.j,api,g.save,g.sleep),/outcome is unknown/);
  assert(!g.calls.some(c => c.method === 'DELETE'));
});

test('setup failures after a confirmed app receipt retain enough evidence for cleanup', async () => {
  for (const failPath of ['/graphql','bootstrap','/machines']) {
    const f = fixture();
    const api = async (...args) => {
      if (args[0] === 'POST' && args[1].endsWith(failPath)) throw new Error('setup failed');
      return f.api(...args);
    };
    const prepare = failPath === 'bootstrap' ? async () => { throw new Error('setup failed'); } : f.prepare;
    await assert.rejects(createFixture(f.j,api,prepare,f.save,f.sleep),/setup failed/);
    assert.equal(f.j.creation,'confirmed');
    await cleanupFixture(f.j,f.api,f.save,f.sleep);
    assert.equal(f.j.cleanup.status,'deleted');
  }
});

test('malformed creation receipts never become ownership evidence', async () => {
  for (const id of [undefined,null,123,'../other']) {
    const f = fixture();
    const api = async (...args) => args[0] === 'POST' && args[1] === '/v1/apps' ? {id} : f.api(...args);
    await assert.rejects(createFixture(f.j,api,f.prepare,f.save,f.sleep),/Missing creation receipt/);
    await assert.rejects(cleanupFixture(f.j,api,f.save,f.sleep),/outcome is unknown/);
  }
});

test('identity drift blocks use and deletion; deletion must be confirmed by API absence', async () => {
  for (const change of [app => app.id = 'different', app => app.name = 'user-app', app => app.organization.slug = 'other']) {
    const f = fixture(); await create(f); change(f.state.app);
    await assert.rejects(cleanupFixture(f.j,f.api,f.save,f.sleep),/identity or organization changed/);
    assert(!f.calls.some(c => c.method === 'DELETE'));
  }
  const f = fixture(); await create(f);
  const api = async (...args) => args[0] === 'DELETE' ? null : f.api(...args);
  await assert.rejects(cleanupFixture(f.j,api,f.save,f.sleep),/still exists/);
  assert.equal(f.j.cleanup.status,'requested');
});

test('both real CLI forms are exercised with fresh trial windows and recorded HTTP acceptance', async () => {
  const f = fixture(); await create(f);
  const commands = [], checks = [];
  await exerciseFixture(f.j,f.api,async args => {commands.push(args); return receipt(f);},async (...args) => checks.push(args),f.save,f.sleep);
  assert.deepEqual(commands.map(args => [args[0],args[2],args[6],args[7]]),[
    ['fly','scripts/fixtures/release-delivery/portable/portable.yaml','static','--prod'],
    ['fly','scripts/fixtures/release-delivery/hosted/hosted.yaml','next','--prod'],
  ]);
  assert.deepEqual(checks.map(c => c[1]),['portable','hosted']);
  assert(f.j.cases.every(c => c.status === 'passed' && c.image === f.image));
  assert.equal(f.calls.filter(c => c.path.endsWith('/stop')).length,2);
  assert.equal(f.calls.filter(c => c.path.endsWith('/start')).length,2);
  await assert.rejects(exerciseFixture(f.j,f.api,async () => {},async () => {},f.save,f.sleep),/fresh, ready/);
});

test('failed CLI or public HTTP checks remain failures and permit independent cleanup', async () => {
  for (const failAt of ['cli','http','receipt']) {
    const f = fixture(); await create(f);
    const zel = async () => { if (failAt === 'cli') throw new Error('CLI failed'); return failAt === 'receipt' ? 'wrong receipt' : receipt(f); };
    const check = async () => { if (failAt === 'http') throw new Error('HTTP 401'); };
    await assert.rejects(exerciseFixture(f.j,f.api,zel,check,f.save,f.sleep));
    assert.equal(f.j.cases.length,1);
    assert.equal(f.j.cases[0].status,'failed');
    await cleanupFixture(f.j,f.api,f.save,f.sleep);
    assert.equal(f.j.cleanup.status,'deleted');
  }
});

test('Machine drift and unavailable health never trigger a deployment', async () => {
  const f = fixture(); await create(f);
  f.state.machine.config.metadata.zellij_acceptance = 'other';
  let invoked = false;
  await assert.rejects(exerciseFixture(f.j,f.api,async () => {invoked = true;},async () => {},f.save,f.sleep),/Machine identity changed/);
  assert(!invoked);
  const g = fixture();
  const api = async (...args) => {
    const value = await g.api(...args);
    if (value?.checks) value.checks[0].status = 'critical';
    return value;
  };
  await assert.rejects(createFixture(g.j,api,g.prepare,g.save,g.sleep),/did not become healthy/);
  await cleanupFixture(g.j,g.api,g.save,g.sleep);
  assert.equal(g.j.cleanup.status,'deleted');
});

test('journal, credential, image and receipt validation reject other resources', () => {
  for (const field of ['session','org','region','run','source']) {
    assert.throws(() => validateJournal({...journal(),[field]:undefined}));
    assert.throws(() => validateJournal({...journal(),[field]:'../outside'}));
  }
  for (const token of [undefined,'','token token','one,two','FlyV1 ']) assert.throws(() => tokenParts(token));
  assert.equal(tokenParts('Bearer fm2_example').authorization,'FlyV1 fm2_example');
  assert.equal(tokenParts('legacy-token').authorization,'Bearer legacy-token');
  const f = fixture();
  assert.throws(() => machineConfig(f.j,'registry.fly.io/user-app@sha256:'+'a'.repeat(64)));
  assert.throws(() => deploymentReceipt('wrong app',f.j));
  assert.throws(() => verifyApp({id:'other'},f.j));
  assert.throws(() => verifyMachine({id:'other'},f.j));
});

test('API uses bounded credential-safe requests and accepts only specific empty acknowledgements', async () => {
  const requests = [];
  const api = flyApi('fm2_secret',async (url,options) => {
    requests.push({url,options});
    return new Response(options.method === 'GET' ? '{}' : null,{status:options.method === 'DELETE' ? 202 : 200});
  });
  assert.deepEqual(await api('GET','/v1/apps/name'),{});
  assert.equal(await api('DELETE','/v1/apps/name?force=true'),null);
  assert.equal(await api('POST','/v1/apps/name/machines/id/start'),null);
  await assert.rejects(api('POST','/v1/apps'),/invalid/);
  assert.equal(requests[0].options.headers.Authorization,'FlyV1 fm2_secret');
  assert.equal(requests[0].options.redirect,'error');
  assert(requests[0].options.signal instanceof AbortSignal);
  for (const path of ['https://other.test/v1/apps','/other','/v1/apps?token=secret','/v1/apps#fragment']) await assert.rejects(api('GET',path),/Unexpected/);
  for (const response of [new Response('SECRET',{status:403}),new Response('SECRET',{status:200}),new Response('{"errors":["SECRET"]}',{status:200}),new Response('x'.repeat(2*1024*1024+1),{status:200})]) {
    const bad = flyApi('fm2_secret',async () => response);
    await assert.rejects(bad('GET','/v1/apps/name'),error => !error.message.includes('SECRET') && !error.message.includes('fm2_secret'));
  }
  const missing = flyApi('token',async () => new Response('SECRET',{status:404}));
  assert.equal(await missing('GET','/v1/apps/name',undefined,true),null);
  const broken = flyApi('token',async () => { throw new Error('SECRET'); });
  await assert.rejects(broken('POST','/v1/apps'),error => /unknown/.test(error.message) && !error.message.includes('SECRET'));
});
