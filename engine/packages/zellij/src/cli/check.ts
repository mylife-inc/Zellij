import { createContentSource } from '../config/runtime';
import { loadSite, type Site } from '../content/site';
import { ZellijContentError } from '../errors';
import { deepCheck } from './deep-check';
import path from 'node:path';

export interface CheckOptions {
  /** Content root; otherwise honor wrapper configuration, then ./content. */
  contentDir?: string;
  cwd?: string;
  /** Parse MDX and validate statically known local links, anchors and media. */
  deep?: boolean;
  /** Treat warnings as a failed check, including unverifiable dynamic MDX. */
  strict?: boolean;
  /** Emit one versioned validation report instead of terminal output. */
  json?: boolean;
}

export interface CheckDiagnostic {
  severity: 'error' | 'warning';
  code: 'content' | 'deep' | 'warning';
  /** Source-relative when known structurally; never inferred from message text. */
  file: string | null;
  message: string;
}

export interface JsonCheckReport {
  schema_version: 1;
  target: 'static';
  root: string;
  deep: boolean;
  strict: boolean;
  ok: boolean;
  pages: number | null;
  diagnostics: CheckDiagnostic[];
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
  let root = path.resolve(cwd, options.contentDir ?? 'content');
  const warnings: string[] = [];
  const collect = (message: string): void => {
    warnings.push(message);
  };
  const json = (ok: boolean, errors: CheckDiagnostic[], site?: Site) => {
    const report: JsonCheckReport = {
      schema_version: 1, target: 'static', root,
      deep: options.deep ?? false, strict: options.strict ?? false,
      ok, pages: site?.routes.size ?? null,
      diagnostics: [
        ...warnings.map((message): CheckDiagnostic => ({ severity: 'warning', code: 'warning', file: null, message })),
        ...errors,
      ],
    };
    console.log(JSON.stringify(report));
  };

  try {
    const { source, contentDir, layout } = await createContentSource({ appRoot: cwd, contentDir: options.contentDir }, collect);
    root = contentDir;
    const legacy = layout === 'legacy';
    const site = await loadSite(source);
    warnings.push(...site.warnings);

    const deep = options.deep ? await deepCheck(site, contentDir) : { errors: [], warnings: [] };
    warnings.push(...deep.warnings);
    const ok = deep.errors.length === 0 && !(options.strict && warnings.length > 0);
    if (options.json) {
      json(ok, deep.errors.map((message) => ({ severity: 'error', code: 'deep', file: null, message })), site);
    } else {
      report(site, contentDir, warnings, legacy, ok);
      for (const error of deep.errors) console.error(`  ✗ ${error}`);
      if (options.strict && warnings.length) console.error(`Strict check failed: ${warnings.length} warning(s).`);
    }
    return { ok, warnings, errors: deep.errors, site };
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    if (options.json) {
      const relative = err instanceof ZellijContentError && err.file
        ? (path.isAbsolute(err.file) ? path.relative(root, err.file) : err.file)
        : undefined;
      const file = relative && !relative.split(path.sep).includes('..') && !path.isAbsolute(relative)
        ? relative.split(path.sep).join('/') : null;
      json(false, [{ severity: 'error', code: 'content', file, message }]);
    } else {
      for (const warning of warnings) console.warn(`  ! ${warning}`);
      console.error('');
      console.error(err instanceof ZellijContentError ? err.message : (err as Error).stack);
    }
    return { ok: false, warnings, errors: [message] };
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
