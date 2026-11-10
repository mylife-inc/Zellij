import {test} from 'node:test';
import assert from 'node:assert/strict';
import {validateLiveTargets,canDeleteImageVersion} from './check-live-providers.mjs';
const fixture = () => ({session:'zellij-live-20261110-abcdef',cloudflare:{project:'zellij-live-20261110-abcdef',account:'a'.repeat(32)},github:{repository:'mylife-inc/zellij-live-20261110-abcdef',repositoryId:123,url:'https://mylife-inc.github.io/zellij-live-20261110-abcdef/'},railway:{project:'11111111-1111-4111-8111-111111111111',environment:'22222222-2222-4222-8222-222222222222',service:'33333333-3333-4333-8333-333333333333',url:'https://temporary.up.railway.app/'}});
test('accepts explicit disposable target identities',()=>assert.equal(validateLiveTargets(fixture()).session,fixture().session));
test('rejects shared repositories, other projects, malformed IDs and foreign URLs before deployment',()=>{
  for (const change of [v=>v.github.repository='mylife-inc/Zellij',v=>v.github.repositoryId=0,v=>v.cloudflare.project='production',v=>v.cloudflare.account='../account',v=>v.railway.service='missing',v=>v.railway.url='https://production.example.test/',v=>v.railway.url='https://user:pass@temporary.up.railway.app/',v=>v.github.url='https://mylife-inc.github.io/Zellij/']) {
    const value=fixture();change(value);assert.throws(()=>validateLiveTargets(value));
  }
});

test('image cleanup preserves shared, release and unrelated versions',()=>{
  const targets = [{package:'zellij/acceptance-static',tag:'live-123-static'}];
  const version = tags => ({metadata:{container:{tags}}});
  assert(canDeleteImageVersion(version(['live-123-static']),'zellij/acceptance-static',targets));
  for (const tags of [[],['0.2.0-rc.2'],['live-123-static','0.2.0-rc.2'],['live-124-static']]) {
    assert(!canDeleteImageVersion(version(tags),'zellij/acceptance-static',targets));
  }
  assert(!canDeleteImageVersion(version(['live-123-static']),'unrelated/package',targets));
  assert(!canDeleteImageVersion({},'zellij/acceptance-static',targets));
});
