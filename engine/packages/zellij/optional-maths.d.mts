import type { Pluggable } from 'unified';
export function loadMathLibraries(): Promise<[{ default: Pluggable }, { default: Pluggable }]>;
