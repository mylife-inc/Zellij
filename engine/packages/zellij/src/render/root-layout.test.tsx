import { renderToStaticMarkup } from 'react-dom/server';
import { expect, it, vi } from 'vitest';
import { siteConfigSchema } from '../config/schema';
import { loadZellijRuntime } from '../config/runtime';
import type { Site } from '../content/site';
import { themeInitScript } from '../chrome/theme-preferences';
import { createZellijRootLayout } from './root-layout';

vi.mock('../config/runtime', () => ({ loadZellijRuntime: vi.fn() }));
vi.mock('../chrome/nav-bar', () => ({ NavBar: () => <nav>Navigation</nav> }));
vi.mock('../chrome/footer', () => ({ Footer: () => <footer>Footer</footer> }));

async function render(config: unknown) {
  vi.mocked(loadZellijRuntime).mockResolvedValue({
    site: { config: siteConfigSchema.parse(config) } as Site,
    contentDir: '/fixture/content', layout: 'folder',
  });
  const Root = createZellijRootLayout();
  return renderToStaticMarkup(await Root({ children: <main>Visible content</main> }));
}

it('emits managed theme CSS in head with an identity that changes with its contents', async () => {
  const config = { name: 'Fixture', theme: 'cupertino' };
  const first = await render(config);
  const repeated = await render(config);
  const changed = await render({ ...config, overrides: { accent: '#123456' } });
  const stylesheet = (html: string) => {
    const head = html.slice(0, html.indexOf('</head>'));
    const styles = [...head.matchAll(/<style data-precedence="zellij-theme" data-href="([^"]+)">([\s\S]*?)<\/style>/g)];
    expect(styles).toHaveLength(1);
    expect(styles[0]![1]).toMatch(/^zellij-theme-[a-f0-9]{64}$/);
    expect(styles[0]![2]).toContain('--zellij-accent:');
    return styles[0]!;
  };
  expect(stylesheet(first)[1]).toBe(stylesheet(repeated)[1]);
  expect(stylesheet(changed)[1]).not.toBe(stylesheet(first)[1]);
  expect(stylesheet(changed)[2]).toContain('#123456');
  expect(first.split('<body>')[1]).not.toContain('<style');
});

it.each([[true, true], [true, false], [false, true], [false, false]])(
  'restores enabled preferences before body content (toggle %s, switcher %s)',
  async (themeToggle, themeSwitcher) => {
    const html = await render({ name: 'Fixture', nav: { themeToggle, themeSwitcher } });
    const [head, body] = html.split('<body>');
    expect(head).not.toContain(themeInitScript);
    if (themeToggle || themeSwitcher) {
      expect(body).toMatch(/^<script\b/);
      expect(body).toContain(themeInitScript);
      expect(body!.indexOf('</script>')).toBeLessThan(body!.indexOf('Skip to content'));
      expect(body!.includes('data-mode-toggle="true"')).toBe(themeToggle);
      expect(body!.includes('data-themes="')).toBe(themeSwitcher);
    } else {
      expect(body).not.toContain('<script');
    }
  },
);
