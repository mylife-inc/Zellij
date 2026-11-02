// Exercise session overrides through real native processes, watched config
// edits, failed rebuilds, browser rendering, and a later unmodified invocation.
import assert from 'node:assert/strict';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { chromium, expect } from '@playwright/test';

export async function checkPreviewFeatures({ temp, start, stop, snapshot, until }) {
  for (const enabled of [true, false]) {
    const root = path.join(temp, `Preview features ${enabled}`);
    const content = path.join(root, 'content');
    const output = path.join(root, 'site');
    const config = path.join(root, 'named preview.yaml');
    const selection = enabled ? 'all' : 'none';
    const settings = name => `version: 1\nrenderer: portable\nname: ${name}\npreview: {port: 0}\nmarkdown: {math: ${!enabled}, mermaid: ${!enabled}}\n`;
    const document = title => `# ${title}\n\n$x^2$\n\n\`\`\`mermaid\ngraph TD; A-->B;\n\`\`\`\n`;
    await mkdir(content, { recursive: true });
    await writeFile(config, settings('Initial settings'));
    await writeFile(path.join(root, 'zel.yaml'), 'invalid neighbor: [');
    await writeFile(path.join(content, 'index.md'), document('Initial document'));
    const args = enabled ? ['v', '--watch', '--no-open'] : ['dev'];
    const server = await start([...args, '--config', config, '--features', selection]);
    const status = () => fetch(`${server.url}_zellij/preview.json`).then(r => r.json());
    const browser = await chromium.launch();
    try {
      const page = await browser.newPage();
      await page.goto(server.url);
      const checkFeatures = async () => {
        const html = await readFile(path.join(output, 'index.html'), 'utf8');
        assert.equal(html.includes('<math'), enabled);
        assert.equal(html.includes('class="mermaid-diagram"'), enabled);
        if (enabled) {
          await expect(page.locator('math')).toBeVisible();
          await expect(page.locator('.mermaid-diagram svg')).toBeVisible({ timeout: 15000 });
        } else {
          await expect(page.locator('math, .mermaid-diagram')).toHaveCount(0);
          assert((await page.locator('article').innerText()).includes('$x^2$'));
        }
      };
      await checkFeatures();
      // Config remains contradictory: the session override must survive rereads.
      await writeFile(config, settings('Changed settings'));
      await expect(page.locator('.brand')).toHaveText('Changed settings', { timeout: 20000 });
      await writeFile(path.join(content, 'index.md'), document('Changed document'));
      await expect(page.getByRole('heading', { name: 'Changed document', exact: true })).toBeVisible({ timeout: 20000 });
      await checkFeatures();
      const good = await snapshot(output);
      await writeFile(path.join(content, 'index.md'), '[Broken](absent.md)');
      await until(async () => (await status()).error, 'invalid feature-preview edit');
      assert.deepEqual(await snapshot(output), good);
      await writeFile(path.join(content, 'index.md'), document('Recovered document'));
      await expect(page.getByRole('heading', { name: 'Recovered document', exact: true })).toBeVisible({ timeout: 20000 });
      await checkFeatures();
      assert.equal(await readFile(config, 'utf8'), settings('Changed settings'));
      assert.equal(await readFile(path.join(content, 'index.md'), 'utf8'), document('Recovered document'));
      await stop(server);
      // A new command without --features reads the saved setting again.
      const ordinary = await start(['view', '--config', config, '--no-open']);
      const html = await readFile(path.join(output, 'index.html'), 'utf8');
      assert.equal(html.includes('<math'), !enabled);
      assert.equal(html.includes('class="mermaid-diagram"'), !enabled);
      await stop(ordinary);
    } finally { await browser.close(); }
  }
  console.log('Preview features: dev/view overrides survive config edits and recovery, preserve sources, and reset for later commands.');
}
