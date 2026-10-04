# Changelog

Todos los cambios relevantes de este proyecto se documentan aquí.

El formato sigue [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/)
y el proyecto usa [Semantic Versioning 2.0.0](https://semver.org/lang/es/).

## [Unreleased]

### Added
- M09 `TextCodec` (Rust): detección automática de la codificación (BOM, UTF-8 válido, heurística `chardetng`, Windows-1252 por defecto) y decodificación estricta de UTF-8 (±BOM), UTF-16 LE/BE, ASCII, ISO-8859-1, ISO-8859-15, Windows-1252 y Mac Roman, con la posición exacta del primer byte no válido.
- M09 `TextCodec`: escritura en las 9 codificaciones (con BOM), detección del fin de línea predominante y aviso de finales mezclados; al guardar se conservan la codificación y el fin de línea originales (ida y vuelta byte a byte).
- M09 `TextCodec`: informe de los caracteres que no caben en la codificación destino (con sus líneas) y conversión con sustitución por `?`, transliteración (`€` → `EUR`) o entidades HTML.
- M08 `FileService` (Rust): lectura de ficheros con aviso de solo lectura, **escritura atómica** (fichero temporal en la misma carpeta, volcado a disco y renombrado: si algo falla, el original queda intacto y no queda basura) y comandos Tauri `open_file` / `save_file`, que combinan M08 con M09. Guardar con caracteres que no caben en la codificación devuelve el informe de pérdidas en vez de perderlos en silencio.

## [0.0.2] - 2026-10-04

Fase F1 · esqueleto MVC con editor WYSIWYG ([PR #5](https://github.com/IsmaelCaballero/editorMD/pull/5) a [PR #9](https://github.com/IsmaelCaballero/editorMD/pull/9)).
Los linters de F0.7 ([PR #3](https://github.com/IsmaelCaballero/editorMD/pull/3)) también se publican en esta versión.

### Added
- **Editor WYSIWYG** con Milkdown *kit* (V02): deshacer/rehacer, pegado de Markdown; el front matter YAML y las cabeceras de los bloques de R Markdown se conservan intactos.
- **Menús** completos (V01): las opciones de fases futuras aparecen deshabilitadas con la fase prevista.
- Archivo → **Nuevo** (Ctrl+N / ⌘N), con aviso de cambios sin guardar; Editar → **Deshacer/Rehacer**; Ayuda → **Acerca de** (versión obtenida del backend Rust).
- **Título** de la ventana con el nombre del documento y `*` si hay cambios.
- **Barra de estado**: codificación, fin de línea, marca de cambios y número de palabras.
- **Diálogos** propios, iguales en Windows, Linux y macOS.
- Arquitectura MVC con puertos y adaptadores: M01 `DocumentState`, M07 `MarkdownCodec` (con corpus de fidelidad), C01 `AppController`, adaptadores de Tauri.
- Linters y formateadores: ESLint + Prettier (TS/Svelte), rustfmt + clippy pedantic (Rust), EditorConfig; job `lint` en la CI.
- Prueba que mantiene sincronizada la versión en `package.json`, `Cargo.toml` y `tauri.conf.json`.

### Changed
- Código existente formateado con Prettier; `build.rs` documentado.

### Known issues
- Limitación de Milkdown 7: los enlaces de referencia (`[texto][ref]`) se guardan como enlaces en línea y las URL sueltas entre `<` `>`. El documento se ve igual.
- Todavía no se pueden abrir ni guardar ficheros (llega en F2, versión 0.1.0).

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

[Unreleased]: https://github.com/IsmaelCaballero/editorMD/compare/v0.0.2...HEAD
[0.0.2]: https://github.com/IsmaelCaballero/editorMD/compare/v0.0.1...v0.0.2
[0.0.1]: https://github.com/IsmaelCaballero/editorMD/releases/tag/v0.0.1
