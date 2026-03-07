# @nemcss/napi

Native Node.js bindings (NAPI) for nemcss. Used internally by `@nemcss/vite` and `@nemcss/postcss`. You generally don't need to install this directly.

## Exported API

```ts
// Extract nemcss utility class names found in a string of content
extractClasses(content: string): string[]

// Generate CSS from a nemcss config file, optionally filtered to a set of used classes
generateCss(configPath: string, usedClasses?: string[]): string
```

For more information, see the [root README](../../README.md).
