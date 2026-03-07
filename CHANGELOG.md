## [0.1.0] - 2026-03-07

### 🚀 Features

- Create cli and init command (#2)
- Create engine crate to generate css (#3)
- *(engine)* Add benchmark tests (#4)
- *(engine)* Support responsive variants (#6)
- Add class extractor crate (#7)
- Add build command (#8)
- *(cli)* Add benchmarks and concurrency build command (#9)
- *(cli)* Add watch command (#10)
- *(lsp)* Create lsp for nemcss with basic completion (#12)
- *(lsp)* Register capabilities to rebuild cache on config or tokens change (#13)
- *(lsp)* Add hover support and context detection (#14)
- *(lsp)* Add autocomplete hover for custom properties (#15)
- *(editors/vscode)* Add vscode extension for lsp (#22)
- *(packages)* Add vite plugin for nemcss (#25)
- *(plugin-postcss)* Add postcss plugin (#29)
- *(npm)* Add npm wrapper for nemcss CLI and scope plugin packages (#30)
- Make adding utilities explicit with semantic layer (#34)

### 🐛 Bug Fixes

- *(cli)* Fix benchmark rand trait after rand update (#21)
- *(lsp)* Fix prefix included in completion in vscode (#23)
- *(lsp)* Fix completion after trigger char in vscode (#24)

### 🚜 Refactor

- *(cli)* Improvements before first release (#33)
