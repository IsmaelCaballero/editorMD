# CONTEXT.md — Contexto del proyecto editorMD

> Léelo **primero** al retomar el proyecto (sobre todo si se ha perdido el contexto de la conversación).
> Después lee STATUS.md (dónde estamos) y PLAN.md (qué falta y cómo).
> Última actualización: 2026-10-05

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
19. **Documentar el uso de IA** (modelo y versión, estimación previa de tokens y coste real de cada paso) en `docs/ai-usage/README.md`, PLAN.md §11 y la web. Medir con `scripts/ai-usage.mjs` sobre la transcripción de la sesión. **Empezar cada fase en una sesión nueva** para abaratar las llamadas.
20. **RQ.1 del usuario** (investigación): ¿es más rentable un modelo caro con poco *rework* o uno barato con más *rework*? Experimento **aprobado** (`docs/ai-usage/README.md` §4.2): F2 pasos 1 y 4, **3 repeticiones** por modelo (Opus 5.5 / Sonnet 5.5), revisión ciega. Sin errores externos al entorno. El usuario prefiere invertir en resultados concluyentes.
21. **Fases largas en varias sesiones**: F2 se parte en tres sesiones (A: pasos 1-3; B: paso 4 con el experimento T2; C: pasos 5-8). Antes de cerrar una sesión se guarda todo (STATUS, CONTEXT, PLAN, métricas de IA) y se deja en STATUS.md el texto exacto para arrancar la siguiente (PLAN.md §7, F2).
22. **Trabajo desde varios ordenadores**: el usuario puede retomar el proyecto en otro equipo clonando el repositorio (`docs/retomar-en-otro-ordenador.md`). Todo lo necesario debe estar en el repositorio: la memoria y las transcripciones de Claude Code son locales y no viajan.

## Convenciones del proyecto
- Idioma de la documentación: **español**. Código e identificadores: inglés.
- Componentes con IDs: `Mxx` (Modelo), `Vxx` (Vista), `Cxx` (Controlador), cada uno con su versión SemVer en PLAN.md §4.3 y su historial en §4.4.
- La lógica de texto vive en el Modelo como funciones puras `(texto, selección) → (texto, selección)`.
- Los controladores dependen de interfaces de Vista y servicios, para poder probarse con mocks.
- Una tarea solo está hecha si sus tests unitarios pasan.
- Al cerrar cada sesión: actualizar STATUS.md (registro de sesiones), marcar las tareas en PLAN.md y subir las versiones de los componentes tocados.

## Entorno de desarrollo
- **Dos ordenadores**, ambos con Windows 11. La carpeta del proyecto indica en cuál se trabaja:
  - `C:\ProgsConIA\editorMD`: el **original** (sesiones #1-#3, F0 a F2 sesión A; proyecto de Claude Code `C--ProgsConIA-editorMD`).
  - `C:\ProgIA\editorMD`: el **nuevo** (desde la sesión #4, F2 sesión B; proyecto `C--ProgIA-editorMD`). Identidad git configurada **solo en el repositorio** (`git config user.email Ismael.Caballero@uclm.es`).

  Las transcripciones de cada sesión, que se usan para medir el uso de IA, solo existen en el ordenador donde se hizo esa sesión.
- Git 2.51, Node 24.16, npm 11.13, WebView2, Rust 1.99 (MSVC), VS Build Tools 2026 y gh 2.102 instalados. `gh` autenticado como `IsmaelCaballero`.
- Repositorio **público**: https://github.com/IsmaelCaballero/editorMD (remoto `origin`). La CI se ejecuta en cada push a `main` y en cada PR.
- Nota: en PowerShell puede hacer falta recargar el PATH para ver `cargo`/`gh`.
- Usuario git: Ismael Caballero.
- **Antivirus Norton** en la máquina de desarrollo: inspecciona cada `editormd.exe` nuevo (debug, sin firmar) al ejecutarlo. Retrasa el arranque y puede distorsionar las medidas de tiempo y memoria; también puede ralentizar la compilación en `src-tauri/target/`. Las métricas de rendimiento (F9) se tomarán con la versión *release* y anotando si Norton estaba activo. Una exclusión del antivirus para `src-tauri	arget` es decisión del usuario (implica un riesgo de seguridad). Para los usuarios finales, firmar el código (D-08, en la 1.0) reduce estos análisis.

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
- **D-17 Milkdown *kit*** (no *Crepe*): núcleo mínimo; menús y barra de herramientas propios en su capa MVC.
- **F1 completada** (5 pasos, PR #5-#9) → versión **0.0.2**, etiqueta `v0.0.2` y ramas renombradas.
- **Plan de F2 aprobado** (2026-10-04): 8 pasos en 2 sesiones, experimento RQ.1 en los pasos 1 y 4 (PLAN.md §7).
- **API Rust de M09** (fijada en `docs/ai-usage/rq1/T1-text-codec-decode.md`): `Encoding` con los mismos identificadores que `src/model/document.ts` (`"utf-8"`, `"utf-8-bom"`, …, `"macintosh"`); la detección nunca devuelve `ascii` ni `iso-8859-1` (ASCII puro → `utf-8`); decodificación estricta, sin `U+FFFD`.
- **Antes de cada push: `npm run lint`** (en el PR #14 la CI falló por el formato de un JSON editado con `sed`).
- **Windows / Git Bash:** la herramienta Bash de Claude Code convierte `\n` y `\r` de los *heredocs* en saltos de línea reales. Para texto con secuencias de escape (JS, Rust), usar la herramienta de edición o `chr(92)` en Python, y comprobar después que el `<script>` de `docs/index.html` sigue siendo válido.
- Al comprobar la app real (`tauri dev`), **avisar al usuario antes de medir** para que no interactúe con la ventana mientras tanto (un `*` «fantasma» en F1 resultó ser una edición suya).

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
