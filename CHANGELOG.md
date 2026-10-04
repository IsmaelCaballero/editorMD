# Changelog

Todos los cambios relevantes de este proyecto se documentan aquí.

El formato sigue [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/)
y el proyecto usa [Semantic Versioning 2.0.0](https://semver.org/lang/es/).

## [Unreleased]

### Added
- Interfaces MVC (`IEditorView`, `IBackend`, `IDialogService`).
- M01 `DocumentState`: estado observable del documento (cambios sin guardar, ruta, codificación, fin de línea).
- M07 `MarkdownCodec`: conversión Markdown ⇄ árbol fiel (GFM, front matter, R Markdown) con estilo canónico y corpus de fidelidad.
- Editor WYSIWYG con Milkdown *kit* (V02): deshacer/rehacer, pegado de Markdown, front matter y bloques R protegidos.
- Barra de menús completa (V01); las opciones de fases futuras aparecen deshabilitadas con la fase prevista.
- Contador de palabras en la barra de estado.
- Linters y formateadores: ESLint + Prettier (TS/Svelte), rustfmt + clippy pedantic (Rust), EditorConfig.
- Job `lint` en la CI.

### Changed
- Código existente formateado con Prettier; `build.rs` documentado.

## [0.0.1] - 2026-10-04

Esqueleto de la fase F0 ([PR #1](https://github.com/IsmaelCaballero/editorMD/pull/1)).

### Added
- Planificación inicial: PLAN.md, STATUS.md, CONTEXT.md, licencia MIT.
- Esqueleto de la aplicación: Tauri 2 (Rust) + Vite + Svelte 5 + TypeScript.
- M00 `AppInfo` (Rust) y comando `app_info`.
- M17 `SemVer`: implementación de Semantic Versioning 2.0.0 con tests.
- Documentación de la API con TypeDoc (`npm run docs:ts`) y rustdoc (`npm run docs:rust`).
- Web del proyecto `docs/index.html` (plan, diario de pasos, ramas, componentes).
- `.gitattributes` con normalización LF y fixtures binarios.
- CI con GitHub Actions: Windows, Ubuntu 22.04, macOS y Fedora.
- `Cargo.lock` versionado.

### Changed
- `Cargo.toml`: `features = []` explícito en `tauri` y `tauri-build` (lo sincroniza `tauri dev`).

[Unreleased]: https://github.com/IsmaelCaballero/editorMD/compare/v0.0.1...HEAD
[0.0.1]: https://github.com/IsmaelCaballero/editorMD/releases/tag/v0.0.1
