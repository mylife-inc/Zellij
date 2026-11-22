import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { expect, it } from 'vitest';
import { components } from './registry';
it('covers every interactive engine component, with the development-only omission', () => {
  const root = path.resolve('src');
  const names = new Set(Object.keys(components));
  const omitted = new Set(['DevelopmentReload']);
  for (const file of readdirSync(root, { recursive: true }).map(String).filter(name => name.endsWith('.tsx'))) {
    const text = readFileSync(path.join(root, file), 'utf8');
    if (!/^\s*['"]use client['"]/.test(text)) continue;
    for (const match of text.matchAll(/export (?:function|const) ([A-Z]\w*)/g)) {
      expect(names.has(match[1]!) || omitted.has(match[1]!), `Register ${file}: ${match[1]} for file exports`).toBe(true);
    }
  }
});
