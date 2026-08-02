import { describe, expect, it } from 'vitest';

import { buildContext } from './context';
import { registeredSectionTypes } from '../sections/registry';
import '../sections/schemas';

/**
 * The context file is what an AI agent authors a content folder from, so a gap
 * in it becomes invalid YAML in somebody else's repository — and because the
 * output looks plausible, the error surfaces as a failed build rather than as
 * an obviously stale document. These check the generator keeps up with the
 * schemas on its own.
 */
describe('agent context file', () => {
  const context = buildContext();

  it('documents every registered section type', () => {
    const missing = registeredSectionTypes().filter(
      (type) => !context.includes(`### \`${type}\``),
    );
    expect(missing).toEqual([]);
  });

  it('states the section count the registry actually has', () => {
    // The prose says "There are N." — a number that was hand-written once and
    // would otherwise be wrong the first time a section was added.
    expect(context).toContain(`There are ${registeredSectionTypes().length}.`);
  });

  it('expands nested objects rather than naming their keys', () => {
    // A promo tile's fields live under `items`. Listing the key names and
    // stopping there is enough for an agent to write something plausible and
    // invalid, which is the failure this guards against.
    expect(context).toContain('**`items`**');
    expect(context).toMatch(/\| `scrim` \| number \|/);
  });

  it('expands the nav block, which the prose guide never covered', () => {
    expect(context).toContain('**`nav`**');
    expect(context).toMatch(/\| `announcement` \|/);
  });

  it('marks required fields and carries defaults', () => {
    expect(context).toMatch(/\| `headline` \| string \| \*\*required\*\* \|/);
    expect(context).toMatch(/default `"floating"`/);
  });

  it('lists the themes from the schema, not from memory', () => {
    expect(context).toContain('`cupertino`');
    expect(context).toContain('`meridian`');
  });

  it('tells the agent how to know it is finished', () => {
    expect(context).toContain('zellij check');
  });
});
