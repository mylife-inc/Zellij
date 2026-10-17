export interface ContentLocationOptions {
  appRoot?: string;
  contentDir?: string;
  configFile?: string;
}
export function resolveContentPaths(options?: ContentLocationOptions): { appRoot: string; configFile: string };
export function resolveContentLocation(options?: ContentLocationOptions): {
  appRoot: string;
  configFile: string;
  contentDir: string;
  layout: 'folder' | 'legacy';
  basePath: string | undefined;
  legacyConfigUsed: boolean;
};
export function normalizeDeploymentBasePath(value: string | undefined): string;
export function resolveDeploymentBasePath(...values: (string | undefined)[]): string;
