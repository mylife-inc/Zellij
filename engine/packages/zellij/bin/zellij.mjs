#!/usr/bin/env node
/**
 * Zellij's build-time CLI. Wrapper apps run these as predev/prebuild steps
 * (spec §13.4, §13.6).
 */
import { init } from '../src/cli/init.ts';
import { syncAssets } from '../src/cli/sync-assets.ts';
import { writeSearchIndex } from '../src/cli/write-search-index.ts';

const [command, ...rest] = process.argv.slice(2);

/** `--flag value` and `--flag=value`, plus bare `--flag` for booleans. */
function parseFlags(args) {
  const flags = {};
  const positional = [];

  for (let i = 0; i < args.length; i += 1) {
    const arg = args[i];
    if (!arg.startsWith('--')) {
      positional.push(arg);
      continue;
    }

    const [name, inline] = arg.slice(2).split('=');
    if (inline !== undefined) {
      flags[name] = inline;
    } else if (args[i + 1] && !args[i + 1].startsWith('--')) {
      flags[name] = args[i + 1];
      i += 1;
    } else {
      flags[name] = true;
    }
  }

  return { flags, positional };
}

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
  case 'init': {
    const { flags, positional } = parseFlags(rest);
    const target = positional[0] ?? '.';

    const created = await init({
      target,
      ...(flags.content ? { content: flags.content } : {}),
      ...(flags.name ? { name: flags.name } : {}),
      ...(flags.port ? { port: Number(flags.port) } : {}),
      ...(flags.force ? { force: true } : {}),
    });

    const steps = [
      ...(target === '.' ? [] : [`cd ${target}`]),
      'npm install',
      'npm run dev',
    ];
    console.log(
      `[zellij] Wrote a wrapper app to ${created}\n\n` +
        steps.map((step) => `  ${step}`).join('\n') +
        '\n',
    );
    break;
  }

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
        `  init [dir]     generate a Next.js wrapper app around a content folder\n` +
        `                   --content <path>  use an existing content folder\n` +
        `                   --name <name>     package name\n` +
        `                   --port <n>        dev port\n` +
        `                   --force           write into a non-empty directory\n` +
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
