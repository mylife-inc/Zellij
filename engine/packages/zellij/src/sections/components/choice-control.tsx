'use client';

import { useEffect, useId, useRef, useState, type CSSProperties, type KeyboardEvent, type ReactNode } from 'react';

type Item = { id: string; label: string; heading: string; swatch?: string; body: ReactNode; media?: ReactNode; facts?: ReactNode; cta?: ReactNode };
export function ChoiceControl({ kind, label, presentation, initial, items, mediaSide, headingLevel }: {
  kind: 'selector' | 'explorer'; label: string; presentation: 'segmented' | 'swatches' | 'tabs' | 'list';
  initial?: string; items: Item[]; mediaSide: 'start' | 'end'; headingLevel: number;
}) {
  const baseId = useId();
  const [ready, setReady] = useState(false);
  const [selected, setSelected] = useState(() => initial ?? items[0]?.id);
  const refs = useRef<Array<HTMLInputElement | HTMLButtonElement | null>>([]);
  useEffect(() => { setReady(true); }, []);
  const active = Math.max(0, items.findIndex(item => item.id === selected));
  const vertical = presentation === 'list';
  const Heading = `h${headingLevel}` as 'h2' | 'h3' | 'h4';
  function onKeyDown(event: KeyboardEvent<HTMLElement>, index: number) {
    const rtl = getComputedStyle(event.currentTarget).direction === 'rtl';
    const forward = rtl ? 'ArrowLeft' : 'ArrowRight', backward = rtl ? 'ArrowRight' : 'ArrowLeft';
    const delta = ((kind === 'selector' || !vertical) && event.key === forward) || ((kind === 'selector' || vertical) && event.key === 'ArrowDown') ? 1
      : ((kind === 'selector' || !vertical) && event.key === backward) || ((kind === 'selector' || vertical) && event.key === 'ArrowUp') ? -1 : 0;
    const next = event.key === 'Home' ? 0 : event.key === 'End' ? items.length - 1
      : delta ? (index + delta + items.length) % items.length : undefined;
    if (next === undefined) return;
    event.preventDefault(); setSelected(items[next]!.id); refs.current[next]?.focus();
  }
  return <div className="zellij-compose-choice" data-kind={kind} data-presentation={presentation} data-ready={ready}>
    <p className="zellij-choice-label" id={`${baseId}-label`}>{label}</p>
    <div className="zellij-choice-layout">
      <div className="zellij-choice-controls" hidden={!ready} role={kind === 'selector' ? 'radiogroup' : 'tablist'}
        aria-labelledby={`${baseId}-label`} aria-orientation={kind === 'explorer' ? vertical ? 'vertical' : 'horizontal' : undefined}>
        {items.map((item, index) => {
          const optionId = `${baseId}-option-${item.id}`, panelId = `${baseId}-panel-${item.id}`;
          return kind === 'selector' ? <label className="zellij-choice-option" key={item.id}>
            <input type="radio" name={`${baseId}-selection`} id={optionId} value={item.id} checked={active === index}
              ref={node => { refs.current[index] = node; }} tabIndex={active === index ? 0 : -1}
              aria-controls={panelId} onChange={() => setSelected(item.id)} onKeyDown={event => onKeyDown(event, index)} />
            <span className="zellij-choice-option-face">
              {presentation === 'swatches' ? <span className="zellij-choice-swatch" aria-hidden="true" style={{'--zellij-choice-swatch': item.swatch} as CSSProperties} /> : null}
              <span>{item.label}</span>
            </span>
          </label> : <button type="button" role="tab" className="zellij-choice-tab" key={item.id} id={optionId}
            aria-controls={panelId} aria-selected={active === index} tabIndex={active === index ? 0 : -1}
            ref={node => { refs.current[index] = node; }} onClick={() => setSelected(item.id)}
            onKeyDown={event => onKeyDown(event, index)}>{item.label}</button>;
        })}
      </div>
      <div className="zellij-choice-panels">
        {items.map((item, index) => <div className="zellij-choice-panel" key={item.id} id={`${baseId}-panel-${item.id}`}
          role={ready && kind === 'explorer' ? 'tabpanel' : 'region'}
          aria-labelledby={ready && kind === 'explorer' ? `${baseId}-option-${item.id}` : `${baseId}-heading-${item.id}`}
          hidden={ready && active !== index} tabIndex={ready ? 0 : undefined}>
          <div className="zellij-choice-panel-layout" data-media-side={mediaSide} data-has-media={Boolean(item.media)}>
            <div className="zellij-choice-copy">
              <Heading className="zellij-choice-heading" id={`${baseId}-heading-${item.id}`}>{item.heading}</Heading>
              {item.body}{item.facts}{item.cta ? <div className="zellij-compose-action">{item.cta}</div> : null}
            </div>
            {item.media}
          </div>
        </div>)}
      </div>
    </div>
  </div>;
}
