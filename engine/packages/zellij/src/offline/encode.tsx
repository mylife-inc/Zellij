import { Fragment, Suspense, isValidElement, createElement, type ElementType } from 'react';
import { renderToString } from 'react-dom/server';
import { components } from './registry';
import type { Wire } from './decode';
const names = new Map<unknown, string>(Object.entries(components).map(([key, value]) => [value, `$${key}`]));

/** Evaluate server components at build time and preserve the shared client boundaries. */
export async function encode(value: unknown): Promise<Wire> {
  let serverId = 0;
  async function visit(value: unknown): Promise<Wire> {
  if (value === undefined) return ['u'];
  if (value === null || typeof value === 'boolean' || typeof value === 'string') return value;
  if (typeof value === 'number' && Number.isFinite(value)) return value;
  if (Array.isArray(value)) return ['a', await Promise.all(value.map(visit))];
  if (isValidElement<Record<string, unknown>>(value)) {
    const { type, key, props } = value;
    const name = names.get(type);
    if (name || typeof type === 'string' || type === Fragment) {
      return ['e', name ?? (type === Fragment ? '$fragment' : type as string), key, await visit(props)];
    }
    if (type === Suspense) return visit(props.children);
    let result: unknown;
    // Give synchronous server components a real React hook dispatcher (useId,
    // context), but capture their tree before SSR traverses async descendants.
    function Capture() {
      if (typeof type === 'function') result = (type as (props: unknown) => unknown)(props);
      else if (type && typeof type === 'object' && 'render' in type) {
        result = (type as { render: (props: unknown, ref: null) => unknown }).render(props, null);
      } else if (type && typeof type === 'object' && 'type' in type) {
        result = createElement((type as { type: ElementType }).type, props);
      } else throw new Error(`Unsupported offline server component: ${String(type)}`);
      return null;
    }
    renderToString(<Capture />, { identifierPrefix: `zel-server-${serverId++}-` });
    return visit(await result);
  }
  if (typeof value === 'object' && Object.getPrototypeOf(value) === Object.prototype) {
    return ['o', await Promise.all(Object.entries(value).map(async ([key, item]) => [key, await visit(item)] as [string, Wire]))];
  }
  throw new Error(`Offline component props must be serializable; received ${typeof value}`);
}

  return visit(value);
}
