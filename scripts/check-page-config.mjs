import assert from 'node:assert/strict';
import { expect } from '@playwright/test';

// Real content loaders/layouts/page factories, with independent root layouts so
// enabled and disabled settings can share one production build.
export function pageConfigFiles(prefix) {
  const files = {};
  for (const name of ['fixed', 'picker']) {
    const content = `page-config-${name}`;
    const base = `/settings/${name}`;
    const root = `src/app/(settings-${name})${base}`;
    files[`${content}/site.yaml`] = `name: Page settings\nbasePath: ${prefix}\ntheme: cupertino\nmode: system\nmotion: none\nnav: { themeToggle: true, themeSwitcher: ${name === 'picker'} }\n${name === 'picker' ? 'overrides: { accent: "#112233", radius: lg, fontBody: "Georgia, serif" }\n' : ''}`;
    files[`${content}/pages.yaml`] = ['plain', 'launch', 'other'].map(page => `- name: ${page}\n  route: ${base}/${page}\n${page === 'launch' ? '  theme: mono\n' : page === 'other' ? '  theme: aurora\n' : ''}  sections: [hero, links]\n`).join('');
    files[`${content}/sections/hero.yaml`] = 'type: hero\nheadline: Page configuration\nsubhead: Theme isolation example\n';
    files[`${content}/sections/links.yaml`] = `type: markdown-prose\nbody: |\n${[['Ordinary page', 'plain'], ['Themed page', 'launch'], ['Other theme', 'other'], ['Guide page', 'guide']].map(([label, page]) => `  [${label}](${base}/${page})`).join('\n\n')}\n`;
    files[`${content}/guide/_sidebar.yaml`] = `route: ${base}/guide\nnav: [index]\n`;
    files[`${content}/guide/index.mdx`] = '# Guide fixture\n';
    files[`${root}/layout.tsx`] = layout(content);
    for (const page of ['plain', 'launch', 'other', 'guide']) {
      files[`${root}/${page}/page.tsx`] = `import { createZellijPages } from '@shebka/zellij';\nconst site = createZellijPages({ contentDir: './${content}' });\nexport default function Page() { return site.Page({ params: Promise.resolve({ slug: ${JSON.stringify(['settings', name, page])} }) }); }\n`;
    }
  }
  for (const name of ['on', 'off']) {
    const content = `analytics-${name}`;
    const root = `src/app/(analytics-${name})/analytics/${name}`;
    files[`${content}/site.yaml`] = `name: Analytics fixture\nbasePath: ${prefix}\nnav: { themeToggle: true }\nanalytics: ${name === 'on' ? '{ provider: plausible, domain: docs.example.com }' : '{ provider: none }'}\n`;
    files[`${content}/guide/_sidebar.yaml`] = `route: /analytics/${name}\nnav: [index]\n`;
    files[`${content}/guide/index.mdx`] = '# Analytics fixture\n';
    files[`${root}/layout.tsx`] = layout(content);
    for (const page of ['one', 'two']) files[`${root}/${page}/page.tsx`] = `import Link from 'next/link';\nexport default function Page() { return <main id="content"><h1>Analytics ${page}</h1><Link href="/analytics/${name}/${page === 'one' ? 'two' : 'one'}">Next page</Link><a href="#anchor">Section anchor</a><h2 id="anchor">Anchor</h2></main>; }\n`;
  }
  return files;
}

function layout(content) {
  return `import '@shebka/zellij/styles.css';\nimport { createZellijRootLayout } from '@shebka/zellij';\nexport default createZellijRootLayout({ contentDir: './${content}' });\n`;
}

