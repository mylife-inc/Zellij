import { readFile, writeFile, mkdir, copyFile, readdir, lstat, rm } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { isValidElement, type ReactNode } from 'react';
import { renderToString, renderToStaticMarkup } from 'react-dom/server';
import { createZellijPages } from '../render/page';
import { createZellijRootLayout } from '../render/root-layout';
import { loadZellijRuntime } from '../config/runtime';
import { encode } from './encode';
import { decodeNode, jsonForScript } from './decode';
import { fileHref, setFileLocation } from './location';
import { FileRedirect } from './navigation';
import { featureEnabled } from '../../feature-config.cjs';
import { Analytics } from '../chrome/analytics';
import { validateOfflineRoutes } from './validate-routes';

const escapeAttribute = (value: string) => value.replace(/&/g, '&amp;').replace(/"/g, '&quot;').replace(/</g, '&lt;');
function rewriteMarkup(html: string) {
  return html.replace(/\b(href|src|poster)="([^"<>]*)"/g,
    (_, key: string, value: string) => `${key}="${escapeAttribute(fileHref(value.replace(/&amp;/g, '&')))}"`);
}
async function files(root: string): Promise<string[]> {
  const result: string[] = [];
  for (const item of await readdir(root, { withFileTypes: true })) {
    const file = path.join(root, item.name);
    if (item.isSymbolicLink()) throw new Error(`Offline output must not contain symlinks: ${file}`);
    if (item.isDirectory()) result.push(...await files(file));
    else if (item.isFile()) result.push(file);
  }
  return result;
}
/** Called on a staged Next static export, before the CLI publishes its output. */
export async function exportOffline(appRoot = process.cwd(), output = path.join(appRoot, 'out')) {
  const root = path.resolve(output);
  if (root === path.parse(root).root || root === path.resolve(appRoot) || path.resolve(appRoot).startsWith(root + path.sep)) throw new Error('Offline output must be a separate generated folder');
  const diagrams = featureEnabled('diagrams', appRoot);
  process.env.NEXT_PUBLIC_ZELLIJ_DIAGRAMS = String(diagrams);
  if ((await lstat(root)).isSymbolicLink()) throw new Error('Offline output cannot be a symlink');
  const inventory = await files(root); // reject nested symlinks before the first write
  const { site } = await loadZellijRuntime({ appRoot });
  const routes = [...site.routes];
  validateOfflineRoutes(routes);
  const pages = createZellijPages({ appRoot });
  const layout = createZellijRootLayout({ appRoot });
  const runtimeDir = path.join(root, '_zellij', 'file');
  await mkdir(runtimeDir, { recursive: true });
  // Bundled at engine publication time; no client-side module imports or HTTP runtime.
  const built = path.dirname(fileURLToPath(import.meta.url));
  for (const name of diagrams ? ['runtime.js', 'mermaid.js', 'THIRD-PARTY-NOTICES.txt'] : ['runtime.js', 'THIRD-PARTY-NOTICES.txt']) await copyFile(path.join(built, name), path.join(runtimeDir, name));
  const search = JSON.parse(await readFile(path.join(root, '_zellij', 'search-index.json'), 'utf8'));
  await writeFile(path.join(runtimeDir, 'search-index.js'), `window.__zellijSearchIndex=${jsonForScript(search)};\n`);
  let count = 0;
  for (const route of routes) {
    const location = { route, basePath: site.config.basePath, routes };
    setFileLocation(location);
    const file = path.join(root, route.slice(1), 'index.html');
    // The exported Next head is the source of CSS, fonts, metadata and theme tokens.
    const original = await readFile(file, 'utf8').catch(async (error) => {
      if (error.code !== 'ENOENT' || route === '/') throw error;
      return readFile(path.join(root, route.slice(1) + '.html'), 'utf8');
    });
    const htmlStart = original.match(/<html\b[^>]*>/)?.[0];
    let head = original.match(/<head>([\s\S]*?)<\/head>/)?.[1];
    if (!htmlStart || head === undefined) throw new Error(`Missing HTML shell: ${route}`);
    head = head.replace(/<script\b[\s\S]*?<\/script>/g, '')
      .replace(/<link\b[^>]*\bas="script"[^>]*>/g, '')
      .replace(/<link\b[^>]*\brel="modulepreload"[^>]*>/g, '');
    let body: string;
    try {
      const page = await pages.Page({ params: Promise.resolve({ slug: route.split('/').filter(Boolean) }) });
      const shell = await layout({ children: page });
      const children = shell.props.children;
      const bodyElement = children.find((node: any) => node?.type === 'body');
      let init = '';
      const content = bodyElement.props.children.filter((node: any) => {
        if (isValidElement(node) && node.type === 'script') { init += renderToStaticMarkup(node); return false; }
        return !isValidElement(node) || node.type !== Analytics;
      });
      const tree = await encode(content);
      const rendered = renderToString(<div id="zellij-file-root" style={{ display: 'contents' }}>{decodeNode(tree)}</div>, { identifierPrefix: 'zel-file-' });
      body = init + rendered + `<script type="application/json" id="zellij-file-page">${jsonForScript({ location, tree, diagrams })}</script>` +
        `<script defer data-zellij-file-runtime src="${escapeAttribute(fileHref('/_zellij/file/runtime.js'))}"></script>`;
    } catch (error) {
      if (!(error instanceof FileRedirect)) throw error;
      const target = escapeAttribute(fileHref(error.to));
      body = `<meta http-equiv="refresh" content="0;url=${target}"><a href="${target}">Continue</a>`;
    }
    await mkdir(path.dirname(file), { recursive: true });
    await writeFile(file, `<!DOCTYPE html>${htmlStart}<head>${rewriteMarkup(head)}</head><body>${body}</body></html>`);
    // Preserve old flat Next URLs as small local redirects, never leave broken Next pages.
    const flat = path.join(root, route.slice(1) + '.html');
    if (route !== '/' && inventory.includes(flat)) {
      const target = './' + encodeURIComponent(route.split('/').at(-1)!) + '/index.html';
      await writeFile(flat, `<!DOCTYPE html><meta charset="utf-8"><meta http-equiv="refresh" content="0;url=${escapeAttribute(target)}"><title>Continue</title><a href="${escapeAttribute(target)}">Continue to this page</a>`);
    }
    count++;
  }
  // CSS lives at its original location; only root-absolute asset references need rewriting.
  for (const file of inventory.filter(file => file.endsWith('.css'))) {
    const css = await readFile(file, 'utf8');
    const result = css.replace(/url\(\s*(['"]?)(\/[^)'"\s]+)\1\s*\)/g, (all, quote: string, url: string) => {
      if (url.startsWith('//')) return all;
      const stripped = site.config.basePath && url.startsWith(site.config.basePath + '/') ? url.slice(site.config.basePath.length) : url;
      const relative = path.relative(path.dirname(file), path.join(root, stripped.slice(1))).split(path.sep).join('/');
      return `url(${quote}${relative}${quote})`;
    });
    if (result !== css) await writeFile(file, result);
  }
  const routeData = new Set(routes.map(route => path.join(root, (route === '/' ? 'index' : route.slice(1)) + '.txt')));
  for (const file of inventory) {
    const relative = path.relative(root, file).split(path.sep).join('/');
    if ((relative.startsWith('_next/') && /\.js(?:\.map)?$/.test(file)) ||
        (!relative.startsWith('_zellij/') && path.basename(file).startsWith('__next.') && file.endsWith('.txt')) || routeData.has(file)) {
      await rm(file);
    }
  }
  const notFound = path.join(root, '404.html');
  if (inventory.includes(notFound)) await writeFile(notFound, '<!DOCTYPE html><meta charset="utf-8"><title>Page not found</title><h1>Page not found</h1><a href="./index.html">Return home</a>');
  await writeFile(path.join(runtimeDir, 'manifest.json'), JSON.stringify({ format: 1, routes, pages: count }, null, 2));
  console.log(`[zellij] Rich file export: ${count} routes → ${root}`);
}
