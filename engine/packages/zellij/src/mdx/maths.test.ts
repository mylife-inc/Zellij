import { afterEach, expect, it, vi } from 'vitest';
import { loadMaths } from './maths';
afterEach(() => vi.unstubAllEnvs());
it('disables math even after installed plugins were loaded and cached', async () => {
  vi.stubEnv('ZELLIJ_MATHS', 'true');
  expect((await loadMaths())[0]).toHaveLength(1);
  vi.stubEnv('ZELLIJ_MATHS', 'false');
  expect(await loadMaths()).toEqual([[], []]);
  vi.stubEnv('ZELLIJ_MATHS', 'true');
  expect((await loadMaths())[0]).toHaveLength(1);
});
