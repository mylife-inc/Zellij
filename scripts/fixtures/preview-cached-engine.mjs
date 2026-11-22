// Deterministic cached-renderer protocol fixture. Real package acceptance is separate.
import assert from 'node:assert/strict';
import { readFile, writeFile, appendFile, mkdir } from 'node:fs/promises';
import { setTimeout as delay } from 'node:timers/promises';
import path from 'node:path';
const request = JSON.parse(await readFile(process.argv[2], 'utf8'));
assert.equal(request.operation, 'build');
assert.equal(request.target, 'static');
const expected = process.env.ZELLIJ_PREVIEW_FEATURES;
assert.deepEqual(request.features, expected === 'all' ? ['maths', 'diagrams'] : []);
assert(!request.content.startsWith('\\\\?\\'), 'Renderer requests use ordinary Windows paths');
const [title, prefix, mode] = (await readFile(path.join(request.content, 'preview.txt'), 'utf8')).split('\n');
await appendFile(process.env.ZELLIJ_PREVIEW_ENGINE_LOG, title + '\n');
if (mode === 'slow') await delay(2000);
if (mode === 'fail') process.exit(96);
if (mode !== 'empty') {
  if (mode !== 'missing-index') {
    await writeFile(path.join(request.output, 'index.html'), `<!doctype html><html><head><title>${title}</title></head><body><h1>${title}</h1><a href="${prefix}/guide">Guide</a></body></html>`);
    await writeFile(path.join(request.output, 'guide.html'), '<!doctype html><html><head><title>Guide</title></head><body><h1>Guide</h1></body></html>');
  }
  await mkdir(path.join(request.output, '_zellij/static'), { recursive: true });
  await writeFile(path.join(request.output, '_zellij/static/manifest.json'), '{}');
  if (mode !== 'missing-metadata') await writeFile('build-report.json', JSON.stringify({ target: 'static', basePath: prefix }));
}
