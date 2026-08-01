#!/usr/bin/env node
/**
 * Zellij's build-time CLI. Wrapper apps run these as predev/prebuild steps
 * (spec §13.4, §13.6).
 */
import { syncAssets } from '../src/cli/sync-assets.ts';
import { writeSearchIndex } from '../src/cli/write-search-index.ts';

const [command, ...rest] = process.argv.slice(2);

switch (command) {
  case 'sync-assets':
    await syncAssets(process.cwd());
    break;
  case 'search-index':
    await writeSearchIndex(process.cwd());
    break;
  case 'prepare':
    // The usual prebuild pair.
    await syncAssets(process.cwd());
    await writeSearchIndex(process.cwd());
    break;
  case 'check': {
    const { check } = await loadCheck();
    const contentDir = rest.find((arg) => !arg.startsWith('-'));
    const result = await check(contentDir ? { contentDir } : {});
    if (!result.ok) process.exit(1);
    break;
  }
  default:
    console.error(
      `Usage: zellij <command>\n\n` +
        `  check [dir]    validate a content root without building\n` +
        `  prepare        sync assets and build the search index\n` +
        `  sync-assets    copy content assets into public/_zellij/assets\n` +
        `  search-index   write the guide search index\n\n` +
        `Unknown command: ${command ?? '(none)'}`,
    );
    process.exit(1);
}

/**
 * `check` needs the whole content loader, whose relative imports carry no file
 * extensions — Node's ESM resolver requires them, so this one command runs from
 * the bundled build rather than from source.
 */
async function loadCheck() {
  try {
    return await import('../dist/cli/check-bundle.js');
  } catch {
    console.error(
      `zellij check needs the package build.\n` +
        `  Run: npm run build -w @shebka/zellij`,
    );
    process.exit(1);
  }
}
