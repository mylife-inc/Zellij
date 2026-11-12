import { expect, it } from 'vitest';
import { parse } from 'yaml';
import { buildSectionPresets } from './section-presets';
import { buildSchemaJson, buildContext } from './context';
import { validateSection } from '../content/model-schema';

it('exports exactly the validated, editable preset artifacts embedded by the native CLI', () => {
  const presets = buildSectionPresets();
  expect(Object.keys(presets).sort()).toEqual(['feature-cards', 'media-accordion', 'product-hero', 'split-story']);
  expect(JSON.parse(buildSchemaJson()).sectionPresets).toEqual(presets);
  const context = buildContext();
  for (const [name, preset] of Object.entries(presets)) {
    const warnings: string[] = [];
    const value = parse(preset.yaml);
    validateSection(name, value, warning => warnings.push(warning));
    expect(warnings).toEqual([]);
    expect(value.type).toBe(preset.sectionType);
    expect(value).not.toHaveProperty('preset');
    expect(context).toContain(`zel ns -p ${name}`);
  }
});
