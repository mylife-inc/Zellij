// Classic scripts can be loaded from file:// without module/fetch CORS exceptions.
let pending: Promise<any> | undefined;
export async function loadMermaid() {
  if (!pending) pending = new Promise((resolve, reject) => {
    const current = document.querySelector<HTMLScriptElement>('script[data-zellij-file-runtime]');
    if (!current?.src) { reject(new Error('Offline runtime URL is missing')); return; }
    const script = document.createElement('script');
    script.src = new URL('mermaid.js', current.src).href;
    script.onload = () => resolve((window as any).__zellijMermaid);
    script.onerror = () => { pending = undefined; reject(new Error('The bundled diagram library could not be loaded')); };
    document.head.append(script);
  });
  return pending;
}
