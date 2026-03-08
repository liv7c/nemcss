import type { Plugin, ResolvedConfig, ViteDevServer, HmrContext } from "vite";
import { createFilter } from "vite";
import type { NemcssPluginOptions } from "./types";
import { resolve, sep } from "node:path";
import { readFileSync } from "node:fs";
import fg from "fast-glob";

import { extractClasses, generateCss, GeneratedCss } from "@nemcss/napi";

const BASE_DIRECTIVE_RE = /@nemcss base;/g;
const BASE_DIRECTIVE = "@nemcss base;";
const UTILITIES_DIRECTIVE_RE = /@nemcss utilities;/g;
const UTILITIES_DIRECTIVE = "@nemcss utilities;";

/**
 * Extracts the non-glob prefix directory from a glob pattern so we can
 * register a directory watch with chokidar (mirrors the CLI's extract_watch_dirs).
 *
 * Examples:
 *   "src/**\/*.svelte"  →  "src"
 *   "templates/*.html"  →  "templates"
 *   "index.html"        →  "index.html"  (no wildcards — watch the file itself)
 */
function extractBaseDir(pattern: string): string {
  const parts = pattern.split("/");
  const base: string[] = [];
  for (const part of parts) {
    if (
      part.includes("*") ||
      part.includes("?") ||
      part.includes("{") ||
      part.includes("[")
    ) {
      break;
    }
    base.push(part);
  }
  return base.length > 0 ? base.join("/") : ".";
}

export function nemcss(options: NemcssPluginOptions = {}): Plugin {
  let configPath: string;
  let viteConfig: ResolvedConfig;
  let contentGlobs: string[] = [];
  let tokensDirAbs: string = "";
  let generatedCss: GeneratedCss = { baseCss: "", utilitiesCss: "" };
  let isContentFile: (id: string) => boolean = () => false;
  let server: ViteDevServer | undefined;

  const nemcssStylesheets = new Set<string>();

  /**
   * Reads nemcss.config.json and updates contentGlobs, tokensDirAbs,
   * and the isContentFile matcher. Safe to call on config-file changes.
   */
  function loadNemcssConfig() {
    try {
      const raw = JSON.parse(readFileSync(configPath, "utf8"));
      contentGlobs = raw.content ?? [];
      const tokensDir = raw.tokensDir ?? "design-tokens";
      tokensDirAbs = resolve(viteConfig.root, tokensDir);
    } catch (e) {
      viteConfig.logger.error(`Error reading ${configPath}: ${e}`);
      contentGlobs = [];
      tokensDirAbs = resolve(viteConfig.root, "design-tokens");
    }

    isContentFile =
      contentGlobs.length > 0
        ? createFilter(contentGlobs, undefined, { resolve: viteConfig.root })
        : () => false;
  }

  /**
   * Adds the tokens dir and every content base dir to the watcher.
   * Safe to call multiple times — chokidar ignores duplicate paths.
   */
  function addWatches(watcher: ViteDevServer["watcher"]) {
    watcher.add(tokensDirAbs);
    for (const pattern of contentGlobs) {
      watcher.add(resolve(viteConfig.root, extractBaseDir(pattern)));
    }
  }

  async function rebuild() {
    const files = await fg(contentGlobs, {
      cwd: viteConfig.root,
      absolute: true,
      ignore: ["**/node_modules/**", "**/dist/**"],
    });

    const allClasses = new Set<string>();
    for (const file of files) {
      try {
        const content = readFileSync(file, "utf8");
        const classes = extractClasses(content);
        for (const cls of classes) allClasses.add(cls);
      } catch (e) {
        viteConfig.logger.error(`Error reading ${file}: ${e}`);
      }
    }

    viteConfig.logger.info(
      `scanned ${files.length} files, found ${allClasses.size} unique classes`,
    );

    try {
      generatedCss = generateCss(configPath, [...allClasses]);
    } catch (e) {
      viteConfig.logger.error(`Error generating CSS: ${e}`);
    }
  }

  return {
    name: "vite-plugin-nemcss",
    enforce: "pre" as const,
    configResolved(this: void, resolvedConfig: ResolvedConfig) {
      viteConfig = resolvedConfig;
      configPath = resolve(
        viteConfig.root,
        options.configPath ?? "nemcss.config.json",
      );
      loadNemcssConfig();
    },
    configureServer(this: void, devServer: ViteDevServer) {
      server = devServer;
      if (options.hmr === false) return;
      // Watch the tokens directory and all content base directories so that
      // handleHotUpdate fires even for files outside Vite's root (mirrors the
      // CLI's watcher.watch(dir) calls in create_debounced_watcher).
      addWatches(devServer.watcher);
    },
    async buildStart(this) {
      this.addWatchFile(configPath);
      await rebuild();
    },
    async handleHotUpdate(this: void, { file, modules, server }: HmrContext) {
      if (options.hmr === false) return;

      const isConfig = file === configPath;
      const isToken =
        tokensDirAbs.length > 0 && file.startsWith(tokensDirAbs + sep);
      const isContent = isContentFile(file);

      if (!isConfig && !isToken && !isContent) return;

      if (isConfig) {
        viteConfig.logger.info("nemcss: config changed, reloading...");
        loadNemcssConfig();
        // Re-register watches: tokensDir or content globs may have changed.
        addWatches(server.watcher);
      }

      await rebuild();

      // Invalidate all nemcss CSS modules so they get re-transformed with the
      // new generatedCss on the next request.
      const cssModules = [];
      for (const id of nemcssStylesheets) {
        const mod = server.moduleGraph.getModuleById(id);
        if (mod) {
          server.moduleGraph.invalidateModule(mod);
          cssModules.push(mod);
        }
      }

      if (cssModules.length === 0) return;

      // For content file changes, preserve the file's own modules so that
      // component HMR (Svelte, React, etc.) still works alongside CSS updates.
      if (isContent) {
        return [...modules, ...cssModules];
      }

      return cssModules;
    },
    transform(this: void, code: string, id: string) {
      if (!id.endsWith(".css")) return;

      const hasBaseDirective = code.includes("@nemcss base;");
      const hasUtilitiesDirective = code.includes("@nemcss utilities;");
      if (!hasBaseDirective && !hasUtilitiesDirective) {
        nemcssStylesheets.delete(id);
        return;
      }

      nemcssStylesheets.add(id);

      let result = code;
      if (hasBaseDirective) {
        result = result.replace(BASE_DIRECTIVE_RE, () => generatedCss.baseCss);
      }
      if (hasUtilitiesDirective) {
        result = result.replace(
          UTILITIES_DIRECTIVE_RE,
          () => generatedCss.utilitiesCss,
        );
      }

      return {
        code: result,
        map: null,
      };
    },
  };
}

export type { NemcssPluginOptions } from "./types.js";
