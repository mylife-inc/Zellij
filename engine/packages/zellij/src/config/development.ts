import { readFile } from 'node:fs/promises';
import path from 'node:path';

/** Published only by a live preparation watcher; contains no source paths. */
export const DEVELOPMENT_REVISION_FILE = 'dev-revision.json';

export async function readDevelopmentRevision(appRoot: string): Promise<string | undefined> {
  try {
    const value: unknown = JSON.parse(await readFile(
      path.join(appRoot, 'public', '_zellij', DEVELOPMENT_REVISION_FILE), 'utf8',
    ));
    if (value && typeof value === 'object' && 'version' in value && value.version === 1
      && 'revision' in value && typeof value.revision === 'string' && value.revision) {
      return value.revision;
    }
  } catch (error) {
    if (!(error instanceof SyntaxError) && (error as NodeJS.ErrnoException).code !== 'ENOENT') throw error;
  }
  return undefined;
}
