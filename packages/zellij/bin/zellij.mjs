#!/usr/bin/env node
/**
 * Zellij's build-time CLI. v1 exposes a single command, `sync-assets`, which
 * wrapper apps run as a predev/prebuild step (spec §13.4).
 */
import { syncAssets } from '../src/cli/sync-assets.ts';

const [command] = process.argv.slice(2);

switch (command) {
  case 'sync-assets':
    await syncAssets(process.cwd());
    break;
  default:
    console.error(`Usage: zellij sync-assets\n\nUnknown command: ${command ?? '(none)'}`);
    process.exit(1);
}
