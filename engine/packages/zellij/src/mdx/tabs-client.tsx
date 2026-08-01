'use client';

import { useId, useRef, useState, type ReactNode } from 'react';

export interface TabPanel {
  label: string;
  content: ReactNode;
}

/**
 * Shared implementation behind <Tabs> and <CodeGroup>. Follows the ARIA tabs
 * pattern: roving tabindex, arrow/Home/End navigation (spec §11.7).
 */
export function TabsClient({
  panels,
  variant = 'default',
}: {
  panels: TabPanel[];
  variant?: 'default' | 'code';
}) {
  const [active, setActive] = useState(0);
  const baseId = useId();
  const tabRefs = useRef<Array<HTMLButtonElement | null>>([]);

  if (panels.length === 0) return null;

  const focusTab = (index: number): void => {
    const next = (index + panels.length) % panels.length;
    setActive(next);
    tabRefs.current[next]?.focus();
  };

  const onKeyDown = (event: React.KeyboardEvent<HTMLButtonElement>, index: number): void => {
    switch (event.key) {
      case 'ArrowRight':
        event.preventDefault();
        focusTab(index + 1);
        break;
      case 'ArrowLeft':
        event.preventDefault();
        focusTab(index - 1);
        break;
      case 'Home':
        event.preventDefault();
        focusTab(0);
        break;
      case 'End':
        event.preventDefault();
        focusTab(panels.length - 1);
        break;
      default:
        break;
    }
  };

  return (
    <div className="zellij-tabs" data-variant={variant}>
      <div className="zellij-tablist" role="tablist">
        {panels.map((panel, index) => (
          <button
            key={panel.label}
            type="button"
            role="tab"
            id={`${baseId}-tab-${index}`}
            aria-controls={`${baseId}-panel-${index}`}
            aria-selected={index === active}
            tabIndex={index === active ? 0 : -1}
            ref={(node) => {
              tabRefs.current[index] = node;
            }}
            className="zellij-tab"
            onClick={() => setActive(index)}
            onKeyDown={(event) => onKeyDown(event, index)}
          >
            {panel.label}
          </button>
        ))}
      </div>

      {panels.map((panel, index) => (
        <div
          key={panel.label}
          role="tabpanel"
          id={`${baseId}-panel-${index}`}
          aria-labelledby={`${baseId}-tab-${index}`}
          className="zellij-tabpanel"
          hidden={index !== active}
          tabIndex={0}
        >
          {panel.content}
        </div>
      ))}
    </div>
  );
}
