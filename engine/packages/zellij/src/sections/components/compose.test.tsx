import { expect, it } from 'vitest';
import '../schemas';
import { renderToStaticMarkup } from 'react-dom/server';
import { composeSchema } from '../compose-schema';
import { Compose } from './compose';
import { SectionsPageView } from '../../render/sections-page';
import type { Site } from '../../content/site';
import type { SectionsPage } from '../../content/types';

const site = { config: { basePath: '/product', motion: 'none' } } as Site;
const section = composeSchema.parse({ type: 'compose', version: 1, label: 'A composed story', id: 'story',
  layout: { kind: 'split' }, children: [
    { type: 'group', children: [
      { type: 'heading', text: 'Make space for ideas.', level: 1, gradient: { colors: ['#be5528', '#f2b568'], effect: 'reveal' } },
      { type: 'text', text: 'Keep **every word** and [read more](/guide).' },
      { type: 'button', label: 'Read the guide', href: '/guide' },
    ] },
    { type: 'image', image: 'assets/story.svg', alt: 'A new perspective', caption: 'Designed with **care**.' },
  ] });

it('renders nested copy, a prefixed image, links and readable gradient fallback in source order', () => {
  const html = renderToStaticMarkup(<Compose section={section} site={site} defaultMotion="none" />);
  expect(html).toContain('data-layout="split"');
  expect(html).toContain('<strong>every word</strong>');
  expect(html).toContain('href="/guide"');
  expect(html).toContain('/product/_zellij/assets/story.svg');
  expect(html).toContain('alt="A new perspective"');
  expect(html).toContain('aria-hidden="true" class="zellij-gradient-reveal-paint"');
  expect(html).toContain('<span>Make space for ideas.</span>');
  expect(html.indexOf('Make space')).toBeLessThan(html.indexOf('A new perspective'));
});

it('uses instance heading ownership and an explicit rail label', () => {
  const page = { title: 'Fallback page title', sections: [section], layout: 'stack-sidebar' } as SectionsPage;
  const html = renderToStaticMarkup(<SectionsPageView site={site} page={page} />);
  expect(html.match(/<h1\b/g)).toHaveLength(1);
  expect(html).toContain('A composed story');
  expect(html).not.toContain('Fallback page title');
  const secondary = composeSchema.parse({ ...section, children: [{ type: 'heading', text: 'Secondary', level: 2 }] });
  const fallback = renderToStaticMarkup(<SectionsPageView site={site} page={{ ...page, sections: [secondary] }} />);
  expect(fallback).toContain('<h1 class="zellij-visually-hidden">Fallback page title</h1>');
});

it('retains solid text beneath a static shared gradient so failed background paint cannot hide it', () => {
  const value = composeSchema.parse({ type: 'compose', version: 1, label: 'Static',
    titleGradient: ['unsupportedcolor', 'anotherunsupportedcolor'],
    children: [{ type: 'heading', text: 'Always readable' }] });
  const html = renderToStaticMarkup(<Compose section={value} site={site} defaultMotion="subtle" />);
  expect(html).toContain('<span>Always readable</span>');
  expect(html).toContain('aria-hidden="true" class="zellij-gradient-reveal-paint"');
  expect(html).not.toContain('class="zellij-gradient-text"');
});
