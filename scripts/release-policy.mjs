// Shared by tagged CI releases and the local publishing helpers. No npm dependencies.
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export function releasePolicy(tag) {
  const match = /^v(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?$/.exec(tag ?? '');
  if (!match || match[0] !== tag || match[4]?.split('.').some(part => /^0\d+$/.test(part))) {
    throw new Error('Use vMAJOR.MINOR.PATCH or vMAJOR.MINOR.PATCH-prerelease (for example v0.2.0-rc.1).');
  }
  const version = tag.slice(1);
  // Leave room for the longest feature suffix in Docker's 128-character tag limit.
  if (version.length + '-diagrams'.length > 128) throw new Error('Release version is too long for image tags.');
  return { version, series: `${match[1]}.${match[2]}`, stable: !match[4], prerelease: !!match[4] };
}

export function imageTags(tag, image, features = 'none') {
  const policy = releasePolicy(tag);
  if (/\s/.test(image) || !/^[a-z0-9]+(?:[.-][a-z0-9]+)*(?::[0-9]+)?\/[a-z0-9]+(?:[._-][a-z0-9]+)*(?:\/[a-z0-9]+(?:[._-][a-z0-9]+)*)$/.test(image)) {
    throw new Error('Expected an image repository without a tag or digest.');
  }
  if (!['none', 'maths', 'diagrams', 'all'].includes(features)) throw new Error('Unknown image feature selection.');
  const suffix = features === 'none' ? '' : `-${features}`;
  const versions = policy.stable ? [policy.version, policy.series, 'latest'] : [policy.version];
  return versions.map(version => `${image}:${version}${suffix}`);
}

export function releaseNotes(tag, repository, scope = '', image = 'ghcr.io/mylife-inc/zellij-base') {
  const policy = releasePolicy(tag);
  if (/\s/.test(repository + scope) || !/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(repository) || (scope && !/^[A-Za-z0-9-]+$/.test(scope))) {
    throw new Error('Invalid release repository or scope.');
  }
  const assetBase = `https://github.com/${repository}/releases/download/${scope ? `${scope}-` : ''}${tag}`;
  const images = ['none', 'maths', 'diagrams', 'all'].map(features => imageTags(tag, image, features)[0]);
  return `\`zel\` ${tag}.${policy.prerelease ? ' This is a prerelease; installation is explicitly pinned below.' : ''}

## Install this version

Linux/macOS:

\`\`\`sh
curl -fsSL '${assetBase}/install.sh' -o install.sh
ZEL_VERSION='${tag}' ZEL_RELEASE_BASE_URL='${assetBase}' sh ./install.sh
\`\`\`

Windows PowerShell:

\`\`\`powershell
Invoke-WebRequest '${assetBase}/install.ps1' -OutFile install.ps1
.\\install.ps1 -Version '${tag}' -ReleaseBaseUrl '${assetBase}'
\`\`\`

The installers verify archive checksums. Add the reported installation directory
to your PATH. Or download an available platform archive below and verify its checksum.

## Container images

For hosted/container commands, select the matching engine for this session.
Portable Markdown builds do not need an engine image.

\`\`\`sh
export ZELLIJ_IMAGE='${images[0]}'
\`\`\`

\`\`\`powershell
$env:ZELLIJ_IMAGE = '${images[0]}'
\`\`\`

These are the exact version references; check the release workflow's image jobs
before using them if image publication failed independently of native packaging.

\`\`\`text
${images.join('\n')}
\`\`\`
`;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const [command, tag, ...args] = process.argv.slice(2);
    if (command === 'outputs') {
      for (const [key, value] of Object.entries(releasePolicy(tag))) console.log(`${key}=${value}`);
    } else if (command === 'image-tags') {
      console.log(imageTags(tag, ...args).join('\n'));
    } else if (command === 'notes') {
      process.stdout.write(releaseNotes(tag, ...args));
    } else {
      throw new Error('Usage: release-policy.mjs outputs|image-tags|notes <version> [arguments]');
    }
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
