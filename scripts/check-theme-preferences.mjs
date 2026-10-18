// Multiple real root layouts share an origin, exposing all four nav-control
// combinations without rebuilding the fixture or rewriting authored examples.
import assert from 'node:assert/strict';
import { expect } from '@playwright/test';

export function themePreferenceFiles(prefix) {
  const files = {};
  for (const [name, toggle, switcher] of combinations) {
    const content = `preferences-${name}`;
    const root = `src/app/(preferences-${name})/preferences/${name}`;
    files[`${content}/site.yaml`] = `name: ${name}\nbasePath: ${prefix}\ntheme: cupertino\nmode: light\nnav: { themeToggle: ${toggle}, themeSwitcher: ${switcher} }\n`;
    files[`${content}/guide/_sidebar.yaml`] = `route: /preferences/${name}\nnav: [index]\n`;
    files[`${content}/guide/index.mdx`] = `# ${name}\n`;
    files[`${root}/layout.tsx`] = `import '@shebka/zellij/styles.css';\nimport { createZellijRootLayout } from '@shebka/zellij';\nexport default createZellijRootLayout({ contentDir: './${content}' });\n`;
    const page = `import Link from 'next/link';\nexport default function Page() { return <main id="content"><h1>Preference fixture</h1><Link href="/preferences/${name}/second">Second page</Link></main>; }\n`;
    files[`${root}/page.tsx`] = page;
    files[`${root}/second/page.tsx`] = page;
  }
  return files;
}

const combinations = [['both', true, true], ['toggle', true, false], ['switcher', false, true], ['neither', false, false]];

export async function checkThemePreferences(browser, origin, prefix) {
  for (const [name, toggle, switcher] of combinations) {
    const context = await browser.newContext();
    const failures = [];
    context.on('page', page => page.on('pageerror', error => failures.push(error.message)));
    await context.addInitScript(() => {
      if (!['http:', 'https:'].includes(location.protocol)) return;
      if (!sessionStorage.getItem('preferences-seeded')) {
        localStorage.setItem('zellij:mode', 'dark');
        localStorage.setItem('zellij:theme', 'forest');
        sessionStorage.setItem('preferences-seeded', 'yes');
      }
    });
    let release;
    const hydration = new Promise(resolve => { release = resolve; });
    const hold = async route => { await hydration; await route.continue(); };
    await context.route(/\/_next\/.*\.js(?:\?|$)/, hold);
    try {
      const page = await context.newPage();
      await page.goto(`${origin}${prefix}/preferences/${name}`, { waitUntil: 'commit' });
      await expect(page.getByRole('heading', { name: 'Preference fixture' })).toBeVisible();
      // The document's own head script must work before hydration arrives.
      await expect(page.locator('html')).toHaveAttribute('data-zellij-mode', toggle ? 'dark' : 'light');
      await expect(page.locator('html')).toHaveAttribute('data-zellij-theme', switcher ? 'forest' : 'cupertino');
      release();
      await context.unrouteAll({ behavior: 'wait' });
      await page.waitForLoadState('load');
      if (toggle) await page.getByRole('button', { name: 'Switch to light theme', exact: true }).click();
      if (switcher) {
        const button = page.getByRole('button', { name: 'Change theme', exact: true });
        await button.focus();
        await button.press('ArrowDown');
        await page.getByRole('button', { name: 'Carbon', exact: true }).click();
      }
      const theme = switcher ? 'carbon' : 'cupertino';
      await expect(page.locator('html')).toHaveAttribute('data-zellij-theme', theme);
      await expect(page.locator('html')).toHaveAttribute('data-zellij-mode', 'light');
      await page.getByRole('link', { name: 'Second page', exact: true }).click();
      await expect(page).toHaveURL(`${origin}${prefix}/preferences/${name}/second`);
      await page.reload();
      await expect(page.locator('html')).toHaveAttribute('data-zellij-theme', theme);
      await expect(page.locator('html')).toHaveAttribute('data-zellij-mode', 'light');
      assert.deepEqual(await page.evaluate(() => [localStorage.getItem('zellij:mode'), localStorage.getItem('zellij:theme')]), [toggle ? 'light' : 'dark', switcher ? 'carbon' : 'forest']);
      assert.deepEqual(failures, []);
    } finally { release(); await context.unrouteAll({ behavior: 'wait' }); await context.close(); }
  }

  for (const blocked of [false, true]) {
    const context = await browser.newContext();
    await context.addInitScript(blocked => {
      if (!['http:', 'https:'].includes(location.protocol)) return;
      if (blocked) {
        Storage.prototype.getItem = Storage.prototype.setItem = () => { throw new DOMException('Storage unavailable', 'SecurityError'); };
      } else {
        localStorage.setItem('zellij:mode', 'invalid');
        localStorage.setItem('zellij:theme', 'missing-theme');
      }
    }, blocked);
    try {
      const page = await context.newPage();
      const errors = []; page.on('pageerror', error => errors.push(error.message));
      await page.goto(`${origin}${prefix}/preferences/both`);
      await expect(page.locator('html')).toHaveAttribute('data-zellij-theme', 'cupertino');
      await expect(page.locator('html')).toHaveAttribute('data-zellij-mode', 'light');
      await page.getByRole('button', { name: 'Switch to dark theme', exact: true }).click();
      await expect(page.locator('html')).toHaveAttribute('data-zellij-mode', 'dark');
      await page.getByRole('button', { name: 'Change theme', exact: true }).press('ArrowDown');
      await page.getByRole('button', { name: 'Forest', exact: true }).click();
      await expect(page.locator('html')).toHaveAttribute('data-zellij-theme', 'forest');
      assert.deepEqual(errors, []);
    } finally { await context.close(); }
  }
  const context = await browser.newContext({ colorScheme: 'dark' });
  await context.addInitScript(() => {
    if (['http:', 'https:'].includes(location.protocol)) localStorage.setItem('zellij:mode', 'system');
  });
  try {
    const page = await context.newPage();
    await page.goto(`${origin}${prefix}/preferences/toggle`);
    await expect(page.getByRole('button', { name: 'Switch to light theme', exact: true })).toBeVisible();
    await page.emulateMedia({ colorScheme: 'light' });
    const toggle = page.getByRole('button', { name: 'Switch to dark theme', exact: true });
    await expect(toggle).toBeVisible();
    await toggle.click();
    await expect(page.locator('html')).toHaveAttribute('data-zellij-mode', 'dark');
  } finally { await context.close(); }

}
