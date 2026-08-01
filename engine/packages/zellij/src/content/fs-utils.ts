import { readdir, stat } from 'node:fs/promises';
import path from 'node:path';

export async function exists(target: string): Promise<boolean> {
  try {
    await stat(target);
    return true;
  } catch {
    return false;
  }
}

/** Recursively lists files under `dir` matching `test`, returned dir-relative. */
export async function walk(dir: string, test: (name: string) => boolean): Promise<string[]> {
  if (!(await exists(dir))) return [];
  const out: string[] = [];

  async function recurse(current: string, prefix: string): Promise<void> {
    const entries = await readdir(current, { withFileTypes: true });
    for (const entry of entries) {
      if (entry.name.startsWith('.')) continue;
      const rel = prefix ? `${prefix}/${entry.name}` : entry.name;
      if (entry.isDirectory()) {
        await recurse(path.join(current, entry.name), rel);
      } else if (test(entry.name)) {
        out.push(rel);
      }
    }
  }

  await recurse(dir, '');
  return out.sort();
}

/** Immediate subdirectories of `dir`, excluding dotfiles. Sorted. */
export async function subdirectories(dir: string): Promise<string[]> {
  if (!(await exists(dir))) return [];
  const entries = await readdir(dir, { withFileTypes: true });
  return entries
    .filter((entry) => entry.isDirectory() && !entry.name.startsWith('.'))
    .map((entry) => entry.name)
    .sort();
}
