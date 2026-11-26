// Rust owns this process tree; stdin EOF requests cooperative shutdown on every OS.
import { readFile, readdir, symlink, writeFile } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';
import path from 'node:path';
// Do not start descendants until the caller attaches its Windows job/process group.
let disconnected = false;
let running = false;
process.stdin.on('end', () => {
  disconnected = true;
  if (running) process.emit('SIGTERM');
});
await new Promise((resolve, reject) => {
  const ended = () => reject(new Error('Development parent disconnected before startup'));
  process.stdin.once('end', ended);
  process.stdin.once('data', () => { process.stdin.off('end', ended); resolve(); });
});
const request = JSON.parse(await readFile(process.argv[2], 'utf8'));
// No package manager runs here. The validated dependencies stay unchanged;
// .next, feature selection, tsconfig and public assets belong to this session.
// Junctions on Windows do not need Developer Mode or administrator rights.
for (const entry of await readdir(request.modules, { withFileTypes: true })) {
  if (!entry.isDirectory() || entry.name === '@shebka') throw new Error('Unexpected development dependency directory');
  await symlink(path.join(request.modules, entry.name), path.join(process.cwd(), 'node_modules', entry.name),
    process.platform === 'win32' ? 'junction' : 'dir');
}
// Match ordinary editable wrappers, with one narrow additional dependency root.
// Original images avoid the unused native Sharp/libvips optimizer.
const config = {
  images: { unoptimized: true },
  experimental: { turbopackAdditionalRoots: { zellijDependencies: { path: path.dirname(request.modules) } } },
};
await writeFile(path.join(process.cwd(), 'next.config.js'),
  "import { withZellij } from '@shebka/zellij/next-config';\nexport default withZellij(" + JSON.stringify(config) + ");\n");
const { runDev } = await import(pathToFileURL(path.join(process.cwd(), 'node_modules/@shebka/zellij/dist/cli/dev-bundle.js')));
if (disconnected) throw new Error('Development parent disconnected before startup');
process.stdin.resume();
running = true;
process.exitCode = await runDev({ appRoot: process.cwd(), contentDir: request.content },
  ['--turbopack', '--hostname', request.host, '--port', String(request.port)]);
running = false;
process.stdin.pause();
