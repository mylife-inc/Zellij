// Disposable acceptance fixture only. The product adapter does not require AWS CLI.
import assert from 'node:assert/strict';
import {randomBytes} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {readFile,rename,writeFile} from 'node:fs/promises';
import {fileURLToPath} from 'node:url';
import {checkDeliverySite} from './check-delivery-http.mjs';

const pause = ms => new Promise(resolve => setTimeout(resolve,ms));
const operations = new Set(['get-caller-identity','get-public-access-block','head-bucket','create-bucket',
  'get-bucket-location','put-bucket-tagging','get-bucket-tagging','get-bucket-versioning',
  'put-public-access-block','put-bucket-policy','put-bucket-website','list-objects-v2','delete-objects','delete-bucket']);
const emptyOperations = new Set(['head-bucket','put-bucket-tagging','put-public-access-block','put-bucket-policy','put-bucket-website','delete-bucket']);

export function validateJournal(j) {
  assert(j?.version === 1 && /^zellij-live-\d{8}-[0-9a-f]{12}$/.test(j.bucket ?? ''), 'Expected a disposable S3 bucket journal');
  // Avoid us-east-1's legacy CreateBucket success/ACL reset for an existing owned bucket.
  assert(j.region === 'us-east-2' && /^\d{12}$/.test(j.account ?? ''), 'Expected us-east-2 and an explicit AWS account ID');
  assert(/^\d+$/.test(j.run ?? '') && /^[0-9a-f]{40}$/.test(j.source ?? ''), 'Expected Actions source/run identities');
  assert(['not-started','requested','confirmed'].includes(j.creation) && Array.isArray(j.cases), 'Invalid fixture lifecycle');
  return j;
}

export function awsEnvironment(env) {
  assert(typeof env.AWS_ACCESS_KEY_ID === 'string' && /^[A-Z0-9]{16,128}$/.test(env.AWS_ACCESS_KEY_ID), 'Set AWS_ACCESS_KEY_ID');
  assert(typeof env.AWS_SECRET_ACCESS_KEY === 'string' && /^[\x21-\x7e]{1,4096}$/.test(env.AWS_SECRET_ACCESS_KEY), 'Set AWS_SECRET_ACCESS_KEY');
  const session = env.AWS_SESSION_TOKEN || undefined;
  assert(!session || /^[\x21-\x7e]{1,16384}$/.test(session), 'Invalid AWS_SESSION_TOKEN');
  assert(!env.AWS_ACCESS_KEY_ID.startsWith('ASIA') || session, 'Temporary access keys require AWS_SESSION_TOKEN');
  // No profiles, alternate endpoints, inherited role/web-identity credentials or metadata fallback.
  return {...Object.fromEntries(Object.entries(env).filter(([key]) => !key.startsWith('AWS_'))),
    AWS_ACCESS_KEY_ID:env.AWS_ACCESS_KEY_ID,AWS_SECRET_ACCESS_KEY:env.AWS_SECRET_ACCESS_KEY,
    ...(session ? {AWS_SESSION_TOKEN:session} : {}),AWS_REGION:'us-east-2',AWS_DEFAULT_REGION:'us-east-2',
    AWS_CONFIG_FILE:'/dev/null',AWS_SHARED_CREDENTIALS_FILE:'/dev/null',AWS_EC2_METADATA_DISABLED:'true',
    AWS_MAX_ATTEMPTS:'1',AWS_PAGER:'',AWS_CLI_AUTO_PROMPT:'off'};
}

