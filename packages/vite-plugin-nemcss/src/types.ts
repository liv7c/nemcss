/**
 * Options for the nemcss Vite plugin
 */
export interface NemcssPluginOptions {
  /**
   * Path to nemcss.config.json, relative to Vite's root
   * @default nemcss.config.json
   */
  configPath?: string;
  /**
   * Enable Hot Module Replacement (HMR) for CSS changes
   * @default true
   */
  hmr?: boolean;
}
