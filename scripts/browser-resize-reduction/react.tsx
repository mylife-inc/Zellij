import { useCallback, useEffect, useRef, useState } from 'react';
import { createRoot } from 'react-dom/client';
import { useNavOverflow } from '../../engine/packages/zellij/src/chrome/use-nav-overflow';
import { useFocusTrap } from '../../engine/packages/zellij/src/chrome/hooks';
import { items, moreWidth } from './shared';
function Fixture() {
  const nav = useRef<HTMLElement>(null);
  const probes = useRef<HTMLUListElement>(null);
  const links = useRef<HTMLUListElement>(null);
  const panel = useRef<HTMLDivElement>(null);
  const restore = useRef(false);
  const [open, setOpen] = useState(false);
  const close = useCallback(() => setOpen(false), []);
  const changed = useCallback(() => {
    restore.current = Boolean(links.current?.contains(document.activeElement));
    setOpen(false);
  }, []);
  const { count, ready } = useNavOverflow(nav, probes, items.length, changed);
  useFocusTrap(panel, open, close);
  useEffect(() => { if (open) panel.current?.querySelector('a')?.focus(); }, [open]);
  useEffect(() => {
    if (!restore.current) return;
    const frame = requestAnimationFrame(() => {
      restore.current = false;
      const active = document.activeElement;
      if (active !== document.body && active?.getClientRects().length) return;
      links.current?.querySelector<HTMLElement>('[data-zellij-navitem]')?.focus({ preventScroll: true });
    });
    return () => cancelAnimationFrame(frame);
  }, [count]);
  return <header className="zellij-header" data-solid={open}>
    <div className="zellij-nav-inner" data-align="center">
      <div className="zellij-nav-company" />
      <div className="zellij-nav-content">
        <a className="zellij-brand" href="#home">Fixture</a>
        <nav ref={nav} className="zellij-nav-desktop" aria-label="Main" data-measured={ready}>
          <ul ref={links} className="zellij-nav-list">
            {items.slice(0, count).map((item, i) => <li key={i} data-nav-index={i}><a className="zellij-nav-link" data-zellij-navitem href={`#${i}`} style={{ width: item.width }}>{item.label}</a></li>)}
            {count < items.length && <li data-nav-index="more"><button className="zellij-nav-link" data-zellij-navitem style={{ width: moreWidth }} onKeyDown={event => { if (event.key === 'ArrowDown') { event.preventDefault(); setOpen(true); } }}>More</button>
              {open && <div ref={panel} className="zellij-mega-panel" tabIndex={-1}>{items.slice(count).map((item, i) => <a key={i} href={`#${i}`}>{item.label}</a>)}</div>}
            </li>}
          </ul>
          <ul ref={probes} className="zellij-nav-list zellij-nav-measure">{[...items, { label: 'More', width: moreWidth }].map(item => <li key={item.label} style={{ width: item.width }}>{item.label}</li>)}</ul>
        </nav>
      </div>
      <div className="zellij-nav-actions"><button>Search</button><button aria-label="Switch to dark theme">Theme</button></div>
    </div>
  </header>;
}
createRoot(document.getElementById('app')!).render(<Fixture />);
