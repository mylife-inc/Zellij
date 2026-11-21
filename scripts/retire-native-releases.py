"""Exact rc.12 native-release retirement. Default: read-only preflight."""
import datetime
import hashlib
import json
import os
import re
import subprocess
import sys
from pathlib import Path

KNOWN = {
    ('mylife-inc/Zellij', 'v0.2.0-rc.12'): 452071949,
    ('mylife-inc/releases', 'zellij-v0.2.0-rc.12'): 452072007,
}
PROTECTED = {
    (repo, prefix + '0.2.0-rc.' + str(version))
    for repo, prefix in [('mylife-inc/Zellij', 'v'), ('mylife-inc/releases', 'zellij-v')]
    for version in [11, 13, 14, 15]
} | {('mylife-inc/releases', 'codeseal-v1.1.0'), ('mylife-inc/releases', 'codeseal-v1.0.4')}
assert len(sys.argv) in [3, 4] and (len(sys.argv) == 3 or sys.argv[3] == '--apply'), \
    'Usage: retire-native-releases.py plan.json journal.json [--apply]'
apply = len(sys.argv) == 4
raw = Path(sys.argv[1]).read_bytes()
digest = hashlib.sha256(raw).hexdigest()
assert os.environ.get('RETIRE_PLAN_SHA256') == digest, 'Reviewed plan hash does not match'
plan = json.loads(raw)
assert plan['schema'] == 1 and plan['status'] == 'reviewed' and plan['replacement'] == 'v0.2.0-rc.15'
assert len(plan['targets']) == 2 and {(t['repository'], t['tag']): t['id'] for t in plan['targets']} == KNOWN
assert len(plan['protected']) == 10 and {(p['repository'], p['tag']) for p in plan['protected']} == PROTECTED
assert set(plan['repository_heads']) == {'mylife-inc/Zellij', 'mylife-inc/releases', 'mylife-inc/homebrew-tap', 'mylife-inc/scoop-bucket'}


def api(endpoint, method='GET', missing=False):
    p = subprocess.run(['gh', 'api', '--hostname', 'github.com', '--method', method, endpoint],
                       text=True, capture_output=True, timeout=60)
    if p.returncode:
        code = re.search(r'HTTP (\d+)', p.stderr)
        if missing and code and code[1] == '404':
            return None
        raise RuntimeError(f'{method} {endpoint}: HTTP {code[1] if code else p.returncode}')
    return json.loads(p.stdout) if p.stdout.strip() else None


def assets(rows):
    return sorted([dict(id=a['id'], name=a['name'], size=a['size'], digest=a.get('digest'))
                   for a in rows], key=lambda a: a['name'])


def unchanged_release(row, missing=False):
    base = f'repos/{row["repository"]}/releases'
    r = api(f'{base}/{row["id"]}', missing=missing)
    by_tag = api(f'{base}/tags/{row["tag"]}', missing=missing)
    assert (r is None) == (by_tag is None), 'Release and tag lookup disagree'
    if r is not None:
        assert r['id'] == by_tag['id'] == row['id'] and r['tag_name'] == row['tag'], 'Release identity changed'
        assert assets(r['assets']) == assets(row['assets']), 'Release assets changed'
        assert r['draft'] == row['draft'] and r['prerelease'] == row['prerelease'], 'Release publication state changed'
    return r


def unchanged_ref(row, missing=False):
    ref = api(f'repos/{row["repository"]}/git/ref/tags/{row["tag"]}', missing=missing)
    if ref is not None:
        assert ref == row['tag_ref'], 'Tag identity changed'
    return ref


def protected():
    for row in plan['protected']:
        assert unchanged_release(row) is not None
        assert unchanged_ref(row) is not None


def heads():
    for repo, head in plan['repository_heads'].items():
        assert api(f'repos/{repo}/commits/main')['sha'] == head, f'{repo} main moved; re-review'


def gates():
    assert len(plan['gates']) == 17 and len({(g['repository'], g['run']) for g in plan['gates']}) == 17
    for g in plan['gates']:
        assert g['repository'] in plan['repository_heads'] and re.fullmatch('[a-f0-9]{40}', g['head'])
        r = api(f'repos/{g["repository"]}/actions/runs/{g["run"]}')
        assert r['status'] == 'completed' and r['conclusion'] == 'success' and r['head_sha'] == g['head'], \
            f'Acceptance gate changed: {g["run"]}'


assert api('user')['login'] == 'mylife-inc', 'Require the intended repository owner login'
heads()
gates()
protected()
for row in plan['targets']:
    r = unchanged_release(row, missing=True)
    if r is not None:
        assert not r.get('immutable', False), 'Refuse immutable release'
    unchanged_ref(row, missing=True)
    print(('REMOVE' if apply else 'PLAN'), row['repository'], row['tag'], row['id'], len(row['assets']), 'assets', flush=True)
if not apply:
    print('Read-only preflight passed. No deletion.')
    sys.exit(0)

out = Path(sys.argv[2])
report = json.loads(out.read_text()) if out.exists() else {
    'plan_sha256': digest, 'replacement': plan['replacement'],
    'started_at': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'events': []}
assert report['plan_sha256'] == digest
report['complete'] = False


def save():
    out.write_text(json.dumps(report, indent=2) + '\n')


save()  # Prove the journal is writable before any remote mutation.
try:
    for row in plan['targets']:
        heads()
        repo, tag, rid = row['repository'], row['tag'], row['id']
        current = unchanged_release(row, missing=True)
        unchanged_ref(row, missing=True)
        if current is not None:
            assert not current.get('immutable', False), 'Refuse immutable release'
            api(f'repos/{repo}/releases/{rid}', 'DELETE')
            report['events'].append(dict(repository=repo, tag=tag, id=rid, action='deleted-release', assets=len(row['assets'])))
            save()
        if unchanged_ref(row, missing=True) is not None:
            api(f'repos/{repo}/git/refs/tags/{tag}', 'DELETE')
            report['events'].append(dict(repository=repo, tag=tag, action='deleted-tag'))
            save()
        assert api(f'repos/{repo}/releases/tags/{tag}', missing=True) is None
        assert api(f'repos/{repo}/git/ref/tags/{tag}', missing=True) is None
    protected()
    report['complete'] = True
    report['finished_at'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
except Exception as error:
    report['error'] = str(error)
    raise
finally:
    save()
print('Retired exactly two rc.12 releases and two tags; rc.11/rc.13/rc.14/rc.15 and CodeSeal are unchanged.')
