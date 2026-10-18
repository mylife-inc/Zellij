import path from 'node:path';
import { realpath, stat } from 'node:fs/promises';
import { unified } from 'unified';
import remarkParse from 'remark-parse';
import remarkMdx from 'remark-mdx';
import remarkGfm from 'remark-gfm';
import type { Site } from '../content/site';
import type { AnySection } from '../sections/schemas';
import { siteConfigSchema } from '../config/schema';
import { schemaReferences } from '../content/reference-schema';
import { inlineLinks } from '../content/inline-markdown';
import { ASSET_PUBLIC_PREFIX, assetUrl, normalizeRoute } from '../content/paths';
import { getSection } from '../sections/registry';
import { flattenSidebar } from '../guide/navigation';
import { readAnchorIds } from '../mdx/headings';
import { walkPortable } from '../content/portable-document';
import { loadMaths } from '../mdx/maths';

interface Ast {
  type: string;
  value?: unknown;
  url?: string;
  identifier?: string;
  name?: string | null;
  children?: Ast[];
  attributes?: Array<{ type: string; name?: string; value?: unknown }>;
  position?: { start: { line: number; column: number } };
  data?: { estree?: { body?: Array<{ expression?: { type: string; value?: unknown } }> } };
}
interface Reference {
  kind: 'href' | 'asset' | 'url';
  value: string;
  where: string;
  route?: string;
}
export interface DeepCheckResult { errors: string[]; warnings: string[] }

