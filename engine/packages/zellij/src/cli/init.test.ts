import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { expect, it } from 'vitest';
import { init, setFeatures } from './init';

it('carries security constraints into standalone apps and feature changes', async () => {
  const target = await mkdtemp(path.join(os.tmpdir(), 'zellij-init-'));
  try {
    await init({ target, name: 'security-fixture', features: ['all'] });
    const generated = JSON.parse(await readFile(path.join(target, 'package.json'), 'utf8'));
    const engine = JSON.parse(await readFile(new URL('../../package.json', import.meta.url), 'utf8'));
    const root = JSON.parse(await readFile(new URL('../../../../../package.json', import.meta.url), 'utf8'));
    expect(generated.dependencies.next).toBe(engine.peerDependencies.next);
    expect(generated.overrides).toEqual(engine.overrides);
    expect(root.overrides).toMatchObject(engine.overrides);
    expect(generated.dependencies.mermaid).toBe(engine.optionalDependencies.mermaid);
    // Updating an older wrapper must add the same constraints.
    delete generated.overrides;
    await writeFile(path.join(target, 'package.json'), JSON.stringify(generated));
    await setFeatures(target, ['maths']);
    const changed = JSON.parse(await readFile(path.join(target, 'package.json'), 'utf8'));
    expect(changed.overrides).toEqual(engine.overrides);
    expect(changed.dependencies.mermaid).toBeUndefined();
    expect(changed.dependencies['remark-math']).toBe(engine.optionalDependencies['remark-math']);
  } finally {
    await rm(target, { recursive: true, force: true });
  }
});
