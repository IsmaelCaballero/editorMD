# CONTEXT.md — Contexto del proyecto editorMD

> Léelo **primero** al retomar el proyecto (sobre todo si se ha perdido el contexto de la conversación).
> Después lee STATUS.md (dónde estamos) y PLAN.md (qué falta y cómo).
> Última actualización: 2026-10-03

## Qué es
Un editor Markdown de escritorio con GUI, **ligero** (poca memoria) y **portable** a Windows, Ubuntu, Fedora y macOS, con **instalador** para cada sistema. Habrá varias versiones a lo largo del tiempo, así que la arquitectura debe facilitar su evolución.

## Requisitos del usuario (literales y obligatorios)
1. Ficheros: abrir, cerrar, guardar, guardar como, importar, exportar.
2. Edición: seleccionar todo, copiar, pegar, buscar, reemplazar.
3. Formato: título 1/2/3, negrita, cursiva y otros adecuados.
4. Objetos: tabla, listas con viñetas, listas numeradas.
5. Lenguaje portable entre los 4 SO, con instaladores.
6. Aplicación ligera y con poca memoria.
7. **Patrón MVC** con componentes identificados y **versionados** (para ver cómo evolucionan).
8. **Todas las pruebas unitarias** correspondientes.
9. Mantener PLAN.md (avance), STATUS.md (estado) y CONTEXT.md (contexto).
10. Para texto plano (`.txt`, `.Rmd`, `.md`…): soportar distintas codificaciones (ASCII, UTF-8…) y finales de línea para **exportar/intercambiar ficheros entre SO** (PLAN.md §2.5).
11. Habrá varias versiones de la aplicación en el futuro.
12. Versionado con **Semantic Versioning 2.0.0** (PLAN.md §10.1).
13. Código documentado con el estándar de cada lenguaje: **rustdoc** (Rust) y **TSDoc + TypeDoc** (TS) (PLAN.md §10.2).
14. **Una rama git por feature**; **preguntar siempre al usuario antes de cada pull request/merge** (PLAN.md §10.3).
15. El usuario quiere **aprender**: mantener la página web `docs/index.html` con el plan, los pasos dados, un resumen, hipervínculos internos y el progreso gráfico (incluidos ramas y PR). Actualizarla en cada paso (PLAN.md §10.4).
16. **El usuario fusiona los PR él mismo desde GitHub.** Abrir el PR, esperar la CI en verde y avisar con una guía de revisión; después, revisar la fusión.
17. **No borrar ramas** tras fusionarlas; **al cerrar cada fase se renombran** (con el ok del usuario) a `hecha/F<m>.<n>de<T>/<nombre>` (PLAN.md §10.3 y §10.6). Objetivo: que se vea todo el proceso de desarrollo, no solo el resultado. En un desarrollo normal se recomienda **borrar las ramas una vez fusionadas**. En este proyecto **se conservan deliberadamente**: por razones académicas (para mostrar a los alumnos el proceso completo) y por **trazabilidad del esfuerzo de desarrollo**, ya que en el futuro el trabajo se repartirá entre varios miembros del equipo, cada uno trabajando sobre una parte del sistema con su propio arnés.
18. **Cada etiqueta (tag/release) se enlaza en `docs/index.html`** (sección «Versiones publicadas», `PROJECT.tags`).

## Convenciones del proyecto
- Idioma de la documentación: **español**. Código e identificadores: inglés.
- Componentes con IDs: `Mxx` (Modelo), `Vxx` (Vista), `Cxx` (Controlador), cada uno con su versión SemVer en PLAN.md §4.3 y su historial en §4.4.
- La lógica de texto vive en el Modelo como funciones puras `(texto, selección) → (texto, selección)`.
- Los controladores dependen de interfaces de Vista y servicios, para poder probarse con mocks.
- Una tarea solo está hecha si sus tests unitarios pasan.
- Al cerrar cada sesión: actualizar STATUS.md (registro de sesiones), marcar las tareas en PLAN.md y subir las versiones de los componentes tocados.

## Entorno de desarrollo
- Máquina principal: Windows 11, carpeta `C:\ProgsConIA\editorMD`.
- Git 2.51, Node 24.16, npm 11.13, WebView2, Rust 1.99 (MSVC), VS Build Tools 2026 y gh 2.102 instalados. `gh` autenticado como `IsmaelCaballero`.
- Repositorio **público**: https://github.com/IsmaelCaballero/editorMD (remoto `origin`). La CI se ejecuta en cada push a `main` y en cada PR.
- Nota: en PowerShell puede hacer falta recargar el PATH para ver `cargo`/`gh`.
- Usuario git: Ismael Caballero.

## Retomar el trabajo
Lee el apartado **«⏸ Punto de reanudación»** al principio de STATUS.md.

## Decisiones tomadas
*(Se trasladan aquí desde PLAN.md §8 cuando se cierran, con su justificación.)*
- Arquitectura MVC con *Passive View*: decidido (requisito del usuario).
- Electron y Java/JavaFX descartados por consumo de memoria.
- **D-01 Stack: Rust + Tauri 2** (backend Rust = servicios del Modelo; frontend TypeScript = Vista + Controladores + modelo de documento). Elegido por el usuario el 2026-10-03.
- **D-02 Modo de edición: WYSIWYG** (elegido por el usuario). Riesgo principal: la fidelidad del Markdown al guardar.
- **D-03 Formatos**: HTML, PDF, TXT nativos + DOCX/ODT vía Pandoc (opcional).
- **D-15 Codificaciones**: UTF-8 (±BOM), UTF-16 LE/BE, ASCII, ISO-8859-1/15, Windows-1252, Mac Roman; fin de línea LF/CRLF/CR. Se conservan al guardar; conversión con informe de pérdidas y transliteración.
- `.Rmd` admitido: los bloques R se conservan intactos y no se ejecutan.
- **D-05 Extras v1.0**: tema claro/oscuro, interfaz es+en, autoguardado/recuperación, pestañas multidocumento y barra de accesos directos configurable (guardar, previsualizar…).
- **D-07 Licencia**: MIT, código abierto.
- **D-11 PDF**: Typst embebido (sin dependencias externas).
- **D-12**: modo fuente Markdown en la v1.0, además del WYSIWYG.

## Decisiones abiertas
Ninguna bloqueante. Confirmadas por el usuario el 2026-10-03: D-06 GitHub, D-08 sin firma hasta la 1.0, D-10 SO mínimos, D-13 Svelte 5, D-14 Milkdown, D-16 convenciones. GitHub **público**. Ver PLAN.md §8.

## Cómo trabajar en el repo
- `npm test` (Vitest) · `npm run coverage` · `npm run check` (tipos) · `npm run build`
- `npm run lint` / `npm run format` (ESLint + Prettier) · `npm run lint:rust` / `npm run format:rust` (clippy + rustfmt)
- `npm run tauri dev` (app de escritorio; requiere Rust) · `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run docs` → `docs/api/ts` + `docs/api/rust` (no se versionan)
- Web de aprendizaje: `docs/index.html`; datos en el objeto `PROJECT` del `<script>`. **Actualizarla en cada paso** (diario, fases, ramas, componentes, KPIs).
- Estructura: `src/{model,view,controller}` (TS), `src-tauri/src/{model,commands.rs}` (Rust), `tests/` (Vitest), `fixtures/` (binarios).

## Glosario
- **GFM**: GitHub Flavored Markdown (tablas, tachado, tareas).
- **Split view**: editor de texto a la izquierda y vista previa renderizada a la derecha.
- **Passive View**: variante de MVC en la que la vista no tiene lógica y el controlador la gobierna por completo.
