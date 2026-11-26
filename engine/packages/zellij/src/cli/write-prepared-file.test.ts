import { mkdtemp, readFile, readdir, rename, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { afterEach, expect, it, vi } from 'vitest';
import { writePreparedFile } from './write-prepared-file';

vi.mock('node:fs/promises', async (original) => {
  const fs = await original<typeof import('node:fs/promises')>();
  return { ...fs, rename: vi.fn(fs.rename) };
});
const native = await vi.importActual<typeof import('node:fs/promises')>('node:fs/promises');
const platform = process.platform;
const roots: string[] = [];
afterEach(async () => {
  Object.defineProperty(process, 'platform', { value: platform });
  vi.mocked(rename).mockReset().mockImplementation(native.rename);
  await Promise.all(roots.splice(0).map(root => rm(root, { recursive: true, force: true })));
});
async function fixture() {
  const root = await mkdtemp(path.join(tmpdir(), 'zellij-prepared-lock-'));
  roots.push(root);
  const file = path.join(root, 'marker.json');
  await writeFile(file, 'previous');
  return { root, file };
}

it.each(['EPERM', 'EACCES', 'EBUSY'])('retries temporary Windows %s without removing the previous file', async code => {
  const f = await fixture();
  Object.defineProperty(process, 'platform', { value: 'win32' });
  const blocked = async () => {
    expect(await readFile(f.file, 'utf8')).toBe('previous');
    throw Object.assign(new Error('temporarily locked'), { code });
  };
  vi.mocked(rename).mockImplementationOnce(blocked).mockImplementationOnce(blocked);
  await writePreparedFile(f.file, 'replacement');
  expect(rename).toHaveBeenCalledTimes(3);
  expect(await readFile(f.file, 'utf8')).toBe('replacement');
  expect(await readdir(f.root)).toEqual(['marker.json']);
});

it('bounds persistent Windows locks and retains the prior file without temporary leftovers', async () => {
  const f = await fixture();
  Object.defineProperty(process, 'platform', { value: 'win32' });
  const error = Object.assign(new Error('still locked'), { code: 'EPERM' });
  vi.mocked(rename).mockRejectedValue(error);
  await expect(writePreparedFile(f.file, 'replacement')).rejects.toBe(error);
  expect(rename).toHaveBeenCalledTimes(7);
  expect(await readFile(f.file, 'utf8')).toBe('previous');
  expect(await readdir(f.root)).toEqual(['marker.json']);
});

it.each([['linux', 'EPERM'], ['win32', 'ENOSPC'], ['win32', 'ENOENT']])('does not retry %s %s', async (target, code) => {
  const f = await fixture();
  Object.defineProperty(process, 'platform', { value: target });
  const error = Object.assign(new Error('not transient'), { code });
  vi.mocked(rename).mockRejectedValue(error);
  await expect(writePreparedFile(f.file, 'replacement')).rejects.toBe(error);
  expect(rename).toHaveBeenCalledTimes(1);
  expect(await readFile(f.file, 'utf8')).toBe('previous');
  expect(await readdir(f.root)).toEqual(['marker.json']);
});
