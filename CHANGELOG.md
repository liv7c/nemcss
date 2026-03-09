## [0.2.1] - 2026-03-10

### ⬆️ Dependencies

- *(lsp)* Bump tokio to 1.50.0 (runtime and sync bug fixes)

### 📚 Documentation

- Improve npm package README

## [0.2.0] - 2026-03-08

### 🚀 Features

- *(engine)* Split to_css into base_to_css and utilities_to_css
- *(engine)* Remove to_css method
- *(napi)* Use new base_to_css and utilities_to_css methods
- *(cli)* Use split base_to_css and utilities_to_css methods
- *(postcss)* Split base and utilities css generation
- *(vite-plugin)* Update plugin to support both base and utilities nemcss directives

### 🐛 Bug Fixes

- Address pre-release review issues
