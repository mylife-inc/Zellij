import { Children, isValidElement, type ReactElement, type ReactNode } from 'react';
import { ZellijContentError } from '../errors';
import { TabsClient, type TabPanel } from './tabs-client';

interface LabelledProps {
  label?: string;
  title?: string;
  children?: ReactNode;
}

/**
 * Collects labelled children into tab panels on the server, so the client shell
 * never has to introspect props across the RSC boundary.
 */
function collectPanels(children: ReactNode, component: string): TabPanel[] {
  return Children.toArray(children)
    .filter((child): child is ReactElement<LabelledProps> => isValidElement(child))
    .map((child, index) => {
      const { label, title } = child.props;
      const resolved = label ?? title;

      if (!resolved) {
        throw new ZellijContentError(
          `<${component}> child ${index + 1} is missing a "label".\n` +
            `  Example: <${component}><Tab label="CLI">…</Tab></${component}>`,
        );
      }

      return { label: resolved, content: child };
    });
}

export function Tabs({ children }: { children?: ReactNode }) {
  return <TabsClient panels={collectPanels(children, 'Tabs')} />;
}

export function Tab({ children }: LabelledProps) {
  return <>{children}</>;
}

/**
 * Multi-language code tabs. Labels come from each block's `title="…"` meta, so
 * authors write a normal fenced block per language (spec §7.2).
 */
interface CodeProps {
  'data-title'?: string;
  'data-lang'?: string;
}

/**
 * MDX wraps each fenced block in a fragment, so the code element's props sit a
 * level (or more) below CodeGroup's direct children. Descend to find them.
 */
function findCodeProps(node: ReactNode, depth = 0): CodeProps | undefined {
  if (depth > 4 || !isValidElement(node)) return undefined;

  const props = node.props as CodeProps & { children?: ReactNode };
  if (props['data-title'] || props['data-lang']) return props;

  for (const child of Children.toArray(props.children)) {
    const found = findCodeProps(child, depth + 1);
    if (found) return found;
  }

  return undefined;
}

export function CodeGroup({ children }: { children?: ReactNode }) {
  const panels = Children.toArray(children)
    .filter((child): child is ReactElement => isValidElement(child))
    .map((child, index) => {
      const props = findCodeProps(child);
      const label = props?.['data-title'] ?? props?.['data-lang'] ?? `Tab ${index + 1}`;
      return { label, content: child };
    });

  return <TabsClient panels={panels} variant="code" />;
}
