import type { ZodError } from 'zod';

/**
 * Every build-time failure in Zellij surfaces as one of these. Content authors
 * see the file, the path inside it, and what was expected — never a stack trace
 * from deep inside a parser (spec §3).
 */
export class ZellijContentError extends Error {
  override readonly name = 'ZellijContentError';

  constructor(
    message: string,
    readonly file?: string,
  ) {
    super(message);
  }
}

/** Renders a zod issue path as dotted/bracketed notation: `sections[2].cta[0].href`. */
function formatPath(path: ReadonlyArray<PropertyKey>): string {
  if (path.length === 0) return '(root)';
  return path.reduce<string>((acc, segment) => {
    if (typeof segment === 'number') return `${acc}[${segment}]`;
    return acc === '' ? String(segment) : `${acc}.${String(segment)}`;
  }, '');
}

/**
 * Turns a ZodError into a human-readable, multi-line build error naming the
 * file, the path, and the expectation for each issue.
 */
export function formatZodError(file: string, error: ZodError): ZellijContentError {
  const issues = error.issues.map((issue) => {
    const where = formatPath(issue.path);
    return `  · ${where}: ${issue.message}`;
  });

  const header =
    issues.length === 1
      ? `Invalid content in ${file}:`
      : `Invalid content in ${file} (${issues.length} problems):`;

  return new ZellijContentError(`${header}\n${issues.join('\n')}`, file);
}

/**
 * Aggregates many errors into a single failure so authors fix everything in one
 * pass instead of rebuilding once per mistake (spec §13.3).
 */
export function aggregateErrors(
  headline: string,
  errors: readonly ZellijContentError[],
): ZellijContentError {
  const body = errors.map((e) => e.message).join('\n\n');
  return new ZellijContentError(`${headline}\n\n${body}`);
}