export function awsApi(env, execute = spawnSync) {
  const clean = awsEnvironment(env);
  return async (operation,input = {},missingCode) => {
    assert(operations.has(operation), 'Unexpected fixture operation');
    const service = operation === 'get-caller-identity' ? 'sts'
      : operation === 'get-public-access-block' ? 's3control' : 's3api';
    const result = execute('aws',[service,operation,'--cli-input-json',JSON.stringify(input),
      '--region','us-east-2','--output','json','--no-cli-pager','--no-paginate',
      '--cli-connect-timeout','15','--cli-read-timeout','30'],
    {env:clean,encoding:'utf8',timeout:60_000,maxBuffer:2*1024*1024});
    if (result.error || result.status !== 0) {
      const match = result.stderr?.match(/^An error occurred \(([A-Za-z0-9]+)\) when calling the ([A-Za-z0-9]+) operation:/m);
      const expectedOperation = operation.split('-').map(word => word[0].toUpperCase()+word.slice(1)).join('');
      if (!result.error && result.status !== 0 && match?.[2] === expectedOperation && match[1] === missingCode) return null;
      throw new Error(`AWS ${operation} failed${match?.[2] === expectedOperation ? ` (${match[1]})` : ''}; provider output omitted`);
    }
    if (!result.stdout?.trim() && emptyOperations.has(operation)) return {};
    try {
      const data = JSON.parse(result.stdout);
      assert(data && typeof data === 'object' && !Array.isArray(data));
      return data;
    } catch { throw new Error(`AWS ${operation}: invalid response; provider output omitted`); }
  };
}

const bucketArgs = j => ({Bucket:j.bucket,ExpectedBucketOwner:j.account});
const tags = j => [{Key:'zellij-acceptance',Value:j.bucket},{Key:'actions-run',Value:j.run},{Key:'source',Value:j.source}];
async function identity(j,api) {
  validateJournal(j);
  const who = await api('get-caller-identity');
  assert(who.Account === j.account, 'AWS credentials belong to a different account');
}
async function verifyBucket(j,api,{tagged = true} = {}) {
  const location = await api('get-bucket-location',bucketArgs(j));
  assert(location.LocationConstraint === j.region, 'Bucket region changed');
  const versioning = await api('get-bucket-versioning',bucketArgs(j));
  assert(versioning.Status === undefined && versioning.MFADelete === undefined, 'Fixture bucket versioning changed; manual inspection required');
  if (tagged) {
    const actual = await api('get-bucket-tagging',bucketArgs(j));
    assert(Array.isArray(actual.TagSet) && tags(j).every(tag => actual.TagSet.some(item => item.Key === tag.Key && item.Value === tag.Value)),
      'Bucket ownership tags changed; refusing mutation');
  }
}

export async function createFixture(j,api,save) {
  validateJournal(j);
  assert(j.creation === 'not-started', 'Refusing to reuse a fixture journal');
  await identity(j,api);
  const block = await api('get-public-access-block',{AccountId:j.account},'NoSuchPublicAccessBlockConfiguration');
  assert(block === null || (block.PublicAccessBlockConfiguration?.BlockPublicPolicy === false
    && block.PublicAccessBlockConfiguration?.RestrictPublicBuckets === false),
  'Account public-access policy prevents a website; no account-wide settings were changed');
  assert(await api('head-bucket',bucketArgs(j),'404') === null, 'Bucket name already exists; no bucket was changed');
  j.creation = 'requested';
  await save(j);
  const receipt = await api('create-bucket',{Bucket:j.bucket,CreateBucketConfiguration:{LocationConstraint:j.region},ObjectOwnership:'BucketOwnerEnforced'});
  const locations = [`/${j.bucket}`,...['http:','https:'].flatMap(protocol =>
    ['s3.amazonaws.com','s3.us-east-2.amazonaws.com','s3-us-east-2.amazonaws.com'].map(host => `${protocol}//${j.bucket}.${host}/`))];
  assert(locations.includes(receipt.Location), 'Missing bucket creation receipt; inspect the recorded name before manual cleanup');
  j.creation = 'confirmed';
  await save(j); // Receipt remains available if tagging or website setup fails.
  await verifyBucket(j,api,{tagged:false});
  await api('put-bucket-tagging',{...bucketArgs(j),Tagging:{TagSet:tags(j)}});
  j.tagged = true;
  await save(j);
  await verifyBucket(j,api);
  await api('put-public-access-block',{...bucketArgs(j),PublicAccessBlockConfiguration:{BlockPublicAcls:true,IgnorePublicAcls:true,BlockPublicPolicy:false,RestrictPublicBuckets:false}});
  const policy = {Version:'2012-10-17',Statement:[{Effect:'Allow',Principal:'*',Action:'s3:GetObject',
    Resource:['portable','hosted'].map(prefix => `arn:aws:s3:::${j.bucket}/${prefix}/*`)}]};
  await api('put-bucket-policy',{...bucketArgs(j),Policy:JSON.stringify(policy)});
  await api('put-bucket-website',{...bucketArgs(j),WebsiteConfiguration:{IndexDocument:{Suffix:'index.html'}}});
  j.ready = true;
  await save(j);
}

