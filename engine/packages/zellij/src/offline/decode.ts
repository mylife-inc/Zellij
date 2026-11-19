import { createElement, Fragment, type ReactNode, type ElementType } from 'react';
import { components } from './registry';
import { assetProps, fileHref } from './location';
// Every collection has an explicit tag; authored objects cannot impersonate an element.
export type Wire = null | boolean | string | number | ['u'] | ['a', Wire[]] |
  ['o', [string, Wire][]] | ['e', string, string | null, Wire];
export function decode(value: Wire): unknown {
  if (!Array.isArray(value)) return value;
  switch (value[0]) {
    case 'u': return undefined;
    case 'a': return value[1].map(decode);
    case 'o': return Object.fromEntries(value[1].map(([key, item]) => [key, decode(item)]));
    case 'e': {
      const name = value[1];
      const type = name === '$fragment' ? Fragment : name.startsWith('$')
        ? components[name.slice(1) as keyof typeof components] : name;
      if (!type) throw new Error(`Unknown offline component: ${name}`);
      let props = decode(value[3]) as Record<string, unknown>;
      props = assetProps(props) as Record<string, unknown>;
      if (typeof type === 'string') {
        for (const key of ['href', 'src', 'poster', 'action']) {
          if (typeof props[key] === 'string') props[key] = fileHref(props[key]);
        }
      }
      if (typeof type === 'string' && typeof props.srcSet === 'string') {
        props.srcSet = props.srcSet.replace(/(^|,\s*)(\/[^,\s]+)/g, (_, prefix: string, url: string) => prefix + fileHref(url));
      }
      return createElement(type as ElementType, { ...props, key: value[2] });
    }
  }
}
export const decodeNode = (value: Wire) => decode(value) as ReactNode;
export const jsonForScript = (value: unknown) => JSON.stringify(value).replace(/</g, '\\u003c')
  .replace(/\u2028/g, '\\u2028').replace(/\u2029/g, '\\u2029');
