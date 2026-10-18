// Cargo embeds this snapshot; native scaffolding never executes Node or Docker.
import { readFile, writeFile } from 'node:fs/promises';
import { wrapperTemplate } from '../engine/packages/zellij/src/cli/init.ts';
const file = new URL('../cli/crates/zellij-scaffold/generated/app.json', import.meta.url);
const expected = JSON.stringify(await wrapperTemplate(), null, 2) + '\n';
if (process.argv.includes('--write')) await writeFile(file, expected);
else if ((await readFile(file, 'utf8')).replaceAll('\r\n', '\n') !== expected) {
  throw new Error('Native wrapper template is stale. Run node scripts/prepare-app-scaffold.mjs --write');
}
