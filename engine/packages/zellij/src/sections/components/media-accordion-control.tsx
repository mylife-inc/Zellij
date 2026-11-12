'use client';

import { useEffect, useId, useRef, useState, type ReactNode } from 'react';
import type { MotionPreset } from '../../config/schema';

/** Server-rendered sequential content is the baseline. Hydration adds selection;
 * CSS alone restores every panel and image in print, including hidden panels. */
export function MediaAccordionControl({ label, items, mediaSide, headingLevel, motion }: {
  label: string; items: { title: string; body: ReactNode; media: ReactNode; caption?: ReactNode }[];
  mediaSide: 'start' | 'end'; headingLevel: number; motion: MotionPreset;
}) {
  const id = useId();
  const [ready, setReady] = useState(false);
  const [selected, setSelected] = useState(0);
  const buttons = useRef<Array<HTMLButtonElement | null>>([]);
  useEffect(() => { setReady(true); }, []);
  const active = Math.min(selected, items.length - 1);
  const Heading = `h${headingLevel}` as 'h2' | 'h3' | 'h4';
  return <div className="zellij-media-accordion" role="group" aria-label={label}>
    <div className="zellij-media-accordion-layout" data-ready={ready} data-media-side={mediaSide} data-motion={motion}>
      <div className="zellij-media-accordion-items">
        {items.map((item, index) => {
          const expanded = !ready || active === index;
          const titleId = `${id}-title-${index}`, panelId = `${id}-panel-${index}`;
          return <div className="zellij-media-accordion-item" key={index}>
            <Heading className="zellij-media-accordion-heading">
              {ready ? <button id={titleId} type="button" ref={node => { buttons.current[index] = node; }}
                aria-expanded={expanded} aria-controls={panelId} aria-disabled={expanded}
                onClick={() => setSelected(index)} onKeyDown={event => {
                  const next = event.key === 'ArrowDown' ? (index + 1) % items.length
                    : event.key === 'ArrowUp' ? (index + items.length - 1) % items.length
                      : event.key === 'Home' ? 0 : event.key === 'End' ? items.length - 1 : undefined;
                  if (next !== undefined) { event.preventDefault(); buttons.current[next]?.focus(); }
                }}>
                <span>{item.title}</span><span className="zellij-media-accordion-sign" aria-hidden="true">{expanded ? '−' : '+'}</span>
              </button> : <span id={titleId}>{item.title}</span>}
            </Heading>
            <div id={panelId} className="zellij-media-accordion-panel" hidden={!expanded}>
              <div className="zellij-media-accordion-body">{item.body}</div>
              <div className="zellij-media-accordion-inline">{item.media}</div>
              {item.caption ? <p className="zellij-media-accordion-caption">{item.caption}</p> : null}
            </div>
          </div>;
        })}
      </div>
      {ready ? <div className="zellij-media-accordion-companion" key={active} role="group" aria-labelledby={`${id}-title-${active}`}>
        {items[active]!.media}
      </div> : null}
    </div>
  </div>;
}
