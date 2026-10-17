import path from 'node:path';
import { stat } from 'node:fs/promises';

import { FolderContentSource } from '../content/folder-source';
import { LegacyContentSource } from '../content/legacy-source';
import { loadSite, type Site } from '../content/site';
import { ZellijContentError } from '../errors';
import { deepCheck } from './deep-check';

export interface CheckOptions {
  /** Content root, absolute or relative to cwd. Defaults to ./content. */
  contentDir?: string;
  cwd?: string;
  /** Parse MDX and validate statically known local links, anchors and media. */
  deep?: boolean;
  /** Treat warnings as a failed check, including unverifiable dynamic MDX. */
  strict?: boolean;
}

export interface CheckResult {
  ok: boolean;
  warnings: string[];
  errors: string[];
  site?: Site;
}

/**
 * Validates a content root without building anything — design §11.
 *
 * Checks the content model and its supported navigation references. MDX
 * compilation still needs build acceptance. Deep mode adds nonexecuting static
 * reference/syntax analysis; neither mode needs Next.js or downloads assets.
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
    return { ok: false, warnings, errors: [`No content root at ${contentDir}`] };
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

    const deep = options.deep ? await deepCheck(site, contentDir) : { errors: [], warnings: [] };
    warnings.push(...deep.warnings);
    const ok = deep.errors.length === 0 && !(options.strict && warnings.length > 0);
    report(site, contentDir, warnings, legacy, ok);
    for (const error of deep.errors) console.error(`  ✗ ${error}`);
    if (options.strict && warnings.length) console.error(`Strict check failed: ${warnings.length} warning(s).`);
    return { ok, warnings, errors: deep.errors, site };
  } catch (err) {
    for (const warning of warnings) console.warn(`  ! ${warning}`);
    console.error('');
    console.error(err instanceof ZellijContentError ? err.message : (err as Error).stack);
    return { ok: false, warnings, errors: [(err as Error).message] };
  }
}

function report(site: Site, contentDir: string, warnings: string[], legacy: boolean, ok: boolean): void {
  const guidePageCount = site.guidePages.length;

  console.log(`${ok ? '✓' : '✗'} ${contentDir}${legacy ? '  (legacy marketing/ layout)' : ''}`);
  console.log(`  site        ${site.config.name} · theme ${site.config.theme}`);
  console.log(`  pages       ${site.pages.length}`);
  console.log(
    `  guides      ${site.guides.length}` +
      (site.guides.length > 0
        ? ` (${guidePageCount} page${guidePageCount === 1 ? '' : 's'}) · ` +
          site.guides.map((guide) => guide.route).join(', ')
        : ''),
  );
  if (site.examples.size > 0) {
    console.log(`  examples    ${site.examples.size} validated · examples/`);
  }
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
