import { SEARCH_INDEX_PATH } from '../content/paths';
import type { SearchEntry } from './build-index';
let offline: Promise<SearchEntry[]> | undefined;
/** A classic-script transport for file exports; HTTP sites retain their lazy fetch. */
export async function loadSearchIndex(basePath: string, signal: AbortSignal): Promise<SearchEntry[]> {
  const runtime = document.querySelector<HTMLScriptElement>('script[data-zellij-file-runtime]');
  if (runtime) {
    if (!offline) offline = new Promise((resolve, reject) => {
      const script = document.createElement('script');
      script.src = new URL('search-index.js', runtime.src).href;
      script.onload = () => resolve((window as unknown as { __zellijSearchIndex: SearchEntry[] }).__zellijSearchIndex);
      script.onerror = () => { offline = undefined; reject(new Error('The bundled search index could not be loaded')); };
      document.head.append(script);
    });
    return offline;
  }
  const response = await fetch(`${basePath}${SEARCH_INDEX_PATH}`, { signal,
    ...(process.env.NODE_ENV === 'development' ? { cache: 'no-store' as const } : {}) });
  if (!response.ok) throw new Error(`index request failed: ${response.status}`);
  return response.json() as Promise<SearchEntry[]>;
}
