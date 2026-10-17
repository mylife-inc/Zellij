import { spawn, type ChildProcess } from 'node:child_process';
import { createRequire } from 'node:module';
import path from 'node:path';
import { resolveContentPaths, type ZellijRuntimeOptions } from '../config/runtime';
import { watchPreparation, type PreparationWatch } from './watch-preparation';

/** Own the watcher and Next process together, including failures and Ctrl-C. */
export async function runDev(options: ZellijRuntimeOptions, nextArgs?: string[]): Promise<number> {
  const { appRoot } = resolveContentPaths(options);
  // Resolve the app's installed Next; never invoke a shell or download via npx.
  const next = nextArgs === undefined ? undefined
    : createRequire(path.join(appRoot, 'package.json')).resolve('next/dist/bin/next');
  const controller = new AbortController();
  let child: ChildProcess | undefined;
  let watcher: PreparationWatch | undefined;
  let killTimer: ReturnType<typeof setTimeout> | undefined;
  let exitCode = 0;
  const stop = (signal: NodeJS.Signals) => {
    exitCode = signal === 'SIGINT' ? 130 : 143;
    controller.abort();
    if (child && child.exitCode === null && child.signalCode === null) {
      child.kill(signal);
      killTimer ??= setTimeout(() => child?.kill('SIGKILL'), 5000);
    }
  };
  const interrupt = () => stop('SIGINT');
  const terminate = () => stop('SIGTERM');
  process.on('SIGINT', interrupt);
  process.on('SIGTERM', terminate);
  try {
    watcher = await watchPreparation(options, { signal: controller.signal });
    if (controller.signal.aborted) return exitCode;
    if (next) {
      child = spawn(process.execPath, [next, 'dev', ...nextArgs!], { cwd: appRoot, stdio: 'inherit' });
      const result = await new Promise<number>((resolve, reject) => {
        child!.once('error', reject);
        child!.once('exit', (code, signal) => resolve(code ?? (signal === 'SIGINT' ? 130 : 1)));
      });
      return controller.signal.aborted ? exitCode : result;
    }
    await watcher.done;
    return exitCode;
  } finally {
    await watcher?.close();
    if (killTimer) clearTimeout(killTimer);
    process.off('SIGINT', interrupt);
    process.off('SIGTERM', terminate);
  }
}
