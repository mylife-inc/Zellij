// Actions-only acceptance in a freshly created Fly app. The journal contains
// resource identities and results, never tokens or provider response bodies.
import assert from 'node:assert/strict';
import {randomBytes} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {readFile, rename, writeFile} from 'node:fs/promises';
import {isIP} from 'node:net';
import {fileURLToPath} from 'node:url';
import {checkDeliverySite} from './check-delivery-http.mjs';

const sessionPattern = /^zellij-live-\d{8}-[0-9a-f]{6}$/;
const slugPattern = /^[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?$/;
const idPattern = /^[A-Za-z0-9-]{1,128}$/;
const validId = value => typeof value === 'string' && idPattern.test(value);
const bootstrapImage = 'ghcr.io/mylife-inc/zellij/acceptance-static@sha256:eafc8f5cbbde2d269a38152a8e143bfbdcdb2fce63cdd0727247b05878b7069a';
const pause = ms => new Promise(resolve => setTimeout(resolve, ms));

export function validateJournal(journal) {
  assert(journal?.version === 1 && sessionPattern.test(journal.session), 'Expected a disposable Fly journal');
  assert(typeof journal.org === 'string' && slugPattern.test(journal.org) && /^[a-z]{3}$/.test(journal.region), 'Expected an explicit Fly organization and region');
  assert(/^\d+$/.test(journal.run ?? '') && /^[0-9a-f]{40}$/.test(journal.source ?? ''), 'Expected Actions source/run identities');
  assert(['not-started','requested','confirmed'].includes(journal.creation) && Array.isArray(journal.cases), 'Invalid fixture lifecycle');
  if (journal.app !== undefined) assert(validId(journal.app), 'Invalid app ID');
  if (journal.machine !== undefined) assert(validId(journal.machine), 'Invalid Machine ID');
  return journal;
}

export function verifyApp(app, journal) {
  validateJournal(journal);
  assert(app?.id === journal.app && app.name === journal.session && app.organization?.slug === journal.org,
    'Fly app identity or organization changed; refusing to use it');
}

export function tokenParts(value) {
  assert(typeof value === 'string', 'Set FLY_API_TOKEN');
  const raw = value.replace(/^(FlyV1 |Bearer )/, '');
  assert(typeof raw === 'string' && raw.length <= 16384 && /^[\x21-\x7e]+$/.test(raw) && !raw.includes(','), 'Set one FLY_API_TOKEN without whitespace');
  return {raw, authorization:`${/^(fm1r_|fm1a_|fm2_)/.test(raw) ? 'FlyV1' : 'Bearer'} ${raw}`};
}

export function flyApi(token, fetcher = fetch) {
  const {authorization} = tokenParts(token);
  return async (method, path, body, allowMissing = false) => {
    const graph = path === '/graphql';
    const url = new URL(path, graph ? 'https://api.fly.io' : 'https://api.machines.dev');
    assert((graph ? method === 'POST' && url.origin === 'https://api.fly.io'
      : url.origin === 'https://api.machines.dev' && /^\/v1\/apps(?:\/|$)/.test(url.pathname))
      && !url.username && !url.password && !url.hash && (!url.search || url.search === '?force=true'), 'Unexpected Fly API path');
    let response;
    try {
      response = await fetcher(url, {method, redirect:'error', signal:AbortSignal.timeout(30_000),
        headers:{Authorization:authorization, 'Content-Type':'application/json'},
        ...(body === undefined ? {} : {body:JSON.stringify(body)})});
    } catch {
      throw new Error(`Fly ${method} ${path}: request failed; remote outcome may be unknown`);
    }
    if (allowMissing && response.status === 404) return null;
    assert(response.ok, `Fly ${method} ${path}: HTTP ${response.status}; body omitted`);
    // App deletion returns HTTP 202 with no body. Reads and creation receipts
    // remain strict JSON; an empty response never proves a resource identity.
    if (method === 'DELETE' && [202,204].includes(response.status)) { await response.body?.cancel(); return null; }
    try {
      const reader = response.body?.getReader();
      const chunks = [];
      let size = 0;
      for (;;) {
        const {done,value} = reader ? await reader.read() : {done:true};
        if (done) break;
        size += value.length;
        if (size > 2 * 1024 * 1024) { await reader.cancel(); throw new Error(); }
        chunks.push(value);
      }
      if (size === 0 && method === 'POST' && /\/(?:start|stop)$/.test(url.pathname)) return null;
      const data = JSON.parse(Buffer.concat(chunks).toString('utf8'));
      assert(data && typeof data === 'object' && !data.error && !data.errors);
      return data;
    } catch {
      throw new Error(`Fly ${method} ${path}: invalid, oversized or error response; body omitted`);
    }
  };
}

export function machineConfig(journal, image) {
  validateJournal(journal);
  assert(new RegExp(`^registry\\.fly\\.io/${journal.session}@sha256:[0-9a-f]{64}$`).test(image), 'Expected a pinned bootstrap image in this app registry');
  return {
    image,
    env:{PORT:'3000', HOSTNAME:'0.0.0.0'},
    guest:{cpu_kind:'shared',cpus:1,memory_mb:512},
    metadata:{fly_process_group:'app',zellij_acceptance:journal.session},
    restart:{policy:'no'},
    services:[{protocol:'tcp',internal_port:3000,autostop:'off',autostart:false,
      ports:[{port:443,handlers:['tls','http']}],
      checks:[{type:'http',port:3000,method:'GET',path:'/',interval:'10s',timeout:'2s',grace_period:'10s'}]}],
  };
}

export function verifyMachine(machine, journal) {
  assert(machine?.id === journal.machine && machine.name === 'acceptance'
    && machine.region === journal.region && machine.config?.metadata?.zellij_acceptance === journal.session,
  'Fly Machine identity changed; refusing to use it');
}

async function waitForMachine(journal, api, sleep = pause) {
  const path = `/v1/apps/${journal.session}/machines/${journal.machine}`;
  for (let attempt = 0; attempt < 60; attempt++) {
    const machine = await api('GET', path);
    verifyMachine(machine, journal);
    if (machine.state === 'started' && machine.host_status === 'ok' && !machine.cordoned
      && machine.checks?.length && machine.checks.every(check => check.status === 'passing')) return;
    assert(!['destroyed','destroying','failed'].includes(machine.state), 'Fixture Machine failed to start');
    await sleep(2000);
  }
  throw new Error('Fixture Machine did not become healthy within the bounded wait');
}

export async function createFixture(journal, api, prepareImage, save, sleep = pause) {
  validateJournal(journal);
  assert(journal.creation === 'not-started' && !journal.app, 'Refusing to reuse a fixture journal');
  const path = `/v1/apps/${journal.session}`;
  assert.equal(await api('GET', path, undefined, true), null, 'Fixture name already exists; no app was changed');
  journal.creation = 'requested';
  await save(journal);
  const receipt = await api('POST', '/v1/apps', {app_name:journal.session,org_slug:journal.org,network:journal.session});
  assert(validId(receipt?.id), 'Missing creation receipt; inspect the recorded name before manual cleanup');
  journal.app = receipt.id;
  journal.creation = 'confirmed';
  await save(journal); // Preserve the receipt even if setup fails afterwards.
  verifyApp(await api('GET', path), journal);
  const ip = await api('POST', '/graphql', {
    query:'mutation($input: AllocateIPAddressInput!) { allocateIpAddress(input: $input) { app { name sharedIpAddress } } }',
    variables:{input:{appId:journal.session,type:'shared_v4',region:''}},
  });
  const assigned = ip?.data?.allocateIpAddress?.app;
  assert(assigned?.name === journal.session && isIP(assigned.sharedIpAddress) === 4,
    'Shared IPv4 allocation did not confirm the fixture app');
  verifyApp(await api('GET', path), journal);
  journal.sharedIp = assigned.sharedIpAddress;
  await save(journal);
  // Copy the pinned release fixture to this app's registry before starting any
  // trial VM. The Machine never depends on unauthenticated GHCR pull access.
  journal.bootstrapImage = await prepareImage(journal);
  const config = machineConfig(journal, journal.bootstrapImage);
  await save(journal);
  verifyApp(await api('GET', path), journal);
  const machine = await api('POST', `${path}/machines`, {name:'acceptance',region:journal.region,config});
  assert(validId(machine?.id), 'Missing Machine receipt; app cleanup is still available');
  journal.machine = machine.id;
  await save(journal);
  verifyMachine(machine, journal);
  await waitForMachine(journal, api, sleep);
  journal.ready = true;
  await save(journal);
}

export async function cleanupFixture(journal, api, save, sleep = pause) {
  validateJournal(journal);
  if (!journal.app) {
    assert(journal.creation === 'not-started', 'App creation outcome is unknown; inspect the recorded organization/name before manual cleanup');
    journal.cleanup = {status:'not-created'};
    await save(journal);
    return;
  }
  assert(journal.creation === 'confirmed', 'Cleanup requires a confirmed creation receipt');
  const path = `/v1/apps/${journal.session}`;
  const app = await api('GET', path, undefined, true);
  if (app !== null) {
    verifyApp(app, journal);
    journal.cleanup = {status:'requested'};
    await save(journal);
    await api('DELETE', `${path}?force=true`);
    let absent = false;
    for (let attempt = 0; attempt < 30; attempt++) {
      const current = await api('GET', path, undefined, true);
      if (current === null) { absent = true; break; }
      verifyApp(current, journal);
      await sleep(2000);
    }
    assert(absent, 'Fixture app still exists after deletion; manual cleanup is required');
  }
  journal.cleanup = {status:app === null ? 'already-absent' : 'deleted',verifiedAt:new Date().toISOString()};
  await save(journal);
}

export function deploymentReceipt(stdout, journal) {
  assert(stdout.includes(`Fly app ${journal.session}: 1 Machines updated with fresh passing health checks`), 'zel did not report a healthy one-Machine rollout');
  const image = stdout.match(/registry\.fly\.io\/[a-z0-9-]+@sha256:[0-9a-f]{64}/)?.[0];
  assert(image?.startsWith(`registry.fly.io/${journal.session}@sha256:`), 'Unexpected deployed image receipt');
  return image;
}

export async function exerciseFixture(journal, api, zel, check, save, sleep = pause) {
  validateJournal(journal);
  assert(journal.creation === 'confirmed' && journal.ready && journal.machine && !journal.cleanup && journal.cases.length === 0,
    'Expected a fresh, ready fixture');
  for (const [kind,renderer] of [['static','portable'],['next','hosted']]) {
    verifyApp(await api('GET', `/v1/apps/${journal.session}`), journal);
    const path = `/v1/apps/${journal.session}/machines/${journal.machine}`;
    const machine = await api('GET', path);
    verifyMachine(machine, journal);
    // Reset the trial runtime window for each case, including when the Machine
    // is still running. Never restart/retry in the middle of a zel rollout.
    assert(['started','stopped'].includes(machine.state), 'Fixture Machine is not ready for a new case');
    if (machine.state === 'started') {
      await api('POST', `${path}/stop`);
      let stopped = false;
      for (let attempt = 0; attempt < 60; attempt++) {
        const current = await api('GET', path);
        verifyMachine(current,journal);
        if (current.state === 'stopped') { stopped = true; break; }
        await sleep(2000);
      }
      assert(stopped, 'Fixture Machine did not stop before the next test');
    }
    await api('POST', `${path}/start`);
    await waitForMachine(journal, api, sleep);
    const item = {kind,renderer,status:'started',url:`https://${journal.session}.fly.dev/`};
    journal.cases.push(item);
    await save(journal);
    try {
      item.image = deploymentReceipt(await zel(['fly','--config',`scripts/fixtures/release-delivery/${renderer}/${renderer}.yaml`,
        '--site',journal.session,'--kind',kind,'--prod']), journal);
      await save(journal);
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

async function main(action, path, binary) {
  assert(['create','exercise','cleanup'].includes(action) && path, 'Pass create|exercise|cleanup and a journal path');
  assert(process.env.GITHUB_ACTIONS === 'true' && process.env.GITHUB_REPOSITORY === 'mylife-inc/Zellij', 'Run this lifecycle in Zellij GitHub Actions');
  const api = flyApi(process.env.FLY_API_TOKEN);
  const save = async journal => {
    await writeFile(`${path}.tmp`, JSON.stringify(journal,null,2)+'\n');
    await rename(`${path}.tmp`,path);
  };
  let journal;
  if (action === 'create') {
    assert(process.env.ZEL_FLY_TRIAL_CONFIRMED === 'true', 'Confirm an active no-card trial before provisioning');
    journal = validateJournal({version:1,session:`zellij-live-${new Date().toISOString().slice(0,10).replaceAll('-','')}-${randomBytes(3).toString('hex')}`,
      org:process.env.FLY_ORG,region:process.env.FLY_REGION,run:process.env.GITHUB_RUN_ID,source:process.env.GITHUB_SHA,creation:'not-started',cases:[]});
    await writeFile(path,JSON.stringify(journal,null,2)+'\n',{flag:'wx'});
    await createFixture(journal,api,async fixture => {
      const run = args => {
        const result = spawnSync('docker',args,{encoding:'utf8',timeout:5*60*1000,maxBuffer:4*1024*1024});
        assert.equal(result.status,0,`Bootstrap docker ${args[0]} failed; provider output omitted`);
        return result.stdout;
      };
      const tag = `registry.fly.io/${fixture.session}:acceptance-bootstrap`;
      run(['pull','--platform','linux/amd64',bootstrapImage]);
      run(['tag',bootstrapImage,tag]);
      run(['push',tag]);
      const digests = JSON.parse(run(['image','inspect','--format','{{json .RepoDigests}}',tag]));
      assert(Array.isArray(digests), 'Missing bootstrap digest');
      const image = digests.find(value => typeof value === 'string' && value.startsWith(`registry.fly.io/${fixture.session}@sha256:`));
      machineConfig(fixture,image); // Validate before persisting or provisioning.
      return image;
    },save);
  } else {
    journal = validateJournal(JSON.parse(await readFile(path,'utf8')));
    assert(journal.org === process.env.FLY_ORG && journal.run === process.env.GITHUB_RUN_ID && journal.source === process.env.GITHUB_SHA,
      'Journal belongs to another organization or Actions run');
    if (action === 'cleanup') await cleanupFixture(journal,api,save);
    else {
      assert(binary, 'Pass the current zel binary');
      await exerciseFixture(journal,api,args => {
        const result = spawnSync(binary,args,{env:{...process.env,NO_COLOR:'1'},encoding:'utf8',maxBuffer:32*1024*1024,timeout:10*60*1000});
        process.stdout.write(result.stdout ?? ''); process.stderr.write(result.stderr ?? '');
        assert.equal(result.status,0,'zel fly failed');
        return result.stdout;
      },async (url,renderer) => {
        for (let attempt = 0;; attempt++) {
          try { await checkDeliverySite(url,renderer); return; }
          catch (error) { if (attempt === 12) throw error; await pause(5000); }
        }
      },save);
    }
  }
  console.log(`Fly ${action}: ${journal.session} (${journal.app ?? 'no confirmed app'})`);
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) await main(...process.argv.slice(2));