export function websiteUrl(j,renderer) {
  validateJournal(j);
  assert(['portable','hosted'].includes(renderer), 'Invalid renderer');
  return `http://${j.bucket}.s3-website.us-east-2.amazonaws.com/${renderer}/`;
}

export async function exerciseFixture(j,api,zel,check,save) {
  validateJournal(j);
  assert(j.creation === 'confirmed' && j.ready && j.tagged && !j.cleanup && j.cases.length === 0, 'Expected a fresh ready fixture');
  await identity(j,api);
  for (const renderer of ['portable','hosted']) {
    // The second upload exercises real ETags and If-Match, as well as first-upload If-None-Match.
    for (const upload of ['create','replace']) {
      await verifyBucket(j,api);
      const item = {renderer,upload,url:websiteUrl(j,renderer),status:'running'};
      j.cases.push(item); await save(j);
      try {
        const stdout = await zel(['s3','--config',`scripts/fixtures/release-delivery/${renderer}/${renderer}.yaml`,
          '--site',`${j.bucket}/${renderer}`,'--prod']);
        const count = stdout.match(/Uploaded (\d+) S3 objects/)?.[1];
        assert(count && Number(count) > 0 && Number(count) <= 5000 && stdout.includes(item.url), 'Missing expected S3 upload receipt');
        item.objects = Number(count); await save(j);
        await check(item.url,renderer);
        item.status = 'passed';
      } catch (error) { item.status = 'failed'; await save(j); throw error; }
      await save(j);
    }
  }
}

export async function cleanupFixture(j,api,save,sleep = pause) {
  validateJournal(j);
  if (j.creation === 'not-started') { j.cleanup = {status:'not-created'}; await save(j); return; }
  assert(j.creation === 'confirmed', 'Bucket creation outcome is unknown; inspect the recorded name/account before manual cleanup');
  await identity(j,api);
  if (await api('head-bucket',bucketArgs(j),'404') === null) {
    j.cleanup = {status:'already-absent',verifiedAt:new Date().toISOString()}; await save(j); return;
  }
  await verifyBucket(j,api,{tagged:Boolean(j.tagged)});
  // Gather/validate every page before deleting anything. Only our two fixture prefixes are eligible.
  const objects = []; let continuation, bytes = 0;
  for (let page = 0; page < 10; page++) {
    const result = await api('list-objects-v2',{...bucketArgs(j),MaxKeys:1000,...(continuation ? {ContinuationToken:continuation} : {})});
    assert(typeof result.IsTruncated === 'boolean' && Array.isArray(result.Contents ?? []), 'Invalid object inventory');
    for (const object of result.Contents ?? []) {
      assert(j.tagged && typeof object.Key === 'string' && /^(portable|hosted)\/.+/.test(object.Key)
        && Number.isSafeInteger(object.Size) && object.Size >= 0 && object.Size <= 25*1024*1024,
      'Unexpected objects in fixture bucket; refusing deletion');
      objects.push({Key:object.Key}); bytes += object.Size;
    }
    assert(objects.length <= 10000 && bytes <= 200*1024*1024, 'Fixture inventory exceeds bounded cleanup');
    if (!result.IsTruncated) { continuation = undefined; break; }
    assert(typeof result.NextContinuationToken === 'string' && result.NextContinuationToken.length > 0
      && result.NextContinuationToken !== continuation, 'Invalid inventory continuation');
    continuation = result.NextContinuationToken;
  }
  assert(!continuation && new Set(objects.map(object => object.Key)).size === objects.length, 'Incomplete or repeated object inventory');
  j.cleanup = {status:'requested',objects:objects.length}; await save(j);
  for (let offset = 0; offset < objects.length; offset += 1000) {
    await verifyBucket(j,api); // Identity and versioning are rechecked before each destructive batch.
    const batch = objects.slice(offset,offset+1000);
    const result = await api('delete-objects',{...bucketArgs(j),Delete:{Objects:batch,Quiet:false}});
    assert(!(result.Errors?.length) && Array.isArray(result.Deleted) && result.Deleted.length === batch.length
      && batch.every(object => result.Deleted.some(deleted => deleted.Key === object.Key)), 'Object deletion incomplete; inspect before retrying');
  }
  await verifyBucket(j,api,{tagged:Boolean(j.tagged)});
  const remaining = await api('list-objects-v2',{...bucketArgs(j),MaxKeys:1});
  assert(remaining.IsTruncated === false && remaining.KeyCount === 0 && !(remaining.Contents?.length), 'Bucket is not empty after cleanup');
  await api('delete-bucket',bucketArgs(j));
  for (let attempt = 0; attempt < 15; attempt++) {
    if (await api('head-bucket',bucketArgs(j),'404') === null) {
      j.cleanup = {...j.cleanup,status:'deleted',verifiedAt:new Date().toISOString()}; await save(j); return;
    }
    await sleep(2000);
  }
  throw new Error('Bucket deletion was not confirmed; manual inspection required');
}

