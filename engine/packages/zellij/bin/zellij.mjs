#!/usr/bin/env node
/**
 * Zellij's build-time CLI. Wrapper apps run these as predev/prebuild steps
 * (spec §13.4, §13.6).
 */
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const [command, ...rest] = process.argv.slice(2);

/**
 * Loads a command from source where that works, and from the build where it
 * does not.
 *
 * Running the TypeScript directly is what keeps engine development free of a
 * compile step: edit a CLI file, run the command, see the change. That stops
 * working the moment the package is installed as a dependency — Node refuses to
 * strip types from anything under `node_modules` — and it never worked for the
 * commands whose imports reach into the content loader, whose relative
 * specifiers carry no file extensions for Node's resolver to follow.
 *
 * So: source first, but only when this file is not itself inside node_modules,
 * and the bundled build whenever that fails to resolve.
 */
const installed = fileURLToPath(import.meta.url).split(path.sep).includes('node_modules');

async function load(name) {
  if (!installed) {
    try {
      return await import(`../src/cli/${name}.ts`);
    } catch (err) {
      // A missing extension surfaces as ERR_MODULE_NOT_FOUND too, so both the
      // "no such file" and "cannot resolve its imports" cases land here and
      // fall through to the bundle. Anything else is a real bug in the source
      // and is worth failing on rather than papering over with a stale build.
      if (err?.code !== 'ERR_MODULE_NOT_FOUND') throw err;
    }
  }

  return import(`../dist/cli/${name}-bundle.js`);
}

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
  case 'sync-assets': {
    const { syncAssets } = await load('sync-assets');
    await syncAssets(process.cwd());
    break;
  }
  case 'search-index': {
    const { writeSearchIndex } = await load('write-search-index');
    await writeSearchIndex(process.cwd());
    break;
  }
  case 'prepare': {
    // The usual prebuild pair.
    const { syncAssets } = await load('sync-assets');
    const { writeSearchIndex } = await load('write-search-index');
    await syncAssets(process.cwd());
    await writeSearchIndex(process.cwd());
    break;
  }
  case 'init': {
    const { init } = await load('init');
    const { flags, positional } = parseFlags(rest);
    const target = positional[0] ?? '.';

    const created = await init({
      target,
      ...(flags.content ? { content: flags.content } : {}),
      ...(flags.name ? { name: flags.name } : {}),
      ...(flags.port ? { port: Number(flags.port) } : {}),
      ...(flags.force ? { force: true } : {}),
      ...(flags['content-only'] ? { contentOnly: true } : {}),
    });

    if (flags['content-only']) {
      console.log(
        `[zellij] Wrote a content folder to ${created}\n\n` +
          `  zellij check ${target}\n`,
      );
      break;
    }

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

  case 'context': {
    const { writeContext } = await load('context');
    const { flags, positional } = parseFlags(rest);
    const target = flags.out ?? positional[0];

    const written = await writeContext(typeof target === 'string' ? target : undefined);
    if (written !== '(stdout)') {
      console.log(`[zellij] Wrote the agent context file to ${written}`);
    }
    break;
  }

  case 'check': {
    const { check } = await load('check');
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
        `                   --content-only    write just the content folder\n` +
        `  context [file] write a single reference describing Zellij's content\n` +
        `                   model, for an AI agent to author a folder from\n` +
        `  check [dir]    validate a content root without building\n` +
        `  prepare        sync assets and build the search index\n` +
        `  sync-assets    copy content assets into public/_zellij/assets\n` +
        `  search-index   write the guide search index\n\n` +
        `Unknown command: ${command ?? '(none)'}`,
    );
    process.exit(1);
}