export async function checkPageThemes(browser, origin, prefix) {
  for (const name of ['fixed', 'picker']) {
    const context = await browser.newContext({ colorScheme: 'light' });
    const errors = [];
    try {
      const page = await context.newPage();
      page.on('pageerror', error => errors.push(error.message));
      const base = `${origin}${prefix}/settings/${name}`;
      await page.goto(`${base}/launch`);
      const main = page.locator('main');
      const headline = main.getByRole('heading', { level: 1 });
      await expect(main).toHaveAttribute('data-zellij-theme', 'mono');
      // Cupertino's explicit 600 weight must not override Mono's 700 token.
      await expect(headline).toHaveCSS('font-weight', '700');
      await expect(headline).toHaveCSS('font-family', /Space Grotesk/);
      await expect(main).toHaveCSS('color', 'rgb(0, 0, 0)');
      await expect(page.locator('body')).toHaveCSS('color', 'rgb(29, 29, 31)');
      await expect(page.locator('.zellij-header')).toHaveCSS('--zellij-nav-height', '3rem');
      await expect(main).toHaveCSS('--zellij-nav-height', '3rem');
      if (name === 'picker') {
        await expect(main).toHaveCSS('--zellij-accent', '#112233');
        await expect(main).toHaveCSS('--zellij-radius', '1rem');
        await expect(main).toHaveCSS('font-family', 'Georgia, serif');
      }
      await page.emulateMedia({ colorScheme: 'dark' });
      await expect(main).toHaveCSS('color', 'rgb(255, 255, 255)');
      await page.getByRole('button', { name: 'Switch to light theme', exact: true }).click();
      await expect(main).toHaveCSS('color', 'rgb(0, 0, 0)');
      if (name === 'picker') {
        await page.getByRole('button', { name: 'Change theme', exact: true }).press('ArrowDown');
        await page.getByRole('button', { name: 'Forest', exact: true }).click();
        await expect(page.locator('html')).toHaveAttribute('data-zellij-theme', 'forest');
        await expect(main).toHaveAttribute('data-zellij-theme', 'mono');
        await expect(headline).toHaveCSS('font-weight', '700');
      }
      await main.getByRole('link', { name: 'Other theme', exact: true }).click();
      await expect(page).toHaveURL(`${base}/other`);
      await expect(main).toHaveAttribute('data-zellij-theme', 'aurora');
      await expect(main).toHaveCSS('background-color', 'rgb(247, 249, 255)');
      await expect(headline).toHaveCSS('font-family', /Outfit/);
      await main.getByRole('link', { name: 'Ordinary page', exact: true }).click();
      await expect(page).toHaveURL(`${base}/plain`);
      await expect(main).not.toHaveAttribute('data-zellij-theme');
      await expect(headline).toHaveCSS('font-weight', name === 'picker' ? '650' : '600');
      await main.getByRole('link', { name: 'Guide page', exact: true }).click();
      await expect(page.getByRole('heading', { name: 'Guide fixture', exact: true })).toBeVisible();
      await expect(page.locator('main')).not.toHaveAttribute('data-zellij-theme');
      await page.goBack();
      await expect(page).toHaveURL(`${base}/plain`);
      await main.getByRole('link', { name: 'Themed page', exact: true }).click();
      await expect(page).toHaveURL(`${base}/launch`);
      await page.reload();
      await expect(main).toHaveAttribute('data-zellij-theme', 'mono');
      await expect(headline).toHaveCSS('font-weight', '700');
      assert.deepEqual(errors, []);
    } finally { await context.close(); }
  }
  const context = await browser.newContext({ javaScriptEnabled: false, colorScheme: 'dark' });
  try {
    const page = await context.newPage();
    await page.goto(`${origin}${prefix}/settings/fixed/launch`);
    await expect(page.locator('main')).toHaveCSS('color', 'rgb(255, 255, 255)');
    await expect(page.getByRole('heading', { level: 1 })).toHaveCSS('font-weight', '700');
    await page.locator('main').getByRole('link', { name: 'Other theme', exact: true }).click();
    await expect(page.locator('main')).toHaveCSS('background-color', 'rgb(11, 16, 32)');
  } finally { await context.close(); }
}

export async function checkAnalytics(browser, origin, prefix) {
  // Serve the local fixture at a non-local test origin; leave Plausible's real
  // localhost exclusion intact. Every event is intercepted, never transmitted.
  const virtual = 'http://zellij.example';
  for (const scenario of ['on', 'off', 'ignored', 'blocked', 'localhost']) {
    const context = await browser.newContext();
    const events = [];
    const errors = [];
    await context.addInitScript(ignored => {
      // Official tracker bypass for its automation filter, only in this fixture.
      window.__plausible = true;
      if (ignored && location.protocol.startsWith('http')) localStorage.plausible_ignore = 'true';
    }, scenario === 'ignored');
    await context.route('**/*', async route => {
      const request = route.request();
      const url = new URL(request.url());
      if (url.href === 'https://plausible.io/api/event') {
        events.push(request.postDataJSON());
        if (scenario === 'blocked') return route.abort();
        return route.fulfill({ status: 202, headers: { 'access-control-allow-origin': '*' }, body: '{}' });
      }
      if (url.origin === virtual) {
        return route.fulfill({ response: await route.fetch({ url: `${origin}${url.pathname}${url.search}` }) });
      }
      if (url.origin === origin) return route.continue();
      throw new Error(`Unexpected external request: ${url}`);
    });
    try {
      const page = await context.newPage();
      page.on('pageerror', error => errors.push(error.message));
      const base = `${scenario === 'localhost' ? origin : virtual}${prefix}/analytics/${scenario === 'off' ? 'off' : 'on'}`;
      await page.goto(`${base}/one`);
      await expect(page.getByRole('heading', { name: 'Analytics one', exact: true })).toBeVisible();
      await expect(page.getByRole('button', { name: /Switch to .* theme/ })).toBeVisible();
      if (scenario !== 'off') await expect.poll(() => page.evaluate(() => window.plausible?.s)).toBe('npm');
      const views = () => events.filter(event => event.n === 'pageview').map(event => event.u);
      const tracked = scenario === 'on' || scenario === 'blocked';
      if (tracked) await expect.poll(views).toEqual([`${base}/one`]);
      await page.evaluate(() => { window.zellijNavigationSentinel = true; });
      await page.getByRole('link', { name: 'Next page', exact: true }).click();
      await expect(page).toHaveURL(`${base}/two`);
      assert.equal(await page.evaluate(() => window.zellijNavigationSentinel), true, 'Must test client-side navigation');
      if (tracked) await expect.poll(views).toEqual([`${base}/one`, `${base}/two`]);
      await page.getByRole('link', { name: 'Section anchor', exact: true }).click();
      await expect(page).toHaveURL(`${base}/two#anchor`);
      await page.goBack();
      await page.goBack();
      await expect(page).toHaveURL(`${base}/one`);
      if (tracked) await expect.poll(views).toEqual([`${base}/one`, `${base}/two`, `${base}/one`]);
      await page.getByRole('button', { name: /Switch to .* theme/ }).click();
      await page.waitForLoadState('networkidle');
      assert.equal(views().length, tracked ? 3 : 0, 'No duplicate or disabled pageviews');
      assert(events.every(event => event.d === 'docs.example.com'));
      if (scenario === 'off') assert.equal(await page.evaluate(() => typeof window.plausible), 'undefined');
      assert.deepEqual(errors, []);
    } finally { await context.close(); }
  }
}
