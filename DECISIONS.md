# DECISIONS.md — Registro trazable de decisiones de editorMD

> Todas las decisiones tomadas desde el inicio del proyecto, con su fecha, sesión, quién decidió, las alternativas, el motivo y **dónde queda la traza**.
> Última actualización: 2026-10-05 (cierre de la sesión #5, F2 sesión C).

## Cómo leer este registro

| Campo | Significado |
|---|---|
| **ID** | `D-xx` producto y arquitectura (los mismos ID que PLAN.md §8) · `P-xx` proceso y convenciones · `IA-xx` uso de IA y RQ.1 · `T-xx` decisiones técnicas de implementación · `E-xx` entorno |
| **Sesión** | #1 = 2026-10-03 · #2 y #3 = 2026-10-04 (ordenador original, `C:\ProgsConIA`) · #4 y #5 = 2026-10-05 (ordenador nuevo, `C:\ProgIA`) |
| **Quién** | **Usuario** = la decidió el usuario · **Propuesta** = la propuso el asistente y el usuario la aprobó · **Asistente** = decisión técnica del asistente dentro de lo aprobado, registrada en el PR correspondiente |
| **Traza** | Fichero, sección, PR o commit donde se puede comprobar |
| **—** | No consta en el repositorio (se decidió en una conversación cuya transcripción no se conserva aquí). No se rellena por suposición |

Fuentes: PLAN.md (§2, §4, §7, §8, §10, §11), CONTEXT.md, STATUS.md (registro de sesiones), `docs/ai-usage/README.md`, `docs/index.html` (bitácora), CHANGELOG.md y los PR #1-#19. Las transcripciones de las sesiones #1-#3 solo existen en el ordenador original. Las de #4 y #5 están en el nuevo.

Una decisión que cambia otra anterior **no la borra**: se añade otra fila con «sustituye a …».

---

## 1. Producto y arquitectura (`D-xx`, PLAN.md §8)

| ID | Fecha · sesión | Decisión | Alternativas consideradas | Motivo | Quién | Traza |
|---|---|---|---|---|---|---|
| D-00 | 2026-10-03 · #1 | **Patrón MVC** (variante *Passive View*) con componentes identificados (`Mxx`, `Vxx`, `Cxx`) y **versionados** | — (requisito) | Requisito literal del usuario: ver cómo evoluciona cada componente | Usuario | CONTEXT.md (requisito 7), PLAN.md §4 |
| D-01 | 2026-10-03 · #1 | Stack **Rust + Tauri 2**: backend Rust = servicios del Modelo; frontend TS = Vista + Controladores + modelo del documento | C++/Qt, Python/PySide6, Rust/Slint, Go/Fyne. **Electron y Java/JavaFX descartados por memoria** | Ligereza (requisito 6), portabilidad a 4 SO e instaladores nativos | Usuario (entre opciones propuestas) | PLAN.md §3 y §8 |
| D-02 | 2026-10-03 · #1 | Edición **WYSIWYG** | Editor de texto + vista previa (*split view*, en el glosario de CONTEXT.md) | Preferencia del usuario. Riesgo asumido: la fidelidad del Markdown al guardar (→ T-04, D-12) | Usuario | PLAN.md §8 y §9 |
| D-03 | 2026-10-03 · #1 | Importar/exportar: **HTML, PDF y TXT nativos; DOCX/ODT con Pandoc** (opcional) | — | Pandoc es opcional: lo básico funciona sin dependencias externas | Usuario | PLAN.md §2.4 |
| D-04 | 2026-10-03 · #1 | Dialecto: **CommonMark + GFM + front matter + R Markdown** (`.Rmd`: los bloques R se conservan intactos y no se ejecutan) | — | — | Propuesta (asumida) | PLAN.md §2.3 |
| D-05 | 2026-10-03 · #1 | Extras de la v1.0: tema claro/oscuro, interfaz es+en, autoguardado y recuperación, pestañas multidocumento, **barra de accesos directos configurable** | — | Elegidos por el usuario entre los añadidos propuestos (PLAN.md §2.2) | Usuario | PLAN.md §2.2 |
| D-06 | 2026-10-03 · #1 (confirmada en #2) | **GitHub, repositorio público** + GitHub Actions | — | — | Usuario | PLAN.md §8; repositorio creado en #2 |
| D-07 | 2026-10-03 · #1 | Licencia **MIT**, código abierto | — | — | Usuario | `LICENSE`, PLAN.md §8 |
| D-08 | 2026-10-03 · #1 | **Sin firma de código** en Windows/macOS hasta la 1.0 | — | Se retoma en F8 (riesgo «firma y notarización en macOS», PLAN.md §9) | Usuario | PLAN.md §8 y §9 |
| D-09 | 2026-10-03 · #1 | Interfaz en **español e inglés** | Solo español | Consecuencia de D-05 | Usuario | PLAN.md §8 |
| D-10 | 2026-10-03 · #1 | SO mínimos: **Windows 10+, Ubuntu 22.04+, Fedora 39+, macOS 11+** | — | — | Usuario | PLAN.md §8; CI en los 4 SO |
| D-11 | 2026-10-03 · #1 | PDF con **Typst embebido** (MD → Typst → PDF) | Pandoc + motor externo; imprimir desde el WebView (distinto en cada SO); `printpdf` | Sin dependencias externas y con el mismo resultado en todos los SO | Usuario | PLAN.md §2.4 y §8 |
| D-12 | 2026-10-03 · #1 | **Modo fuente Markdown** en la v1.0, además del WYSIWYG | Solo WYSIWYG | Mitiga el riesgo de D-02 | Usuario | PLAN.md §8 |
| D-13 | 2026-10-03 · #1 (confirmada en #2) | Frontend **Svelte 5 + Vite** | — | — | Usuario | PLAN.md §8 |
| D-14 | 2026-10-03 · #1 (confirmada en #2) | Editor WYSIWYG **Milkdown** (sobre ProseMirror + remark) | Tiptap | *Markdown-first*: mejor fidelidad de ida y vuelta (PLAN.md §3) | Usuario | PLAN.md §8 |
| D-15 | 2026-10-03 · #1 | Codificaciones: **UTF-8 (±BOM), UTF-16 LE/BE, ASCII, ISO-8859-1, ISO-8859-15, Windows-1252 y Mac Roman**; fin de línea **LF/CRLF/CR**; se conservan al guardar; conversión con informe de pérdidas y transliteración | — | Requisito 10: intercambiar ficheros entre SO | Usuario | PLAN.md §2.5 |
| D-16 | 2026-10-03 · #1 | Convenciones: SemVer 2.0.0 · rustdoc + TSDoc/TypeDoc · rama por feature con PR aprobado por el usuario · web de aprendizaje | — | Requisitos 12-15 del usuario | Usuario | PLAN.md §10 |
| D-17 | 2026-10-04 · #2 | **Milkdown *kit*** (núcleo mínimo; menús y barra de herramientas propios) | Milkdown *Crepe* (editor ya montado) | *Crepe* es más pesado e impone su interfaz; los menús propios encajan en la capa MVC | Propuesta | PLAN.md §8, F1 (PR #7) |

## 2. Proceso y convenciones (`P-xx`)

| ID | Fecha · sesión | Decisión | Motivo | Quién | Traza |
|---|---|---|---|---|---|
| P-01 | 2026-10-03 · #1 | Mantener **PLAN.md** (avance), **STATUS.md** (estado) y **CONTEXT.md** (contexto); actualizarlos al cerrar cada sesión | Requisito 9; poder retomar sin la conversación | Usuario | CONTEXT.md |
| P-02 | 2026-10-03 · #1 | Documentación en **español**; código e identificadores en **inglés** | — | — | CONTEXT.md, convenciones |
| P-03 | 2026-10-03 · #1 | **Una rama por feature** (`feature/F<n>-<slug>`); `main` siempre estable; fusión `--no-ff` | Requisito 14 | Usuario | PLAN.md §10.3 |
| P-04 | 2026-10-03 · #1 | **Preguntar siempre al usuario antes de cada PR o fusión** | Requisito 14 | Usuario | CONTEXT.md (requisito 14) |
| P-05 | 2026-10-03 · #1 | **Web de aprendizaje** `docs/index.html` (plan, pasos, conceptos, ramas, PR, progreso), actualizada en cada paso | Requisito 15: el usuario quiere aprender | Usuario | PLAN.md §10.4 |
| P-06 | 2026-10-03 · #1 | Commits con **Conventional Commits**; **CHANGELOG** con Keep a Changelog | Traducir commits a SemVer | Propuesta | PLAN.md §10.1 |
| P-07 | 2026-10-03 · #1 | **TDD y pruebas unitarias obligatorias**: una tarea solo está hecha si sus pruebas pasan; los controladores se prueban con *mocks* de los puertos | Requisito 8 | Usuario / Propuesta | CONTEXT.md, PLAN.md §5 |
| P-08 | 2026-10-04 · #2 | **El usuario fusiona los PR él mismo desde GitHub.** El asistente abre el PR, espera la CI en verde en los 4 SO y avisa con una guía de revisión | Aprender el proceso | Usuario | CONTEXT.md (requisito 16), PR #1 |
| P-09 | 2026-10-04 · #2 | **No borrar ramas tras fusionarlas** | Razones académicas y trazabilidad del esfuerzo cuando trabaje un equipo (en un desarrollo normal se recomienda borrarlas) | Usuario | CONTEXT.md (requisito 17), PLAN.md §10.3 |
| P-10 | 2026-10-04 · #2 | **Renombrar las ramas al cerrar cada fase** a `hecha/F<m>.<n>de<T>/<nombre>`, con el ok del usuario | Ver la secuencia temporal del desarrollo | Usuario | PLAN.md §10.3 y §10.6; ramas `hecha/F0.*`, `hecha/F1.*` |
| P-11 | 2026-10-04 · #2 | Restaurar la rama `feature/F0-scaffold`, que se había borrado antes de que existiera P-09 | Coherencia con P-09 | Usuario | STATUS.md (registro de #2) |
| P-12 | 2026-10-04 · #2 | **Cada etiqueta (tag/release) se enlaza en la web** (`PROJECT.tags`) | Trazabilidad de versiones | Usuario | CONTEXT.md (requisito 18) |
| P-13 | 2026-10-04 · #2 | Linters y formateadores (F0.7): ESLint + Prettier, rustfmt + clippy *pedantic* con `-D warnings`, EditorConfig, job `lint` en la CI. Un PR no se fusiona con `lint` en rojo | Calidad uniforme | Propuesta | PLAN.md §10.5, PR #3 |
| P-14 | 2026-10-04 · #2 | Etiquetas **anotadas** sobre el merge de `main`, con *GitHub Release* (pre-release mientras sea `0.x`) | Releases visibles | Propuesta | PLAN.md §10.3; `v0.0.1`, `v0.0.2` |
| P-15 | 2026-10-04 · #2 | Al comprobar la app real, **avisar al usuario antes de medir**, para que no interactúe con la ventana | Un `*` «fantasma» en F1 resultó ser una edición del usuario | Usuario / Propuesta | CONTEXT.md, PR #8 |
| P-16 | 2026-10-04 · #3 | **`npm run lint` antes de cada push** | En el PR #14 la CI falló por el formato de un JSON editado con `sed` | Propuesta | CONTEXT.md |
| P-17 | 2026-10-04 · #3 | **Trabajo desde varios ordenadores**: todo lo necesario va en el repositorio (la memoria y las transcripciones de Claude Code son locales) | El usuario cambia de equipo | Usuario | CONTEXT.md (requisito 22), `docs/retomar-en-otro-ordenador.md` |
| P-18 | 2026-10-05 · #5 | **Subir a GitHub las ramas de todas las ejecuciones del experimento** (`rq1/T2/r1…r6`; las de T1 ya estaban) | Ejercicio académico: los alumnos deben tener toda la información, sin depender de ningún equipo | Usuario | Push de la sesión #5; web (ramas) |
| P-19 | 2026-10-05 · #5 | **Registro trazable de decisiones** en este fichero, `DECISIONS.md` | Trazabilidad académica | Usuario | Este fichero |

## 3. Uso de IA y pregunta de investigación RQ.1 (`IA-xx`)

| ID | Fecha · sesión | Decisión | Motivo | Quién | Traza |
|---|---|---|---|---|---|
| IA-01 | 2026-10-04 · #2 | **Documentar el uso de IA**: modelo y versión, estimación previa de tokens y coste real de cada paso, en `docs/ai-usage/README.md`, PLAN §11 y la web; medirlo con `scripts/ai-usage.mjs` sobre la transcripción | Requisito 19 | Usuario | PR #10 |
| IA-02 | 2026-10-04 · #2 | **Empezar cada fase en una sesión nueva** | Los datos de F0-F1 mostraron que el contexto acumulado multiplica el coste por llamada (de 145 k a 485 k tokens de media) | Propuesta | `docs/ai-usage/README.md` §3 |
| IA-03 | 2026-10-04 · #2 | Coste en **USD equivalentes** de la API (no es una factura con suscripción) | Comparar modelos y dimensionar la licencia | Propuesta | `docs/ai-usage/README.md` §1 |
| IA-04 | 2026-10-04 · #2 | **Estimación previa en cada paso** desde F2 (modelo: 0,04 USD por llamada + 0,20 USD por millón de tokens de contexto) | Aprender a presupuestar | Propuesta | `docs/ai-usage/README.md` §5 |
| IA-05 | 2026-10-04 · #2-#3 | **RQ.1** (del usuario): ¿modelo caro con poco *rework* o barato con más *rework*? Experimento **aprobado**: Opus 5.5 frente a Sonnet 5.5 en F2 pasos 1 y 4, **3 repeticiones** por modelo, subagentes en *worktrees* aislados, pruebas ocultas, **revisión ciega**, sin incidencias externas | El usuario prefiere invertir en resultados concluyentes | Usuario | `docs/ai-usage/README.md` §4.2 |
| IA-06 | 2026-10-04 · #3 | Plan de F2 en 8 pasos y **2 sesiones** (A: 1-4, B: 5-8) | Que el contexto no supere ~350 k | Propuesta | PLAN.md §7 |
| IA-07 | 2026-10-04 · #3 | Adelantar el cambio de sesión: **A = pasos 1-3**, B empieza en el 4 (sustituye en parte a IA-06) | Ahorrar contexto y empezar T2 limpio | Propuesta | PLAN.md §7, PR #15 |
| IA-08 | 2026-10-04 · #3 | Tras el primer lanzamiento fallido de T1 (los *worktrees* nacían de `main` sin la especificación), el encargo empieza con `git merge --ff-only <commit de la especificación>`; las ejecuciones abortadas cuentan en el paso pero no en la comparación | Validez del experimento | Asistente | `docs/ai-usage/README.md` §4.3 |
| IA-09 | 2026-10-04 · #3 | **T1: gana Sonnet** (44 % más barato, la misma calidad); se integra la candidata A (r1) | Regla de decisión: el menor coste total | Usuario (revisión ciega) | PR #11, #12 (cerrado), #13; §4.4 |
| IA-10 | 2026-10-05 · #4 | **T2: gana Sonnet** (49 % más barato, mejor puntuado: 4,8 frente a 4,2); se integra A (r1) y B se cierra sin fusionar | Regla de decisión | Usuario (revisión ciega) | PR #16, #17 (cerrado), #18; §4.5 |
| IA-11 | 2026-10-05 · #4 | **Respuesta provisional a la RQ.1:** con especificación cerrada, pruebas y CI, Sonnet es más rentable; **Opus queda como orquestador** (planificar, especificar, integrar, documentar). Propuesta: delegar en Sonnet los pasos bien especificados | Datos de T1 y T2 | Propuesta | `docs/ai-usage/README.md` §4.6 |
| IA-12 | 2026-10-05 · #5 | El **paso 5 (FileController) lo hace Opus sin delegar** | El encargo era abierto (diseño de puertos e interfaz) y el ahorro se iba en especificar e integrar | Usuario (opción A) | PR #19; README §5.1 |
| IA-13 | 2026-10-05 · #5 | **Tercera sesión para F2:** B = paso 4, C = paso 5 (sustituye a IA-07 para los pasos 5-8) | La sesión B se cerró tras T2 | Usuario | PLAN.md §7 |
| IA-14 | 2026-10-05 · #5 | **Cerrar la sesión C tras el paso 5**; los pasos 6-8 van en una sesión nueva (**D**) | El contexto (~210 k) habría superado ~350 k en el paso 6: ≈ 5,5 USD aquí frente a ≈ 4 en una sesión nueva | Propuesta + Usuario | STATUS.md (punto de reanudación) |
| IA-15 | 2026-10-05 · #5 | **Quitar los 6 *worktrees* de T2** tras comprobar que estaban limpios; las ramas se conservan | Espacio y orden; sin pérdida de información (P-09) | Usuario | Sesión #5 |

## 4. Decisiones técnicas de implementación (`T-xx`)

| ID | Fecha · sesión | Decisión | Motivo | Quién | Traza |
|---|---|---|---|---|---|
| T-01 | 2026-10-03 · #1 | `.gitattributes` que impide que git convierta los ficheros de prueba de codificaciones | Que las pruebas comprueben los bytes reales | Asistente | PR #1 |
| T-02 | 2026-10-04 · #2 | La lógica de texto vive en el Modelo como **funciones puras** `(texto, selección) → (texto, selección)`; los controladores dependen de **puertos** (`IEditorView`, `IBackend`, `IDialogService`, `IWindowView`) y se conectan en el *composition root* (`src/main.ts`) | Probar sin ventana ni Rust | Propuesta | PLAN.md §4.1, `src/controller/ports.ts` |
| T-03 | 2026-10-04 · #2 | M01: «modificado» se calcula **comparando con la última instantánea guardada**, no con un booleano | Deshacer hasta el texto guardado quita el `*` | Asistente | PR #5 |
| T-04 | 2026-10-04 · #2 | M07: estilo canónico `STRINGIFY_OPTIONS` y **corpus de fidelidad** de 24 documentos; v0.2.0 protege el front matter y las cabeceras `{r …}` (un *spike* mostró 2 pérdidas de datos en Milkdown) | Riesgo principal de D-02 | Asistente | PR #6, #7 |
| T-05 | 2026-10-04 · #2 | **Diálogos propios** (V12), iguales en los 4 SO, para mensajes y cambios sin guardar | Aspecto uniforme y fácil de probar | Propuesta | PR #8 |
| T-06 | 2026-10-04 · #2 | Prueba de **sincronización de versiones** entre `package.json`, `Cargo.toml` y `tauri.conf.json` | Evitar versiones desalineadas | Asistente | PR #9 |
| T-07 | 2026-10-04 · #3 | API Rust de M09: `Encoding` con los **mismos identificadores que el frontend**; la detección **nunca devuelve `ascii` ni `iso-8859-1`** (ASCII puro → `utf-8`); decodificación **estricta** (sin U+FFFD) | Sin ambigüedad entre capas y sin pérdidas silenciosas | Propuesta | `docs/ai-usage/rq1/T1-text-codec-decode.md` |
| T-08 | 2026-10-04 · #3 | ASCII, Latin-1 real y la codificación UTF-16 **implementados a mano** | `encoding_rs` sigue WHATWG: no codifica UTF-16 y trata Latin-1 como Windows-1252 | Asistente | PLAN.md §2.5, PR #14 |
| T-09 | 2026-10-04 · #3 | **Transliteración** con `deunicode`; estrategias de pérdida: sustituir por `?`, transliterar, entidades HTML | D-15 | Asistente | PR #15 |
| T-10 | 2026-10-05 · #4 | M08: **escritura atómica** (temporal en la misma carpeta + `sync_all` + renombrado); `FileError` serializable con discriminante `kind`; comandos `async` | No perder nunca el fichero original | Propuesta (especificación T2) | `docs/ai-usage/rq1/T2-file-service.md`, PR #18 |
| T-11 | 2026-10-05 · #5 | **Selectores nativos** con `tauri-plugin-dialog` detrás de un puerto nuevo, `IFilePicker` (adaptador A03); los diálogos propios (T-05) se quedan para los mensajes | Los selectores de ficheros deben ser los del sistema | Propuesta | PR #19 |
| T-12 | 2026-10-05 · #5 | **Permisos mínimos** de Tauri: `dialog:allow-open`, `dialog:allow-save` y `core:window:allow-destroy` (no `dialog:default`) | Mínimo privilegio | Asistente | `src-tauri/capabilities/default.json`, PR #19 |
| T-13 | 2026-10-05 · #5 | «**Cerrar**» deja un documento vacío sin título (como «Nuevo») hasta que haya pestañas (F7) | Una sola ventana y un solo documento | Propuesta | PR #19 |
| T-14 | 2026-10-05 · #5 | **Fichero de solo lectura**: «Guardar» pasa a «Guardar como» | No fallar al guardar | Propuesta | PR #19 |
| T-15 | 2026-10-05 · #5 | **Caracteres que no caben** al guardar: en el paso 5, un mensaje con los caracteres y sus líneas, sin guardar; las opciones (transliterar, sustituir, cambiar de codificación) llegan en el paso 6 | Nunca perder datos en silencio | Propuesta | PR #19; STATUS.md |
| T-16 | 2026-10-05 · #5 | La guarda de cambios sin guardar se pregunta **antes** de Abrir, Nuevo, Cerrar y **al cerrar la ventana** (`onCloseRequested`) | Igual que los editores clásicos | Asistente | PR #19 |
| T-17 | 2026-10-05 · #5 | M01 0.3.0: `markSaved(path, content)` registra el **texto realmente escrito**; lo tecleado durante el guardado sigue sin guardar | Evitar marcar como guardados cambios que no llegaron al disco | Asistente | PR #19 |
| T-18 | 2026-10-05 · #5 | La **bienvenida** es un documento sin título (antes tenía una ruta inventada) | «Guardar» habría escrito en una carpeta cualquiera | Asistente | PR #19 |
| T-19 | 2026-10-05 · #5 | `CURRENT_PHASE = 'F2'`; «Codificación y fin de línea…» muestra un aviso temporal hasta el paso 6 | Activar Abrir y Guardar en los menús y atajos | Asistente | PR #19 |

## 5. Entorno (`E-xx`)

| ID | Fecha · sesión | Decisión | Quién | Traza |
|---|---|---|---|---|
| E-01 | 2026-10-03 · #1 | Toolchain: Git, Node 24, Rust 1.99 (MSVC), VS Build Tools, WebView2, `gh` | Usuario | CONTEXT.md |
| E-02 | 2026-10-04 · #2 | **Antivirus Norton**: se tiene en cuenta al medir el arranque y la memoria (se medirá con la *release*, anotando si estaba activo). Excluir `src-tauri\target` del análisis es **decisión del usuario** (riesgo de seguridad) | Usuario | CONTEXT.md |
| E-03 | 2026-10-05 · #4 | **Dos ordenadores**: `C:\ProgsConIA\editorMD` (original, sesiones #1-#3) y `C:\ProgIA\editorMD` (nuevo, desde la #4), con la **identidad git configurada solo en el repositorio** | Usuario | CONTEXT.md |

## 6. Decisiones abiertas

Ninguna bloqueante. Pendientes de fases futuras, ya previstas en el plan: la firma de código (D-08, en F8), los valores por defecto de las preferencias (F7) y la medición del rendimiento con la *release* (F9).
