// Shared by Next's plain-JS config and the engine's native Node commands.
const fs = require('node:fs');
const path = require('node:path');
const { createRequire } = require('node:module');
const FEATURE_PACKAGES = {
  maths: { 'remark-math': '^6.0.0', 'rehype-mathjax': '^7.1.0' },
  diagrams: { mermaid: '^11.17.2' },
};
function selectedFeatures(manifest) {
  if (manifest === null || typeof manifest !== 'object' || Array.isArray(manifest) ||
      (manifest.zellij !== undefined && (manifest.zellij === null || typeof manifest.zellij !== 'object' || Array.isArray(manifest.zellij)))) {
    throw new Error('package.json and its zellij settings must be objects');
  }
  const selection = manifest.zellij?.features;
  if (selection === undefined) return undefined; // Legacy wrappers retain automatic detection.
  if (!Array.isArray(selection) || selection.some(name => typeof name !== 'string' || !Object.hasOwn(FEATURE_PACKAGES, name)) || new Set(selection).size !== selection.length) {
    throw new Error('package.json zellij.features must be an array containing maths and/or diagrams once each; [] disables both. Run zel features to set it.');
  }
  return selection;
}
function manifestAt(appRoot = process.cwd()) {
  try { return JSON.parse(fs.readFileSync(path.join(appRoot, 'package.json'), 'utf8')); }
  catch (error) { if (error.code === 'ENOENT') return {}; throw error; }
}
function featureEnabled(name, appRoot = process.cwd()) {
  // Statically named access is required for Next's build-time substitution.
  const compiled = name === 'maths' ? process.env.ZELLIJ_MATHS : process.env.NEXT_PUBLIC_ZELLIJ_DIAGRAMS;
  if (compiled !== undefined) return compiled === 'true';
  const selection = selectedFeatures(manifestAt(appRoot));
  return selection === undefined || selection.includes(name);
}
function resolveFeatures(appRoot = process.cwd()) {
  const selection = selectedFeatures(manifestAt(appRoot));
  const resolve = createRequire(path.join(appRoot, 'package.json')).resolve;
  const result = {};
  for (const [name, packages] of Object.entries(FEATURE_PACKAGES)) {
    const installed = Object.keys(packages).every(pkg => { try { resolve(pkg); return true; } catch { return false; } });
    if (selection?.includes(name) && !installed) {
      throw new Error(`Zellij feature ${name} is selected but its packages are missing. Run npm install in ${appRoot}, then restart or rebuild.`);
    }
    result[name] = selection === undefined ? installed : selection.includes(name);
  }
  return result;
}
module.exports = { FEATURE_PACKAGES, selectedFeatures, featureEnabled, resolveFeatures };
