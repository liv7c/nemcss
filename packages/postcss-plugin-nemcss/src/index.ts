import { resolve } from "node:path";
import { readFileSync } from "node:fs";
import fg from "fast-glob";
import postcss from "postcss";
import type { AtRule, Plugin, PluginCreator } from "postcss";
import type { NemcssPluginOptions } from "./types";
import { extractClasses, generateCss, GeneratedCss } from "@nemcss/napi";

const NEMCSS_CONFIG_FILE = `nemcss.config.json`;
const DEFAULT_IGNORE = ["**/node_modules/**", "**/dist/**"];

export const nemcss: PluginCreator<NemcssPluginOptions> = function (
  options: NemcssPluginOptions = {},
): Plugin {
  return {
    postcssPlugin: "postcss-plugin-nemcss",
    async Once(root, { result }) {
      const configPath = resolve(
        process.cwd(),
        options.configPath ?? NEMCSS_CONFIG_FILE,
      );

      let baseDirective: AtRule | undefined;
      let utilitiesDirective: AtRule | undefined;

      root.walkAtRules("nemcss", (rule) => {
        if (rule.params === "base") {
          baseDirective = rule;
        } else if (rule.params === "utilities") {
          utilitiesDirective = rule;
        }
      });

      if (!baseDirective && !utilitiesDirective) return;

      // attach a message to result to signify to runners they should rebuild css when config file changes
      result.messages.push({
        type: "dependency",
        plugin: "postcss-plugin-nemcss",
        file: configPath,
        parent: result.opts.from,
      });

      let config;
      try {
        config = JSON.parse(readFileSync(configPath, "utf8"));
      } catch (err) {
        result.warn(`nemcss: could not read config at ${configPath}: ${err}`);
        return;
      }

      for (const category of Object.values(config.theme ?? {}) as Array<{
        source?: string;
      }>) {
        if (!category.source) continue;
        // attach a message to result to signify to runners they should rebuild css when a token file gets modified
        result.messages.push({
          type: "dependency",
          plugin: "postcss-plugin-nemcss",
          file: resolve(process.cwd(), category.source),
          parent: result.opts.from,
        });
      }

      const contentGlobs: string[] = config.content ?? [];
      for (const glob of contentGlobs) {
        result.messages.push({
          type: "dir-dependency",
          plugin: "postcss-plugin-nemcss",
          dir: process.cwd(),
          glob,
          parent: result.opts.from,
        });
      }

      const files = await fg(contentGlobs, {
        cwd: process.cwd(),
        absolute: true,
        ignore: [...DEFAULT_IGNORE, ...(options.ignore ?? [])],
      });

      const allClasses = new Set<string>();
      for (const file of files) {
        try {
          const content = readFileSync(file, "utf8");
          const classes = extractClasses(content);
          for (const cls of classes) allClasses.add(cls);
        } catch (err) {
          result.warn(`nemcss: could not read file ${file}: ${err}`);
        }
      }

      let css: GeneratedCss;
      try {
        css = generateCss(configPath, [...allClasses]);
      } catch (err) {
        result.warn(`nemcss: CSS generation failed: ${err}`);
        return;
      }

      if (baseDirective) {
        const newNodes = postcss.parse(css.baseCss, {
          from: result.opts.from,
        });
        baseDirective.replaceWith(newNodes);
      }

      if (utilitiesDirective) {
        const newNodes = postcss.parse(css.utilitiesCss, {
          from: result.opts.from,
        });
        utilitiesDirective.replaceWith(newNodes);
      }
    },
  };
};

nemcss.postcss = true;

export default nemcss;
export type { NemcssPluginOptions } from "./types.js";
