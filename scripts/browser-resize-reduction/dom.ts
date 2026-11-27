import { fittingItems } from '../../engine/packages/zellij/src/chrome/nav-overflow';
import { items, moreWidth } from './shared';
const app = document.getElementById('app')!;
app.innerHTML = `<header class="zellij-header" data-solid="false"><div class="zellij-nav-inner" data-align="center"><div class="zellij-nav-company"></div><div class="zellij-nav-content"><a class="zellij-brand" href="#home">Fixture</a><nav class="zellij-nav-desktop" aria-label="Main"><ul class="zellij-nav-list" id="links"></ul><ul class="zellij-nav-list zellij-nav-measure">${[...items, { label: 'More', width: moreWidth }].map(item => `<li style="width:${item.width}px">${item.label}</li>`).join('')}</ul></nav></div><div class="zellij-nav-actions"><button>Search</button><button aria-label="Switch to dark theme">Theme</button></div></div></header>`;
const header = app.querySelector('header')!;
const bar = app.querySelector<HTMLElement>('.zellij-nav-inner')!;
const nav = app.querySelector<HTMLElement>('nav')!;
const links = app.querySelector<HTMLUListElement>('#links')!;
const probes = app.querySelector<HTMLUListElement>('.zellij-nav-measure')!;
let count = -1;
let frame = 0;
function close() {
  const panel = links.querySelector('.zellij-mega-panel');
  if (!panel) return;
  panel.remove();
  header.dataset.solid = 'false';
  links.querySelector<HTMLButtonElement>('[data-nav-index="more"] > button')?.focus({ preventScroll: true });
}
function render(next: number) {
  if (next === count) return;
  const restore = links.contains(document.activeElement);
  count = next;
  links.innerHTML = items.slice(0, count).map((item, i) => `<li data-nav-index="${i}"><a class="zellij-nav-link" data-zellij-navitem href="#${i}" style="width:${item.width}px">${item.label}</a></li>`).join('') + (count < items.length ? `<li data-nav-index="more"><button class="zellij-nav-link" data-zellij-navitem style="width:${moreWidth}px">More</button></li>` : '');
  header.dataset.solid = 'false';
  if (restore) requestAnimationFrame(() => links.querySelector<HTMLElement>('[data-zellij-navitem]')?.focus({ preventScroll: true }));
}
function update() {
  const style = getComputedStyle(bar);
  const edge = app.querySelector('.zellij-nav-actions')!.getBoundingClientRect().width;
  bar.style.setProperty('--zellij-nav-edge', `${edge}px`);
  nav.style.setProperty('--zellij-nav-min-width', `${moreWidth}px`);
  const available = bar.clientWidth - parseFloat(style.paddingLeft) - parseFloat(style.paddingRight) - 2 * edge - 48 - 120 - 24;
  render(fittingItems(items.map(item => item.width), Math.max(0, available - 1), moreWidth, 4));
  nav.dataset.measured = 'true';
}
const observer = new ResizeObserver(() => { cancelAnimationFrame(frame); frame = requestAnimationFrame(update); });
for (const element of [nav, bar, app.querySelector('.zellij-brand')!, app.querySelector('.zellij-nav-company')!, app.querySelector('.zellij-nav-actions')!, ...probes.children]) observer.observe(element);
function open() {
  const more = links.querySelector('[data-nav-index="more"]');
  if (!more || more.querySelector('.zellij-mega-panel')) return;
  more.insertAdjacentHTML('beforeend', `<div class="zellij-mega-panel">${items.slice(count).map((item, i) => `<a href="#${i}">${item.label}</a>`).join('')}</div>`);
  header.dataset.solid = 'true';
  more.querySelector<HTMLAnchorElement>('a')?.focus();
}
links.addEventListener('keydown', event => { if (event.key === 'ArrowDown') { event.preventDefault(); open(); } });
document.addEventListener('keydown', event => { if (event.key === 'Escape') close(); });
document.fonts.ready.then(() => { cancelAnimationFrame(frame); frame = requestAnimationFrame(update); });
update();
