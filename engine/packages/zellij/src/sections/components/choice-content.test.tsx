import React from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import { expect, it } from 'vitest';
import { selectorNodeSchema, explorerNodeSchema } from '../choice-schema';
import { ChoiceContent } from './choice-content';

const items = [
  {id: 'first', label: 'First', heading: 'First story', body: '**Clear** reading.', media: {image: 'assets/one.svg', alt: 'First study'}, facts: [{label:'Weight',value:'**560 g**'}], cta: {label:'Read',href:'/guide'}},
  {id: 'second', label: 'Second', heading: 'Second story', body: 'Another thought.'},
];
it('renders every panel in source order with isolated IDs and hidden progressive controls', () => {
  const node = selectorNodeSchema.parse({type:'selector',label:'Choices',initial:'second',items});
  const html = renderToStaticMarkup(<><ChoiceContent node={node} basePath="/manual" /><ChoiceContent node={node} basePath="/manual" /></>);
  expect(html).toContain('data-ready="false"');
  expect(html).toContain('class="zellij-choice-controls" hidden="" role="radiogroup"');
  expect(html).not.toMatch(/class="zellij-choice-panel"[^>]*hidden/);
  expect(html.indexOf('First story')).toBeLessThan(html.indexOf('Second story'));
  expect(html).toContain('src="/manual/_zellij/assets/one.svg"');
  expect(html).toContain('<dt>Weight</dt><dd><strong>560 g</strong></dd>');
  expect(html).toContain('href="/guide"');
  const ids = [...html.matchAll(/ id="([^"]+)"/g)].map(m => m[1]);
  expect(new Set(ids).size).toBe(ids.length);
  for (const match of html.matchAll(/(?:aria-controls|aria-labelledby)="([^"]+)"/g)) expect(ids).toContain(match[1]);
  const names = [...html.matchAll(/ name="([^"]+)"/g)].map(m => m[1]);
  expect(new Set(names).size).toBe(2);
});
it('keeps explorer panel headings subordinate and exposes all content before hydration', () => {
  const node = explorerNodeSchema.parse({type:'explorer',label:'Features',presentation:'list',itemHeadingLevel:4,items});
  const html = renderToStaticMarkup(<ChoiceContent node={node} basePath="" />);
  expect(html).toContain('aria-orientation="vertical"');
  expect(html).toContain('<h4');
  expect(html).not.toContain('<h1');
  expect(html).toContain('Second story');
  expect(html).not.toContain('role="tabpanel"');
});
