import * as lucide from 'lucide-react';
import type { ComponentType, SVGProps } from 'react';
import { ZellijContentError } from './errors';

type IconComponent = ComponentType<SVGProps<SVGSVGElement> & { size?: number | string }>;

/** `bar-chart-3` → `BarChart3`, matching lucide's export names. */
function toPascalCase(name: string): string {
  return name
    .split(/[-_\s]+/)
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join('');
}

const iconCache = new Map<string, IconComponent>();

/**
 * lucide icons are `forwardRef` components, which are objects rather than
 * functions — so a `typeof === 'function'` check would reject every valid icon.
 * Accept anything React can render as a component.
 */
function isComponent(value: unknown): boolean {
  if (typeof value === 'function') return true;
  return (
    typeof value === 'object' &&
    value !== null &&
    '$$typeof' in (value as Record<string, unknown>)
  );
}

/**
 * Resolves a kebab-case icon name to a lucide component. Unknown names are a
 * build error rather than a silently missing glyph (spec §5.2).
 *
 * Icons resolve on the server and render to elements passed into client
 * components, so lucide never reaches the client bundle.
 */
export function resolveIcon(name: string, context: string): IconComponent {
  const cached = iconCache.get(name);
  if (cached) return cached;

  const exportName = toPascalCase(name);
  const candidate = (lucide as unknown as Record<string, unknown>)[exportName];

  if (!isComponent(candidate)) {
    throw new ZellijContentError(
      `Unknown icon "${name}" in ${context}.\n` +
        `  Expected a lucide-react icon (looked for the "${exportName}" export).\n` +
        `  Browse valid names at https://lucide.dev/icons — use the kebab-case name, e.g. "shield-check".`,
    );
  }

  const component = candidate as IconComponent;
  iconCache.set(name, component);
  return component;
}

export function Icon({
  name,
  context,
  className,
  size = 20,
}: {
  name: string;
  context: string;
  className?: string;
  size?: number;
}) {
  const Component = resolveIcon(name, context);
  return <Component className={className} size={size} aria-hidden="true" focusable="false" />;
}
