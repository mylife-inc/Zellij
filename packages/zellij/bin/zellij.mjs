#!/usr/bin/env node
/**
 * Zellij's build-time CLI. Wrapper apps run these as predev/prebuild steps
 * (spec §13.4, §13.6).
 */
import { syncAssets } from '../src/cli/sync-assets.ts';
import { writeSearchIndex } from '../src/cli/write-search-index.ts';

const [command] = process.argv.slice(2);

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
  default:
    console.error(
      `Usage: zellij <command>\n\n` +
        `  prepare        sync assets and build the search index\n` +
        `  sync-assets    copy content assets into public/_zellij/assets\n` +
        `  search-index   write the guide search index\n\n` +
        `Unknown command: ${command ?? '(none)'}`,
    );
    process.exit(1);
}
