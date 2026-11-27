import { build } from 'esbuild';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const root = fileURLToPath(new URL('../../', import.meta.url));
const output = path.join(root, 'examples/showcase/public/__resize-reduction');
await mkdir(output, { recursive: true });
const css = await readFile(new URL('style.css', import.meta.url), 'utf8');
for (const variant of ['dom', 'react']) {
  await build({
    entryPoints: [fileURLToPath(new URL(`${variant}.${variant === 'react' ? 'tsx' : 'ts'}`, import.meta.url))],
    outfile: path.join(output, `${variant}.js`), bundle: true, format: 'iife', platform: 'browser',
    target: 'es2022', jsx: 'automatic', define: { 'process.env.NODE_ENV': '"production"' },
    sourcemap: true,
  });
  await writeFile(path.join(output, `${variant}.html`), `<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Resize reduction: ${variant}</title><style>${css}</style><body><div id="app"></div><main><h1>Repository-owned resize fixture</h1><p>Five links, one overflow panel, no remote resources.</p><div class="background"></div></main><script>document.documentElement.dataset.effects = new URL(location.href).searchParams.get('effects') ?? 'on';</script><script defer src="${variant}.js"></script></body></html>`);
}
console.log('Built standalone DOM and actual-hook fixtures. No engine source modified.');
