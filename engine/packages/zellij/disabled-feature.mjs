// A disabled engine loader is never invoked by the guarded renderer.
// Keep these exports compatible without importing any optional peer package.
function disabledFeature() {
  throw new Error('A disabled Zellij feature was invoked; enable it with zel features and rebuild.');
}
export const loadMathLibraries = disabledFeature;
export const loadMermaid = disabledFeature;
