import assert from 'node:assert/strict';
import test from 'node:test';
import {awsApi,awsEnvironment,cleanupFixture,createFixture,exerciseFixture,validateJournal,websiteUrl} from './check-s3-live.mjs';

function fixture() {
  const j = {version:1,bucket:'zellij-live-20261111-123abc456def',account:'123456789012',region:'us-east-2',
    run:'123',source:'a'.repeat(40),creation:'not-started',cases:[]};
  const calls = [], saves = [], state = {exists:false,tags:[],objects:[],block:null,versioning:{}};
  const save = async value => saves.push(structuredClone(value));
  const api = async (operation,input,missing) => {
    calls.push({operation,input,missing});
    if (input?.Bucket && operation !== 'create-bucket') assert.equal(input.ExpectedBucketOwner,j.account);
    if (operation === 'get-caller-identity') return {Account:j.account};
    if (operation === 'get-public-access-block') return state.block;
    if (operation === 'head-bucket') return state.exists ? {BucketRegion:j.region} : null;
    if (operation === 'create-bucket') { state.exists = true; return {Location:`/${j.bucket}`}; }
    if (operation === 'get-bucket-location') return {LocationConstraint:j.region};
    if (operation === 'get-bucket-versioning') return state.versioning;
    if (operation === 'put-bucket-tagging') { state.tags = input.Tagging.TagSet; return {}; }
    if (operation === 'get-bucket-tagging') return {TagSet:state.tags};
    if (operation.startsWith('put-')) return {};
    if (operation === 'list-objects-v2') return {IsTruncated:false,KeyCount:state.objects.length,Contents:state.objects};
    if (operation === 'delete-objects') {
      const keys = input.Delete.Objects;
      state.objects = state.objects.filter(object => !keys.some(key => key.Key === object.Key));
      return {Deleted:keys};
    }
    if (operation === 'delete-bucket') { state.exists = false; return {}; }
    throw new Error('Unexpected operation '+operation);
  };
  return {j,calls,saves,state,api,save,sleep:async()=>{}};
}
const create = f => createFixture(f.j,f.api,f.save);
const cleanup = f => cleanupFixture(f.j,f.api,f.save,f.sleep);
const receipt = (f,renderer) => `✓ Uploaded 3 S3 objects\n  ${websiteUrl(f.j,renderer)}\n`;
const credentials = {AWS_ACCESS_KEY_ID:'AKIA'+'A'.repeat(16),AWS_SECRET_ACCESS_KEY:'secret-never-logged'};

test('new bucket setup isolates public reads and preserves creation identity before mutable setup', async()=>{
  const f = fixture(); await create(f);
  assert(f.j.ready && f.j.tagged);
  assert.deepEqual(f.saves.slice(0,2).map(j=>j.creation),['requested','confirmed']);
  assert.deepEqual(f.calls.find(c=>c.operation === 'create-bucket').input,
    {Bucket:f.j.bucket,CreateBucketConfiguration:{LocationConstraint:'us-east-2'},ObjectOwnership:'BucketOwnerEnforced'});
  const access = f.calls.find(c=>c.operation === 'put-public-access-block').input;
  assert.equal(access.PublicAccessBlockConfiguration.BlockPublicAcls,true);
  const policy = JSON.parse(f.calls.find(c=>c.operation === 'put-bucket-policy').input.Policy);
  assert.deepEqual(policy.Statement[0].Resource,['portable','hosted'].map(prefix=>`arn:aws:s3:::${f.j.bucket}/${prefix}/*`));
  assert.equal(policy.Statement[0].Action,'s3:GetObject');
  assert(f.calls.filter(c=>c.operation.startsWith('put-')).every(c=>c.input.Bucket === f.j.bucket));
  f.state.objects = [{Key:'portable/index.html',Size:100},{Key:'hosted/manual',Size:200}];
  await cleanup(f); assert.equal(f.j.cleanup.status,'deleted'); assert.equal(f.j.cleanup.objects,2);
  assert.equal(f.calls.at(-1).operation,'head-bucket');
  await cleanup(f); assert.equal(f.j.cleanup.status,'already-absent');
});

