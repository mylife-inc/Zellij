// Read-only, loopback-only reproduction using an existing static docs build.
// Run from the repository root: node specs/evidence/serve-head-whitespace-repro.mjs
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const output = path.resolve(process.argv[2] ?? fileURLToPath(new URL('../../examples/zellij-docs/out', import.meta.url)));
const marker = '<meta name="viewport"';
const entry = await readFile(path.join(output, 'cli/completions.html'), 'utf8');
if (!entry.includes(marker)) throw new Error('Expected built docs with /cli/completions and viewport metadata');
const mime = { '.html': 'text/html', '.css': 'text/css', '.js': 'text/javascript', '.json': 'application/json', '.png': 'image/png', '.svg': 'image/svg+xml', '.woff2': 'font/woff2', '.txt': 'text/plain' };
const server = createServer(async (req, res) => {
  try {
    const url = new URL(req.url, 'http://localhost');
    const base = path.resolve(output, `.${decodeURIComponent(url.pathname)}`);
    if (base !== output && !base.startsWith(output + path.sep)) { res.writeHead(403).end(); return; }
    for (const candidate of [base, `${base}.html`, path.join(base, 'index.html')]) {
      try {
        let body = await readFile(candidate);
        if (candidate.endsWith('.html')) {
          const probe = url.searchParams.get('probe');
          const prefix = probe === 'newline' ? '\n' : probe === 'comment' ? '<!-- Hosting annotation -->' : '';
          body = Buffer.from(body.toString('utf8').replace(marker, prefix + marker));
        }
        res.writeHead(200, { 'content-type': mime[path.extname(candidate)] ?? 'application/octet-stream', 'cache-control': 'no-store' }).end(body);
        return;
      } catch (error) { if (!['ENOENT', 'EISDIR', 'ENOTDIR'].includes(error.code)) throw error; }
    }
    res.writeHead(404).end('Not found');
  } catch { res.writeHead(500).end('Local fixture error'); }
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
for (const probe of ['plain', 'newline', 'comment']) {
  console.log(`${probe}: http://127.0.0.1:${server.address().port}/cli/completions?probe=${probe}`);
}
console.log('Open each URL in a fresh browser tab and inspect the console after hydration. Stop with Ctrl+C.');
