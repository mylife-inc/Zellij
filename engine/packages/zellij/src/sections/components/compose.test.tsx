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

it('renders composed video with a named player, nested references and readable no-script/print alternatives', () => {
  const value = composeSchema.parse({ type: 'compose', version: 1, label: 'Media', children: [{ type: 'group', children: [{
    type: 'video', src: 'assets/demo.mp4', label: 'Motion study', poster: 'assets/story.svg',
    tracks: [{ src: 'assets/demo.vtt', srclang: 'en', label: 'English' }],
    transcript: '/transcript', caption: 'An **original** study.',
  }] }] });
  const html = renderToStaticMarkup(<Compose section={value} site={site} defaultMotion="none" />);
  expect(html).toContain('Play video: Motion study');
  expect(html).toContain('/product/_zellij/assets/demo.mp4');
  expect(html).toContain('/product/_zellij/assets/demo.vtt');
  expect(html).toContain('srcLang="en"');
  expect(html).toContain('<noscript>');
  expect(html).toContain('<button hidden=""');
  expect(html).not.toContain('<style');
  expect(html).toContain('preload="none"');
  expect(html).not.toContain('autoPlay');
  expect(html).toContain('<strong>original</strong>');
  expect(html).toContain('href="/transcript"');
  expect(html).toContain('Open video: Motion study');
  expect(html).toMatch(/<figcaption><div class="zellij-video-caption">[\s\S]*<p class="zellij-video-links">[\s\S]*<\/p><\/figcaption><\/figure>/);
});

it('keeps the original provider URL as a no-script poster link and mounts no iframe before play', () => {
  const value = composeSchema.parse({ type: 'compose', version: 1, label: 'Provider', children: [{
    type: 'video', src: 'https://www.youtube.com/watch?v=abcdefghijk', label: 'Provider film', poster: 'assets/poster.svg',
  }] });
  const html = renderToStaticMarkup(<Compose section={value} site={site} defaultMotion="none" />);
  expect(html).toContain('<noscript><a class="zellij-video-frame zellij-video-poster" href="https://www.youtube.com/watch?v=abcdefghijk" aria-label="Open video: Provider film">');
  expect(html).not.toContain('<iframe');
  expect(html).not.toContain('<style');
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


it('renders semantic leaves, accessible gallery links and inert scroll/backdrop fallbacks', () => {
  const value = composeSchema.parse({type: 'compose', version: 1, label: 'Rich', scroll: {from: {y: 40}, to: {}},
    layout: {kind: 'layers', backdrop: {kind: 'video', src: 'assets/bg.mp4', poster: 'assets/poster.svg', label: 'A quiet study'}},
    children: [{type: 'group', place: 'center', children: [
      {type: 'list', ordered: true, items: ['First', 'Second']},
      {type: 'quote', text: 'A **good** idea.', author: 'An author'},
      {type: 'icon', name: 'book-open', label: 'Read'},
      {type: 'code', title: 'Install', code: '<script>not executable</script>'},
      {type: 'table', caption: 'Comparison', rowLabel: 'Format', columns: ['Web'], rows: [{label: 'Markdown', cells: ['**Yes**']}]},
      {type: 'disclosure', summary: 'More', body: 'Read [the guide](/guide)'},
      {type: 'carousel', label: 'Ideas', items: [{title: 'First idea'}, {title: 'Second idea', href: '/guide'}]},
      {type: 'gallery', label: 'Studies', items: [{image: 'assets/a.svg', alt: 'A study', caption: 'Original'}]},
    ]}]});
  const html = renderToStaticMarkup(<Compose section={value} site={site} defaultMotion="expressive" />);
  expect(html).toContain('data-scroll-active="false"');
  expect(html).not.toContain('position:sticky');
  expect(html).not.toContain('autoPlay');
  expect(html).toContain('Play background video: A quiet study');
  expect(html).toContain('<ol class="zellij-compose-list">');
  expect(html).toContain('<blockquote>');
  expect(html).toContain('aria-label="Read"');
  expect(html).toContain('&lt;script&gt;not executable&lt;/script&gt;');
  expect(html).toContain('<caption>Comparison</caption>');
  expect(html).toContain('<th scope="row">Markdown</th>');
  expect(html).toContain('<summary>More</summary>');
  expect(html).toContain('aria-label="Ideas"');
  expect(html).toContain('href="/product/_zellij/assets/a.svg"');
  expect(html).not.toContain('/_zellij/product/');
  expect(html).toContain('aria-label="Studies"');
});

it('renders block Markdown without executing HTML or MDX and keeps primary heading ownership', () => {
  const value = composeSchema.parse({type: 'compose', version: 1, label: 'Prose', children: [{type: 'prose', body:
    '# A subordinate heading\n\nA **clear** paragraph.\n\n- One\n- Two\n\n![Diagram](assets/a.svg)\n\n[Unsafe](javascript:alert(1))\n\n<script>alert(1)</script>\n\n{process.exit(1)}'}]});
  const html = renderToStaticMarkup(<Compose section={value} site={site} defaultMotion="none" />);
  expect(html).toContain('<h2>A subordinate heading</h2>');
  expect(html).not.toContain('<h1');
  expect(html).toContain('<strong>clear</strong>');
  expect(html).toContain('<li>One</li>');
  expect(html).toContain('/product/_zellij/assets/a.svg');
  expect(html).not.toContain('<script');
  expect(html).not.toContain('href="javascript:');
  expect(html).toContain('{process.exit(1)}');
});
