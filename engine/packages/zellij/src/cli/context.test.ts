import { describe, expect, it } from 'vitest';

import { buildContext, buildSchemaJson } from './context';
import { siteConfigSchema, THEME_NAMES } from '../config/schema';
import { THEMES } from '../themes/tokens';
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

  it('documents both analytics choices and requires a domain only for Plausible', () => {
    const variants = JSON.parse(buildSchemaJson()).files['site.yaml'].properties.analytics.anyOf;
    const plausible = variants.find((option: { properties: { provider: { const: string } } }) => option.properties.provider.const === 'plausible');
    const disabled = variants.find((option: { properties: { provider: { const: string } } }) => option.properties.provider.const === 'none');
    expect(plausible.required).toContain('domain');
    expect(plausible.properties.domain.minLength).toBe(1);
    expect(disabled.required ?? []).not.toContain('domain');
    expect(context).toContain('**`analytics (option 1)`**');
    expect(context).toContain('**`analytics (option 2)`**');
    expect(context).toMatch(/\| `domain` \| string \| \*\*required\*\* \|/);
  });

  it('lists the themes from the schema, not from memory', () => {
    expect(context).toContain('`cupertino`');
    expect(context).toContain('`meridian`');
  });

  it('exports the complete theme catalog used by the renderer and config validator', () => {
    const catalog = JSON.parse(buildSchemaJson()).themes;
    expect(catalog.map((theme: { name: string }) => theme.name)).toEqual([...THEME_NAMES]);
    for (const theme of catalog) {
      const name = siteConfigSchema.parse({ name: 'Catalog test', theme: theme.name }).theme;
      expect(theme.label).toBe(THEMES[name].label);
      expect(theme.description).toBe(THEMES[name].description);
      expect(theme.description.length).toBeGreaterThan(0);
    }
  });

  it('tells the agent how to know it is finished', () => {
    expect(context).toContain('zellij check');
  });
});