test('account mismatch, account policy and name collisions prevent creation', async()=>{
  const f = fixture(); f.state.exists = true;
  await assert.rejects(create(f),/already exists/);
  await cleanup(f); assert.equal(f.j.cleanup.status,'not-created');
  assert(!f.calls.some(c=>c.operation === 'delete-bucket' || c.operation === 'create-bucket'));
  for (const flag of ['BlockPublicPolicy','RestrictPublicBuckets']) {
    const g = fixture(); g.state.block = {PublicAccessBlockConfiguration:{BlockPublicPolicy:false,RestrictPublicBuckets:false,[flag]:true}};
    await assert.rejects(create(g),/Account public-access policy/);
    assert(!g.calls.some(c=>c.operation === 'create-bucket'));
  }
  const g = fixture();
  await assert.rejects(createFixture(g.j,async()=>({Account:'999999999999'}),g.save),/different account/);
});

test('ambiguous and malformed creation receipts cannot authorize deletion', async()=>{
  for (const malformed of [false,true]) {
    const f = fixture(); const api = async(...args)=>{
      const result = await f.api(...args);
      if (args[0] === 'create-bucket') { if (malformed) return {}; throw new Error('network lost'); }
      return result;
    };
    await assert.rejects(createFixture(f.j,api,f.save)); assert.equal(f.j.creation,'requested');
    await assert.rejects(cleanup(f),/outcome is unknown/);
    assert(!f.calls.some(c=>c.operation === 'delete-bucket'));
  }
});

test('partial setup permits removal of confirmed empty bucket, but never untagged content', async()=>{
  for (const failure of ['put-bucket-tagging','put-public-access-block','put-bucket-policy','put-bucket-website']) {
    const f = fixture(); const api = async(...args)=>{
      if (args[0] === failure) throw new Error('setup failed');
      return f.api(...args);
    };
    await assert.rejects(createFixture(f.j,api,f.save),/setup failed/);
    assert.equal(f.j.creation,'confirmed'); await cleanup(f); assert.equal(f.j.cleanup.status,'deleted');
  }
  const f = fixture(); await create(f); f.j.tagged = false;
  f.state.objects = [{Key:'portable/index.html',Size:1}];
  await assert.rejects(cleanup(f),/Unexpected objects/);
  assert(!f.calls.some(c=>c.operation.startsWith('delete-')));
});

test('identity drift, versioning and unexpected content block destructive cleanup', async()=>{
  for (const change of [f=>{f.state.tags=[];},f=>{f.state.versioning={Status:'Enabled'};},
    f=>{f.state.objects=[{Key:'user-file.txt',Size:1}];},f=>{f.state.objects=[{Key:'portable/file',Size:26*1024*1024}];}]) {
    const f = fixture(); await create(f); change(f);
    await assert.rejects(cleanup(f)); assert(!f.calls.some(c=>c.operation.startsWith('delete-')));
  }
});

test('cleanup validates bounded pagination before deleting and verifies per-object receipts', async()=>{
  const f = fixture(); await create(f); let page=0;
  const api = async(...args)=>{
    if (args[0] === 'list-objects-v2' && page++ < 2) return page === 1
      ? {IsTruncated:true,Contents:[{Key:'portable/a',Size:1}],NextContinuationToken:'next'}
      : {IsTruncated:false,Contents:[{Key:'hosted/b',Size:1}]};
    return f.api(...args);
  };
  await cleanupFixture(f.j,api,f.save,f.sleep); assert.equal(f.j.cleanup.objects,2);
  for (const response of [{Errors:[{Key:'portable/a',Code:'AccessDenied'}]}, {Deleted:[]}, {Deleted:[{Key:'different'}]}]) {
    const g=fixture(); await create(g); g.state.objects=[{Key:'portable/a',Size:1}];
    await assert.rejects(cleanupFixture(g.j,async(...args)=>args[0] === 'delete-objects' ? response : g.api(...args),g.save,g.sleep),/deletion incomplete/);
    assert(!g.calls.some(c=>c.operation === 'delete-bucket'));
  }
  const g=fixture(); await create(g);
  await assert.rejects(cleanupFixture(g.j,async(...args)=>args[0] === 'list-objects-v2'
    ? {IsTruncated:true,Contents:[],NextContinuationToken:'same'} : g.api(...args),g.save,g.sleep),/continuation/);
  assert(!g.calls.some(c=>c.operation.startsWith('delete-')));
});

