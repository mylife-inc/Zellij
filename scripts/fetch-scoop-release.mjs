// Download and verify both public Windows packages for hosted Scoop acceptance.
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import { downloadRelease } from './download-release.mjs';
import { releasePolicy } from './release-policy.mjs';
import path from 'node:path';

const [version, destination = 'dist'] = process.argv.slice(2);
releasePolicy(version);
const installers = Object.fromEntries(await Promise.all(['install.sh', 'install.ps1'].map(async name => [name, await readFile(`scripts/${name}`)])));
await mkdir(destination);
for (const target of ['x86_64-pc-windows-msvc', 'aarch64-pc-windows-msvc']) {
  const result = await downloadRelease({ version, target, repository: 'mylife-inc/releases', scope: 'zellij',
    destination: path.join(destination, target), installers });
  for (const name of [result.archive, `${result.archive}.sha256`]) {
    await writeFile(path.join(destination, name), result.contents[name], { flag: 'wx' });
  }
}
