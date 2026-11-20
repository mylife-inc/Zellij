/** The downloaded renderer's entry point. No Next build, npm or external tools. */
import { readFile, writeFile, mkdir, cp } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createElement } from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import { loadZellijRuntime } from '../config/runtime';
import { createZellijPages } from '../render/page';
import { createZellijRootLayout } from '../render/root-layout';
import { createZellijSitemap, createZellijRobots } from '../render/seo';
import { syncAssets } from '../cli/sync-assets';
import { writeSearchIndex } from '../cli/write-search-index';
import { check, type JsonCheckReport } from '../cli/check';
import { exportOffline } from './export';
import { assetHref } from '../content/asset-href';
import { validateOfflineRoutes } from './validate-routes';

const escape = (value: unknown) => String(value).replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c]!);

async function main() {
  if (process.argv.length !== 3) throw new Error('Expected the renderer request JSON path');
  const request = JSON.parse(await readFile(process.argv[2]!, 'utf8'));
  const { content, output, features, noCheck, operation = 'build' } = request;
  if (!['build', 'check'].includes(operation)) throw new Error('Unknown renderer operation');
  if (typeof content !== 'string' || !path.isAbsolute(content)) throw new Error('Expected absolute content path');
  if (operation === 'build' && (typeof output !== 'string' || !path.isAbsolute(output))) throw new Error('Expected absolute output path');
  if (!Array.isArray(features) || features.some(f => !['maths', 'diagrams'].includes(f))) throw new Error('Invalid feature selection');
  // All writes stay in the CLI-owned temporary work directory and staged output.
  const appRoot = process.cwd();
  await writeFile(path.join(appRoot, 'package.json'), JSON.stringify({ zellij: { features } }));
  await writeFile(path.join(appRoot, 'zellij.config.yaml'), JSON.stringify({ version: 1, content }));
  process.env.ZELLIJ_MATHS = String(features.includes('maths'));
  process.env.NEXT_PUBLIC_ZELLIJ_DIAGRAMS = String(features.includes('diagrams'));
  if (operation === 'check') {
    if (typeof request.deep !== 'boolean' || typeof request.strict !== 'boolean') throw new Error('Invalid validation options');
    let report: JsonCheckReport | undefined;
    const result = await check({ cwd: appRoot, contentDir: content, deep: request.deep, strict: request.strict,
      json: true, onReport: value => { report = value; } });
    if (!report) throw new Error('Validation did not produce a report');
    await writeFile(path.join(appRoot, 'check-report.json'), JSON.stringify(report));
    process.exitCode = result.ok ? 0 : 1;
    return;
  }
  if (!noCheck && !(await check({ cwd: appRoot, contentDir: content })).ok) throw new Error('Content validation failed');
  await syncAssets({ appRoot });
  await writeSearchIndex({ appRoot });
  await cp(path.join(appRoot, 'public'), output, { recursive: true });
  const bundled = path.dirname(fileURLToPath(import.meta.url));
  const resources = path.join(output, '_zellij/file');
  await mkdir(resources, { recursive: true });
  await cp(path.join(bundled, 'styles.css'), path.join(resources, 'styles.css'));
  await cp(path.join(bundled, 'FONT-LICENSES.txt'), path.join(resources, 'FONT-LICENSES.txt'));
  await cp(path.join(bundled, 'files'), path.join(resources, 'files'), { recursive: true });
  const { site } = await loadZellijRuntime({ appRoot });
  validateOfflineRoutes(site.routes);
  const pages = createZellijPages({ appRoot });
  const shell = await createZellijRootLayout({ appRoot })({ children: null });
  const { children, ...attributes } = shell.props;
  const htmlStart = renderToStaticMarkup(createElement('html', attributes)).match(/^<html\b[^>]*>/)![0];
  const theme = renderToStaticMarkup(children.find((node: any) => node?.type === 'head')).replace(/^<head>|<\/head>$/g, '');
  for (const route of site.routes) {
    const metadata = await pages.generateMetadata({ params: Promise.resolve({ slug: route.split('/').filter(Boolean) }) });
    let head = '<meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">' +
      `<title>${escape(metadata.title ?? site.config.name)}</title>` +
      `<link rel="stylesheet" href="${escape(site.config.basePath + '/_zellij/file/styles.css')}">` + theme;
    for (const [name, value] of Object.entries({ description: metadata.description, 'twitter:card': 'summary_large_image', 'twitter:title': metadata.title, 'twitter:description': metadata.description })) {
      if (value) head += `<meta name="${name}" content="${escape(value)}">`;
    }
    const canonical = site.config.siteUrl ? site.config.siteUrl.replace(/\/+$/, '') + site.config.basePath + route : undefined;
    if (canonical) head += `<link rel="canonical" href="${escape(canonical)}">`;
    for (const [property, value] of Object.entries({ 'og:title': metadata.title, 'og:description': metadata.description, 'og:site_name': site.config.name, 'og:type': 'website', 'og:url': canonical })) {
      if (value) head += `<meta property="${property}" content="${escape(value)}">`;
    }
    if (site.config.ogImage) {
      const asset = assetHref(site.config.ogImage, site.config.basePath);
      const image = site.config.siteUrl ? new URL(asset, site.config.siteUrl).href : asset;
      head += `<meta property="og:image" content="${escape(image)}"><meta name="twitter:image" content="${escape(image)}">`;
    }
    const file = path.join(output, route.slice(1), 'index.html');
    await mkdir(path.dirname(file), { recursive: true });
    await writeFile(file, `<!DOCTYPE html>${htmlStart}<head>${head}</head><body></body></html>`);
  }
  const sitemap = await createZellijSitemap({ appRoot })();
  if (sitemap.length) await writeFile(path.join(output, 'sitemap.xml'), '<?xml version="1.0" encoding="UTF-8"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">' + sitemap.map(item => `<url><loc>${escape(item.url)}</loc></url>`).join('') + '</urlset>');
  const robots = await createZellijRobots({ appRoot })();
  await writeFile(path.join(output, 'robots.txt'), `User-agent: *\nAllow: /\n${robots.sitemap ? `Sitemap: ${robots.sitemap}\n` : ''}`);
  await writeFile(path.join(output, '404.html'), '<!DOCTYPE html><title>Page not found</title>');
  await exportOffline(appRoot, output);
  await writeFile(path.join(appRoot, 'build-report.json'), JSON.stringify({ basePath: site.config.basePath }));
}
main().catch(error => { console.error(`[zellij] ${error instanceof Error ? error.message : String(error)}`); process.exitCode = 1; });
