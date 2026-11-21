"""Fail-closed preflight regression cases with a fake API. No real networking."""
import hashlib
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

source = Path(__file__).with_name('retire-native-releases.py')
original = json.loads(Path(sys.argv[1]).read_text())
with tempfile.TemporaryDirectory(prefix='zel-native-retire-guards-') as temp:
    root = Path(temp)
    (root/'gh').write_text('#!'+sys.executable+'\n'+'''
import json,os,sys
from pathlib import Path
r=Path(__file__).parent;args=sys.argv[1:];method=args[args.index('--method')+1];endpoint=args[-1]
with (r/'calls.jsonl').open('a') as f:f.write(json.dumps({'method':method,'endpoint':endpoint})+'\\n')
assert method=='GET','A failed preflight must never mutate GitHub'
p=json.loads((r/'plan.json').read_text());scenario=os.environ['SCENARIO']
if endpoint=='user':value={'login':'wrong-owner' if scenario=='owner' else 'mylife-inc'}
elif endpoint.endswith('/commits/main'):
 repo=endpoint[len('repos/'):-len('/commits/main')];value={'sha':'moved' if scenario=='head' else p['repository_heads'][repo]}
elif '/actions/runs/' in endpoint:
 gate=next(g for g in p['gates'] if endpoint==f'repos/{g["repository"]}/actions/runs/{g["run"]}')
 value={'status':'completed','conclusion':'failure' if scenario=='gate' else 'success','head_sha':gate['head']}
else:
 rows=p['targets']+p['protected'];row=next(x for x in rows if endpoint.startswith('repos/'+x['repository']+'/') and (endpoint.endswith('/releases/'+str(x['id'])) or endpoint.endswith('/releases/tags/'+x['tag']) or endpoint.endswith('/git/ref/tags/'+x['tag'])))
 if '/git/ref/' in endpoint:
  value=row['tag_ref'].copy()
  if scenario=='ref':value['object']={'sha':'moved'}
 else:
  value={'id':row['id'],'tag_name':row['tag'],'assets':row['assets'],'draft':not row['draft'] if scenario=='draft' else row['draft'],'prerelease':row['prerelease'],'immutable':scenario=='immutable'}
  if scenario=='asset':value['assets']=[]
  if scenario=='tag-replaced' and '/releases/tags/' in endpoint:value['id']+=1
print(json.dumps(value))
''')
    (root/'gh').chmod(0o700)
    for scenario in ['hash','owner','head','gate','asset','ref','draft','tag-replaced','immutable','plan-target','plan-protected','plan-gates','good']:
        plan=json.loads(json.dumps(original))
        if scenario=='plan-target':plan['targets'][0]['id']+=1
        if scenario=='plan-protected':plan['protected'].pop()
        if scenario=='plan-gates':plan['gates'].pop()
        raw=(json.dumps(plan)+'\n').encode();(root/'plan.json').write_bytes(raw)
        calls=root/'calls.jsonl';calls.unlink(missing_ok=True)
        env={**os.environ,'PATH':str(root)+os.pathsep+os.environ['PATH'],'SCENARIO':scenario,'RETIRE_PLAN_SHA256':'wrong' if scenario=='hash' else hashlib.sha256(raw).hexdigest()}
        result=subprocess.run([sys.executable,str(source),str(root/'plan.json'),str(root/'result.json'),*([] if scenario=='good' else ['--apply'])],env=env,capture_output=True,text=True)
        assert (result.returncode==0)==(scenario=='good'),(scenario,result.stderr)
        requests=[json.loads(line) for line in calls.read_text().splitlines()] if calls.exists() else []
        assert all(r['method']=='GET' for r in requests)
        assert not (root/'result.json').exists()
        print(scenario,'guard passed; no mutation')
