import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile, readdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';

test('print font snapshot matches its pinned sources and licenses', async () => {
  const root = new URL('../cli/crates/zellij-docs/assets/print-fonts/', import.meta.url);
  const manifest = JSON.parse(await readFile(new URL('manifest.json', root)));
  assert.equal(manifest.version, 1);
  for (const entry of manifest.files) {
    assert(!entry.path.startsWith('/') && !entry.path.split('/').includes('..'));
    const bytes = await readFile(new URL(entry.path, root));
    assert.equal(bytes.length, entry.bytes);
    assert.equal(createHash('sha256').update(bytes).digest('hex'), entry.sha256, entry.path);
    if (entry.path.endsWith('OFL.txt')) assert.match(bytes.toString(), /SIL OPEN FONT LICENSE/);
    else assert(['00010000', '4f54544f'].includes(bytes.subarray(0, 4).toString('hex')), entry.path);
  }
  for (const family of ['amiri', 'libertinus']) {
    assert.deepEqual((await readdir(new URL(family+'/', root))).sort(), manifest.files.filter(f => f.path.startsWith(family+'/')).map(f => f.path.split('/')[1]).sort());
  }
});