/** Static analysis only: never evaluate MDX expressions/imports or request remote URLs. */
export async function deepCheck(site: Site, contentDir: string): Promise<DeepCheckResult> {
  const errors: string[] = [];
  const warnings: string[] = [];
  const references: Reference[] = [];
  const renderedExamples = new Set<string>();
  const anchors = new Map<string, Set<string>>([...site.anchors].map(([route, ids]) => [route, new Set(ids)]));
  const incomplete = new Set<string>();
  const [remarkMaths] = await loadMaths();
  const processor = unified().use(remarkParse).use(remarkMdx).use(remarkGfm).use(remarkMaths!);
  const knownComponents = new Set(['Callout', 'Steps', 'Step', 'Tabs', 'Tab', 'CodeGroup', 'Card', 'CardGrid', 'Theorem', 'Proof', 'Screenshot', 'SectionPreview']);

  function addId(id: string, route: string | undefined, where: string): void {
    if (!route) return;
    const ids = anchors.get(route) ?? new Set<string>();
    if (ids.has(id)) errors.push(`${where}: duplicate rendered anchor #${id} on ${route}`);
    ids.add(id);
    anchors.set(route, ids);
  }
  function uncertain(where: string, route: string | undefined, reason: string): void {
    warnings.push(`${where}: ${reason}; generated links/assets/anchors require build/browser verification.`);
    if (route) incomplete.add(route);
  }
  function literal(value: unknown): unknown {
    if (typeof value === 'string') return value;
    if (!value || typeof value !== 'object') return undefined;
    const expression = (value as Ast).data?.estree?.body?.[0]?.expression;
    return expression?.type === 'Literal' ? expression.value : undefined;
  }
  function addSchema(value: unknown, schema: Parameters<typeof schemaReferences>[0], where: string, route?: string, stack: string[] = []): void {
    for (const ref of schemaReferences(schema, value)) {
      const location = `${where} › ${ref.field}`;
      if (ref.kind === 'mdx') scanMdx(ref.value, location, route, stack);
      else if (ref.kind === 'inline') {
        for (const href of inlineLinks(ref.value)) references.push({ kind: 'href', value: href, where: location, route });
      } else references.push({ kind: ref.kind, value: ref.value, where: location, route });
    }
  }
  function addSection(section: AnySection, where: string, route?: string, stack: string[] = []): void {
    const schema = getSection(section.type)?.schema;
    if (schema) addSchema(section, schema, where, route, stack);
  }
  function scanMdx(body: string, where: string, route?: string, stack: string[] = []): void {
    let tree: Ast;
    try {
      tree = processor.parse(body) as Ast;
      for (const id of readAnchorIds(body, { remarkPlugins: remarkMaths })) addId(id, route, where);
    } catch (error) {
      errors.push(`${where}: invalid MDX syntax (body coordinates): ${(error as Error).message}`);
      if (route) incomplete.add(route);
      return;
    }
    const definitions = new Map<string, string>();
    function definitionsIn(node: Ast): void {
      // CommonMark uses the first definition of an identifier.
      if (node.type === 'definition' && !definitions.has(node.identifier!)) definitions.set(node.identifier!, node.url!);
      node.children?.forEach(definitionsIn);
    }
    definitionsIn(tree);
    function visit(node: Ast): void {
      const location = node.position ? `${where} (body ${node.position.start.line}:${node.position.start.column})` : where;
      const image = node.type === 'image' || node.type === 'imageReference';
      if (image || node.type === 'link' || node.type === 'linkReference') {
        const value = node.url ?? definitions.get(node.identifier!);
        if (value !== undefined) references.push({ kind: image ? 'url' : 'href', value, where: location, route });
      }
      if (['mdxjsEsm', 'mdxFlowExpression', 'mdxTextExpression'].includes(node.type)) {
        uncertain(location, route, 'MDX JavaScript is not evaluated');
      }
      if (node.type === 'mdxJsxFlowElement' || node.type === 'mdxJsxTextElement') {
        const custom = node.name && /^[A-Z]/.test(node.name) && !knownComponents.has(node.name);
        if (custom) {
          uncertain(location, route, `custom component <${node.name}> is not evaluated`);
        }
        for (const attribute of node.attributes ?? []) {
          if (custom) continue;
          const value = literal(attribute.value);
          if (attribute.type !== 'mdxJsxAttribute' || (attribute.value !== null && value === undefined)) {
            uncertain(location, route, 'computed JSX attributes are not evaluated');
            continue;
          }
          if (typeof value !== 'string') continue;
          if (attribute.name === 'srcSet' || attribute.name === 'srcset') {
            uncertain(location, route, 'srcSet candidates are not statically checked');
          }
          if (attribute.name === 'id') {
            if (node.name && /^[a-z]/.test(node.name)) addId(value, route, location);
            else uncertain(location, route, 'component id props are not guaranteed to produce HTML IDs');
          }
          if (attribute.name === 'href') references.push({ kind: 'href', value, where: location, route });
          if (attribute.name === 'src' || attribute.name === 'poster') {
            references.push({ kind: node.name === 'Screenshot' ? 'asset' : 'url', value, where: location, route });
          }
          if (node.name === 'SectionPreview' && attribute.name === 'example') {
            const example = site.examples.get(value);
            if (!example) errors.push(`${location}: no section example named ${JSON.stringify(value)}`);
            else if (stack.includes(value)) errors.push(`${location}: recursive section example ${JSON.stringify(value)}`);
            else {
              if (route) renderedExamples.add(value);
              if (example.section.id) addId(example.section.id, route, location);
              addSection(example.section, example.sourceFile, route, [...stack, value]);
            }
          }
        }
      }
      node.children?.forEach(visit);
    }
    visit(tree);
  }

  addSchema(site.config, siteConfigSchema, 'site configuration');
  for (const page of site.pages) {
    page.sections.forEach((section, index) => addSection(section,
      `${page.sectionSources?.[index] ?? page.sourceFile} [${page.route}, section ${index + 1}]`, page.route));
  }
  for (const page of site.guidePages) {
    anchors.set(page.route, new Set());
    if (page.portable) {
      walkPortable(page.portable.body, (node) => {
        if (node.type !== 'element') return;
        const props = node.properties;
        if (typeof props.id === 'string') addId(props.id, page.route, page.sourceFile);
        for (const attr of ['href', 'src'] as const) if (typeof props[attr] === 'string') {
          references.push({ kind: attr === 'href' ? 'href' : 'url', value: props[attr], where: page.sourceFile, route: page.route });
        }
      });
    } else scanMdx(page.body, page.sourceFile, page.route);
  }
  // Unused examples still need valid assets/absolute links. Contextual anchors
  // are checked again where SectionPreview actually renders an example.
  for (const example of site.examples.values()) {
    if (!renderedExamples.has(example.name)) addSection(example.section, example.sourceFile, undefined, [example.name]);
  }

  const redirects = new Map<string, string>();
  if (site.homeRedirect) redirects.set('/', site.homeRedirect);
  for (const guide of site.guides) {
    if (!anchors.has(guide.route)) {
      const first = flattenSidebar(guide.sidebar)[0];
      if (first) redirects.set(guide.route, first.route);
    }
  }
  function destination(route: string): string {
    const seen = new Set<string>();
    while (redirects.has(route) && !seen.has(route)) {
      seen.add(route);
      route = redirects.get(route)!;
    }
    return route;
  }
  const assetRoot = path.resolve(contentDir, 'assets');
  const physicalContent = await realpath(contentDir);
  async function checkAsset(urlPath: string): Promise<string | undefined> {
    if (!urlPath.startsWith(`${ASSET_PUBLIC_PREFIX}/`)) {
      return `local asset URL ${JSON.stringify(urlPath)} is outside ${ASSET_PUBLIC_PREFIX}/; use that public prefix in Markdown/HTML, or assets/... in Screenshot and YAML media`;
    }
    const relative = urlPath.slice(ASSET_PUBLIC_PREFIX.length + 1);
    const file = path.resolve(assetRoot, relative);
    if (!inside(assetRoot, file)) return 'asset path escapes the content assets directory';
    try {
      const [root, target, info] = await Promise.all([realpath(assetRoot), realpath(file), stat(file)]);
      if (!inside(physicalContent, root)) return 'assets directory symlink escapes the content directory';
      if (!inside(root, target)) return 'asset symlink escapes the content assets directory';
      if (!info.isFile()) return `asset ${JSON.stringify(relative)} is not a file`;
    } catch (error) {
      return `cannot read asset ${JSON.stringify(relative)} (${(error as NodeJS.ErrnoException).code ?? 'filesystem error'})`;
    }
    return undefined;
  }
  for (const ref of references) {
    const fail = (message: string) => errors.push(`${ref.where}: ${JSON.stringify(ref.value)} — ${message}`);
    // Remote destinations and embedded data are outside local reference checks.
    if (/^(?:[a-z][a-z\d+.-]*:|\/\/)/i.test(ref.value)) continue;
    if (ref.kind === 'href' && (!ref.value || ref.value === '#')) continue;
    if (!ref.route && ref.kind === 'href' && !ref.value.startsWith('/')) {
      warnings.push(`${ref.where}: page-relative link ${JSON.stringify(ref.value)} has no single page context; not verified.`);
      continue;
    }
    try {
      const url = new URL(ref.kind === 'asset' ? assetUrl(ref.value) : ref.value,
        `https://zellij.invalid${ref.route ?? '/'}`);
      const route = normalizeRoute(decodeURIComponent(url.pathname));
      if (route.includes('\\') || route.includes('\0')) { fail('invalid local URL path'); continue; }
      if (ref.kind !== 'href' || route.startsWith(`${ASSET_PUBLIC_PREFIX}/`)) {
        const problem = await checkAsset(route);
        if (problem) fail(problem);
        continue;
      }
      if (!site.routes.has(route)) { fail(`no page resolves to ${JSON.stringify(route)}`); continue; }
      const target = destination(route);
      const fragment = decodeURIComponent(url.hash.slice(1));
      if (fragment && !anchors.get(target)?.has(fragment)) {
        if (incomplete.has(target)) warnings.push(`${ref.where}: anchor #${fragment} on ${target} cannot be verified because that page contains unevaluated MDX.`);
        else fail(`no anchor #${fragment} on ${target}`);
      }
    } catch (error) { fail(`invalid local URL: ${(error as Error).message}`); }
  }
  return { errors: [...new Set(errors)], warnings: [...new Set(warnings)] };
}

function inside(root: string, file: string): boolean {
  const relative = path.relative(root, file);
  return relative !== '' && relative !== '..' && !relative.startsWith(`..${path.sep}`) && !path.isAbsolute(relative);
}
