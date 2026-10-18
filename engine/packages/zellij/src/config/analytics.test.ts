import { describe, expect, it } from 'vitest';
import { siteConfigSchema } from './schema';

describe('analytics configuration', () => {
  it('does not enable tracking by default', () => {
    expect(siteConfigSchema.parse({ name: 'Test' }).analytics.provider).toBe('none');
    expect(siteConfigSchema.parse({ name: 'Test', analytics: {} }).analytics.provider).toBe('none');
  });

  it.each([undefined, '', '   '])('requires a nonempty domain for Plausible (%j)', domain => {
    expect(() => siteConfigSchema.parse({ name: 'Test', analytics: { provider: 'plausible', domain } })).toThrow();
  });

  it('accepts an explicit Plausible domain and rejects unsupported providers', () => {
    expect(siteConfigSchema.parse({ name: 'Test', analytics: { provider: 'plausible', domain: 'docs.example.com' } }).analytics.domain).toBe('docs.example.com');
    expect(() => siteConfigSchema.parse({ name: 'Test', analytics: { provider: 'umami' } })).toThrow();
  });
});
