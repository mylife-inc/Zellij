import path from 'node:path';
import { stat } from 'node:fs/promises';

import { FolderContentSource } from '../content/folder-source';
import { LegacyContentSource } from '../content/legacy-source';
import { loadSite, type Site } from '../content/site';
import { ZellijContentError } from '../errors';

export interface CheckOptions {
  /** Content root, absolute or relative to cwd. Defaults to ./content. */
  contentDir?: string;
  cwd?: string;
}

export interface CheckResult {
  ok: boolean;
  warnings: string[];
  site?: Site;
}

/**
 * Validates a content root without building anything — design §11.
 *
 * Everything the build would reject is rejected here, and everything it would
 * warn about is printed. The point is a fast loop for content authors: no
 * Next.js, no compilation, no dev server.
 */
export async function check(options: CheckOptions = {}): Promise<CheckResult> {
  const cwd = options.cwd ?? process.cwd();
  const contentDir = path.resolve(cwd, options.contentDir ?? 'content');
  const warnings: string[] = [];
  const collect = (message: string): void => {
    warnings.push(message);
  };

  if (!(await exists(contentDir))) {
    console.error(`✗ No content root at ${contentDir}`);
    return { ok: false, warnings };
  }

  const legacy = await exists(path.join(contentDir, 'marketing'));
  const source = legacy
    ? new LegacyContentSource({
        contentDir,
        configFile: path.resolve(cwd, 'zellij.config.yaml'),
        onWarning: collect,
      })
    : new FolderContentSource({ contentDir, onWarning: collect });

  try {
    const site = await loadSite(source);
    warnings.push(...site.warnings);

    report(site, contentDir, warnings, legacy);
    return { ok: true, warnings, site };
  } catch (err) {
    for (const warning of warnings) console.warn(`  ! ${warning}`);
    console.error('');
    console.error(err instanceof ZellijContentError ? err.message : (err as Error).stack);
    return { ok: false, warnings };
  }
}

function report(site: Site, contentDir: string, warnings: string[], legacy: boolean): void {
  const guidePageCount = site.guidePages.length;

  console.log(`✓ ${contentDir}${legacy ? '  (legacy marketing/ layout)' : ''}`);
  console.log(`  site        ${site.config.name} · theme ${site.config.theme}`);
  console.log(`  pages       ${site.pages.length}`);
  console.log(
    `  guides      ${site.guides.length}` +
      (site.guides.length > 0
        ? ` (${guidePageCount} page${guidePageCount === 1 ? '' : 's'}) · ` +
          site.guides.map((guide) => guide.route).join(', ')
        : ''),
  );
  console.log(`  menu        ${site.menu.map((item) => item.label).join(' · ') || '(empty)'}`);
  console.log(`  home        ${site.homeRedirect ? `/ → ${site.homeRedirect}` : (site.home?.route ?? '(none)')}`);
  console.log(`  routes      ${site.routes.size}`);

  if (warnings.length > 0) {
    console.log('');
    for (const warning of warnings) console.warn(`  ! ${warning}`);
  }
}

async function exists(target: string): Promise<boolean> {
  try {
    await stat(target);
    return true;
  } catch {
    return false;
  }
}
