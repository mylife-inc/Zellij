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
