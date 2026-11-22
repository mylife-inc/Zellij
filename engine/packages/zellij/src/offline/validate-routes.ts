/** Refuse routes that cannot map safely to the same files on supported systems. */
export function validateOfflineRoutes(routes: Iterable<string>) {
  const seen = new Set<string>();
  for (const route of routes) {
    if (route !== '/' && (!route.startsWith('/') || route.slice(1).split('/').some(part =>
      !part || part === '.' || part === '..' || /[\\<>:"|?*#%\x00-\x1f\x7f]/.test(part) ||
      /[. ]$/.test(part) || /^(con|prn|aux|nul|com[1-9]|lpt[1-9])(?:\.|$)/i.test(part)))) {
      throw new Error(`Unsafe or non-portable offline route: ${route}`);
    }
    const identity = route.normalize('NFC').toLowerCase();
    if (seen.has(identity)) throw new Error(`Offline routes collide on case-insensitive filesystems: ${route}`);
    seen.add(identity);
  }
}

/** Flat HTTP exports must not overwrite index/404 or turn an HTML file into a directory. */
export function validateHttpRoutes(routes: Iterable<string>) {
  const list = [...routes];
  validateOfflineRoutes(list);
  const files = new Set(['404.html']);
  for (const route of list) {
    const file = (route === '/' ? 'index.html' : route.slice(1) + '.html').normalize('NFC').toLowerCase();
    if (file.startsWith('_zellij/') || file === '_zellij.html' || files.has(file)) throw new Error(`HTTP output routes collide: ${route}`);
    files.add(file);
  }
  for (const file of files) {
    const parts = file.split('/');
    for (let i = 1; i < parts.length; i++) {
      if (files.has(parts.slice(0, i).join('/'))) throw new Error(`HTTP output file/directory collision: ${file}`);
    }
  }
}
