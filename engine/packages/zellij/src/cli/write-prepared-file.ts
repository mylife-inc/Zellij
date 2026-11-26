import { randomUUID } from 'node:crypto';
import { rename, rm, writeFile } from 'node:fs/promises';
import { setTimeout as delay } from 'node:timers/promises';

/** Readers see a complete old or new JSON file while development prepares it. */
export async function writePreparedFile(destination: string, text: string): Promise<void> {
  const temporary = `${destination}.${randomUUID()}.tmp`;
  try {
    await writeFile(temporary, text, 'utf8');
    for (let attempt = 0; ; attempt++) {
      try {
        await rename(temporary, destination);
        break;
      } catch (error) {
        // Windows readers can temporarily deny replacement of an open file.
        // Keep the old destination intact; never unlink it to force a commit.
        const code = (error as NodeJS.ErrnoException).code;
        if (process.platform !== 'win32' || attempt >= 6
          || !['EPERM', 'EACCES', 'EBUSY'].includes(code ?? '')) throw error;
        await delay(10 * 2 ** attempt);
      }
    }
  } finally {
    await rm(temporary, { force: true });
  }
}
