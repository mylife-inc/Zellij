import { expect, it } from 'vitest';
import { z } from 'zod';
import { registerSection } from '../sections/registry';
import { validateSection, warnIgnoredFields } from './model-schema';

it('retains defaults, numeric text coercion, and fields accepted by extension schemas', () => {
  const warnings: string[] = [];
  const warn = (message: string) => warnings.push(message);
  const timeline = validateSection('timeline.yaml', {
    type: 'timeline', items: [{ date: 2026, title: 'One' }, { date: 2027, title: 'Two' }],
  }, warn);
  expect(timeline).toMatchObject({ items: [{ date: '2026' }, { date: '2027' }] });
  registerSection({ type: 'diagnostic-extension', schema: z.looseObject({ type: z.literal('diagnostic-extension') }) });
  const extra = { arbitrary: [{ data: 'Retained extension content' }] };
  expect(validateSection('custom.yaml', { type: 'diagnostic-extension', extra }, warn)).toHaveProperty('extra', extra);
  expect(warnings).toEqual([]);
});

it('quotes unusual keys, tests own properties and does not print discarded values', () => {
  const warnings: string[] = [];
  const raw = JSON.parse('{"constructor":"hidden value","odd.key\\nnext":{"child":"hidden value"}}');
  warnIgnoredFields('section.yaml', raw, {}, message => warnings.push(message));
  expect(warnings).toHaveLength(2);
  expect(warnings[0]).toContain('"constructor"');
  expect(warnings[1]).toContain('odd.key');
  expect(warnings[1]).not.toContain('\n');
  expect(warnings.join('\n')).not.toContain('hidden value');
  expect(warnings.join('\n')).not.toContain('child');
});