async function main(action,path,binary) {
  assert(['create','exercise','cleanup'].includes(action) && path, 'Pass create|exercise|cleanup and a journal path');
  assert(process.env.GITHUB_ACTIONS === 'true' && process.env.GITHUB_REPOSITORY === 'mylife-inc/Zellij', 'Run this lifecycle in Zellij GitHub Actions');
  const env = awsEnvironment(process.env), api = awsApi(env);
  const save = async j => { await writeFile(`${path}.tmp`,JSON.stringify(j,null,2)+'\n'); await rename(`${path}.tmp`,path); };
  let j;
  if (action === 'create') {
    assert(env.ZEL_S3_ACCOUNT_CONFIRMED === 'true', 'Confirm the account and test budget before provisioning');
    j = validateJournal({version:1,bucket:`zellij-live-${new Date().toISOString().slice(0,10).replaceAll('-','')}-${randomBytes(6).toString('hex')}`,
      account:env.S3_EXPECTED_BUCKET_OWNER,region:'us-east-2',run:env.GITHUB_RUN_ID,source:env.GITHUB_SHA,creation:'not-started',cases:[]});
    await writeFile(path,JSON.stringify(j,null,2)+'\n',{flag:'wx'});
    await createFixture(j,api,save);
  } else {
    j = validateJournal(JSON.parse(await readFile(path,'utf8')));
    assert(j.account === env.S3_EXPECTED_BUCKET_OWNER && j.run === env.GITHUB_RUN_ID && j.source === env.GITHUB_SHA, 'Journal belongs to another account or Actions run');
    if (action === 'cleanup') await cleanupFixture(j,api,save);
    else {
      assert(binary, 'Pass the current zel binary');
      await exerciseFixture(j,api,args => {
        const result = spawnSync(binary,args,{env:{...env,NO_COLOR:'1'},encoding:'utf8',maxBuffer:32*1024*1024,timeout:10*60*1000});
        process.stdout.write(result.stdout ?? ''); process.stderr.write(result.stderr ?? '');
        assert.equal(result.status,0,'zel s3 failed'); return result.stdout;
      },checkDeliverySite,save);
    }
  }
  console.log(`S3 ${action}: ${j.bucket} (${j.creation})`);
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) await main(...process.argv.slice(2));
