import { randomUUID } from 'node:crypto';
import { rename, rm, writeFile } from 'node:fs/promises';

/** Readers see a complete old or new JSON file while development prepares it. */
export async function writePreparedFile(destination: string, text: string): Promise<void> {
  const temporary = `${destination}.${randomUUID()}.tmp`;
  try {
    await writeFile(temporary, text, 'utf8');
    await rename(temporary, destination);
  } finally {
    await rm(temporary, { force: true });
  }
}
