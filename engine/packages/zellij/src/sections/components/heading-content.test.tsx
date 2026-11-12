import { beforeAll, expect, it } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import path from 'node:path';
import { FolderContentSource, loadSite, type Site } from '../../content';
import { FeatureGrid } from './feature-grid';
import { StepsPanel } from './steps-panel';
import { Faq } from './faq';
import { HeroSplit } from './hero-split';

let site: Site;
const warnings: string[] = [];
beforeAll(async () => {
  site = await loadSite(new FolderContentSource({
    contentDir: path.resolve('../../../examples/codeseal-docs/content'),
    onWarning: message => warnings.push(message),
  }));
});

it('retains authored labels and the FAQ introduction through schema validation and rendering', () => {
  expect(warnings).toEqual([]);
  const html = site.pages.flatMap(page => page.sections.map(section => {
    switch (section.type) {
      case 'feature-grid': return renderToStaticMarkup(<FeatureGrid site={site} section={section} defaultMotion="none" />);
      case 'steps-panel': return renderToStaticMarkup(<StepsPanel site={site} section={section} defaultMotion="none" />);
      case 'faq': return renderToStaticMarkup(<Faq site={site} section={section} defaultMotion="none" />);
      default: return '';
    }
  })).join('\n');
  for (const label of ['The problem', 'How it works', 'Security model', 'Questions', 'Before you pick a mode',
    'Mode 1 · secrets', 'Mode 2 · portal-key', 'Mode 3 · portal-build', 'Mode 4 · portal-dispatch', 'What it does', 'Architecture']) {
    expect(html).toContain(`<p class="zellij-eyebrow">${label}</p>`);
  }
  expect(html).toContain('href="/guide/faq"');
  expect(html).toContain('before anyone installs anything.');
});

it('keeps the comma inside the authored hero highlight label', () => {
  const section = site.pages.find(page => page.route === '/portal')?.sections.find(section => section.type === 'hero-split');
  if (!section || section.type !== 'hero-split') throw new Error('Expected portal hero');
  expect(renderToStaticMarkup(<HeroSplit site={site} section={section} defaultMotion="none" />)).toContain('Audited, exportable');
});
