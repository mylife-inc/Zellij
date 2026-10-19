export const FEATURE_PACKAGES: Record<string, Record<string, string>>;
export function selectedFeatures(manifest: { zellij?: { features?: unknown } }): string[] | undefined;
export function featureEnabled(name: 'maths' | 'diagrams', appRoot?: string): boolean;
export function resolveFeatures(appRoot?: string): { maths: boolean; diagrams: boolean };