test('cleanup does not report success until the bucket disappears', async()=>{
  const f=fixture(); await create(f);
  await assert.rejects(cleanupFixture(f.j,async(...args)=>args[0] === 'delete-bucket' ? {} : f.api(...args),f.save,f.sleep),/not confirmed/);
  assert.equal(f.j.cleanup.status,'requested');
});

test('real CLI plan covers portable and hosted initial plus replacement uploads with HTTP evidence', async()=>{
  const f=fixture(); await create(f); const commands=[],checks=[];
  await exerciseFixture(f.j,f.api,async args=>{commands.push(args); return receipt(f,args[2].includes('/portable/') ? 'portable' : 'hosted');},
    async(...args)=>checks.push(args),f.save);
  assert.deepEqual(f.j.cases.map(c=>[c.renderer,c.upload,c.status]),[
    ['portable','create','passed'],['portable','replace','passed'],['hosted','create','passed'],['hosted','replace','passed']]);
  assert.equal(commands.length,4); assert.equal(checks.length,4);
  assert(commands.every(args=>args[0] === 's3' && args.at(-1) === '--prod' && args[4].startsWith(f.j.bucket+'/')));
  await assert.rejects(exerciseFixture(f.j,f.api,async()=>{},async()=>{},f.save),/fresh ready/);
});

test('CLI, receipt and HTTP failures retain failed evidence and still allow cleanup', async()=>{
  for (const failure of ['cli','receipt','http']) {
    const f=fixture(); await create(f);
    await assert.rejects(exerciseFixture(f.j,f.api,async()=>{
      if (failure === 'cli') throw new Error('CLI failed');
      return failure === 'receipt' ? 'wrong destination' : receipt(f,'portable');
    },async()=>{if(failure === 'http') throw new Error('HTTP failed');},f.save));
    assert.equal(f.j.cases[0].status,'failed'); await cleanup(f); assert.equal(f.j.cleanup.status,'deleted');
  }
});

test('AWS wrapper isolates env credentials, bounds execution and omits provider output', async()=>{
  const clean = awsEnvironment({...credentials,AWS_PROFILE:'unexpected',AWS_ENDPOINT_URL:'https://bad.example',AWS_ROLE_ARN:'unwanted',ZELLIJ_IMAGE:'test'});
  assert.equal(clean.AWS_PROFILE,undefined); assert.equal(clean.AWS_ENDPOINT_URL,undefined); assert.equal(clean.AWS_ROLE_ARN,undefined);
  assert.equal(clean.AWS_MAX_ATTEMPTS,'1'); assert.equal(clean.AWS_CONFIG_FILE,'/dev/null'); assert.equal(clean.ZELLIJ_IMAGE,'test');
  assert.throws(()=>awsEnvironment({...credentials,AWS_ACCESS_KEY_ID:'ASIA'+'A'.repeat(16)}),/session|SESSION/);
  const api=awsApi(credentials,(command,args,options)=>{
    assert.equal(command,'aws'); assert(!args.join(' ').includes(credentials.AWS_SECRET_ACCESS_KEY));
    assert.equal(options.timeout,60000); assert.equal(options.env.AWS_EC2_METADATA_DISABLED,'true');
    return {status:0,stdout:'{"Account":"123456789012"}'};
  });
  assert.equal((await api('get-caller-identity')).Account,'123456789012');
  for (const code of ['404','403']) {
    const fail=awsApi(credentials,()=>({status:254,stderr:`An error occurred (${code}) when calling the HeadBucket operation: SECRET_BODY`}));
    if (code === '404') assert.equal(await fail('head-bucket',{},'404'),null);
    else await assert.rejects(fail('head-bucket',{},'404'),error=>error.message.includes('403') && !error.message.includes('SECRET_BODY'));
  }
  const fail=awsApi(credentials,()=>({status:0,stdout:'SECRET_BODY'}));
  await assert.rejects(fail('create-bucket'),error=>!error.message.includes('SECRET_BODY'));
  const empty=awsApi(credentials,()=>({status:0,stdout:''}));
  assert.deepEqual(await empty('head-bucket'),{}); assert.deepEqual(await empty('delete-bucket'),{});
  await assert.rejects(empty('create-bucket'),/invalid response/);
  await assert.rejects(api('delete-account'),/Unexpected/);
  for (const value of [{region:'us-east-1'},{account:'bad'},{bucket:'existing-site'},{source:''}]) {
    assert.throws(()=>validateJournal({...fixture().j,...value}));
  }
});
