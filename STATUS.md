# STATUS.md — Estado del proyecto editorMD

> Fotografía del estado actual. Se actualiza **al final de cada sesión de trabajo**.
> Última actualización: 2026-10-04

## ⏸ Punto de reanudación (2026-10-04, fin de la sesión #3 = F2 sesión A)

**Dónde estamos:** F2 pasos 1-3 terminados. Pasos 1 y 2 fusionados (PR #13 y #14). **Paso 3** (`feature/F2-text-codec-loss`, M09 v0.3.0) → **PR #15**, pendiente de tu revisión. M09 `TextCodec` está completo: lectura, escritura, finales de línea, ida y vuelta byte a byte e informe de pérdidas.

**Cambio de plan aprobado:** la sesión A termina tras el paso 3 (no tras el 4) para que el experimento T2 empiece con el contexto limpio (≈ 2 USD eq. de ahorro). **Sesión B = pasos 4-8.**

**Siguiente paso:** F2 4/8 `feature/F2-file-service`: M08 `FileService` (escritura atómica, solo lectura) + comandos Tauri `open_file`/`save_file` que usan M09, con el **experimento RQ.1 T2** (3 × Opus 5.5, 3 × Sonnet 5.5, pruebas ocultas, revisión ciega). Procedimiento y lecciones en `docs/ai-usage/README.md` §4.3. El encargo de los subagentes debe empezar con `git merge --ff-only <commit de la especificación>`, porque sus *worktrees* nacen de `main`.

### Cómo abrir la sesión B
- **En otro ordenador:** sigue [`docs/retomar-en-otro-ordenador.md`](docs/retomar-en-otro-ordenador.md): instalar, clonar, comprobar y abrir Claude Code.
- **En este mismo ordenador:** `git checkout main && git pull`, y después `claude` en `C:\ProgsConIA\editorMD`.

Primero fusiona el PR #15 (o pídele a la sesión B que lo revise contigo). Después pega este texto:

> Retomamos editorMD, F2 sesión B, en un ordenador nuevo recién clonado. Lee CONTEXT.md, STATUS.md, PLAN.md (§7 F2) y docs/ai-usage/README.md. Comprueba el estado de git y de los PR abiertos, verifica que el entorno funciona (npm test, cargo test, npm run lint) y preséntame el paso 4/8 (FileService con el experimento RQ.1 T2) con su estimación de tokens. No empieces hasta que te dé el ok.

**Uso de IA:** F0 10,38 · F1 15,54 · **F2 sesión A ≈ 18,1 USD eq.** (290 llamadas en la transcripción; incluye las 12 ejecuciones de T1 y la corrección de su salida infrarregistrada). Detalle por paso en `docs/ai-usage/README.md` §5.1.

**Reglas de trabajo** (CONTEXT.md 16-22): tú fusionas los PR; las ramas se renombran al cerrar la fase; cada etiqueta se enlaza en la web; avisar antes de medir en la app real; documentar el uso de IA en cada paso; guardar todo antes de cambiar de sesión; `npm run lint` antes de cada push.

**Entorno:** Norton inspecciona cada `editormd.exe` nuevo al ejecutarlo (ver CONTEXT.md): tenerlo en cuenta al medir el arranque o la memoria.

## Resumen

| Campo | Valor |
|---|---|
| Fase actual | **F2 en curso** · pasos 1-3 hechos (sesión A cerrada); siguiente: paso 4 en la sesión B |
| Versión de la app | **0.0.2** (PR #9) · última etiqueta publicada: [v0.0.2](https://github.com/IsmaelCaballero/editorMD/releases/tag/v0.0.2) |
| Stack | Rust + Tauri 2 · WYSIWYG |
| Salud | 🟢 CI en verde en los 4 SO; app arranca en Windows 11 |
| Tests | TS: 341/341 ✅ · Rust: 63/63 + 14 doctests ✅ (+ 31 pruebas ocultas de RQ.1) |
| Cobertura | Modelo + Controlador + Adaptadores: **100 % de líneas** |
| CI | ✅ GitHub Actions: Windows, Ubuntu 22.04, macOS y Fedora |
| Git | https://github.com/IsmaelCaballero/editorMD · F0: PR #1-#4 fusionados y ramas renombradas `hecha/F0.*de4/…` · F1: PR #5-#9 fusionados y ramas renombradas `hecha/F1.*de5/…` · PR #10 métricas de IA |
| Web del proyecto | `docs/index.html` |

## Progreso por fase

| Fase | Estado | Avance |
|---|---|---|
| F0 Decisiones y entorno | ✅ Cerrada (ramas renombradas) | 9/9 |
| F1 Esqueleto MVC | ✅ Cerrada (ramas renombradas, v0.0.2) | 5/5 |
| F2 Ficheros y codificaciones | 🔄 En curso (sesión A cerrada) | 3/8 |
| F3 Edición | ⏳ Pendiente | 0/3 |
| F4 Formato | ⏳ Pendiente | 0/4 |
| F5 Objetos | ⏳ Pendiente | 0/3 |
| F6 Importar / Exportar | ⏳ Pendiente | 0/5 |
| F7 Preferencias y extras | ⏳ Pendiente | 0/6 |
| F8 Empaquetado | ⏳ Pendiente | 0/6 |
| F9 QA y release | ⏳ Pendiente | 0/4 |

## Versiones de componentes (resumen)
Implementados: **M00 AppInfo 0.1.0**, **M09 TextCodec 0.3.0**, **M01 DocumentState 0.2.0**, **M07 MarkdownCodec 0.2.0**, **V02 WysiwygEditorView 0.1.0**, **M17 SemVer 0.1.0**, **V01 MainWindow 0.3.0**, **V12 DialogService 0.1.0**, **C01 AppController 0.1.0**, **A01 TauriBackend 0.1.0**, **A02 TauriWindow 0.1.0**. El resto, en 0.0.0. Ver PLAN.md §4.3.

## Estado de los tests por componente

| Componente | Tests | Pasan | Cobertura |
|---|---|---|---|
| M17 SemVer (TS) | 29 | 29 | 97,95 % |
| M01 DocumentState (TS) | 22 | 22 | 100 % |
| M07 MarkdownCodec (TS) | 163 | 163 | 100 % |
| V01 menús + MenuBar (TS/Svelte) | 43 | 43 | — |
| V02 MilkdownEditorView (jsdom) | 33 | 33 | — |
| C01 AppController (mocks) | 13 | 13 | 100 % |
| V12 diálogos + ShellState (jsdom) | 11 | 11 | — |
| A01/A02 adaptadores Tauri (vi.mock) | 4 | 4 | 100 % |
| M00 AppInfo (Rust) | 3 + 1 doctest | 4 | — |
| M09 TextCodec (Rust) | 60 + 13 doctests (+ 31 ocultas de RQ.1) | 73 | — |

## Bloqueos / pendiente del usuario
- Revisar y fusionar el PR #15 (paso 3).

## Próximos pasos
1. PR #15 (paso 3) → sesión B nueva.
2. Paso 4/8 con el experimento RQ.1 T2 → pasos 5-8 → cierre de F2 (versión 0.1.0).

## Registro de sesiones
| Fecha | Sesión | Hecho |
|---|---|---|
| 2026-10-04 | #3 | PR #13 y #14 fusionados. Pasos 2 y 3: M09 v0.2.0 (escritura, finales de línea, 27 *fixtures* de ida y vuelta) y v0.3.0 (informe de pérdidas, transliteración) → PR #15. Sesión A cerrada tras el paso 3 (aprobado). Guía para retomar en otro ordenador. Fin de la sesión #3. |
| 2026-10-04 | #3 | Sesión nueva para F2. Plan de F2 aprobado (8 pasos, 2 sesiones, RQ.1 con 3 repeticiones). Paso 1/8: especificación T1, pruebas ocultas y 6 ejecuciones del experimento. |
| 2026-10-03 | #1 | Plan inicial; creados PLAN.md, STATUS.md y CONTEXT.md; propuesta de stacks y arquitectura MVC |
| 2026-10-03 | #1 | Decididos D-01 (Tauri), D-02 (WYSIWYG), D-03 (formatos), D-15 (codificaciones y fin de línea). Añadido el soporte .Rmd. Plan reescrito para Tauri |
| 2026-10-03 | #1 | Decididos D-05 (extras + barra de accesos directos), D-07 (MIT), D-11 (Typst), D-12 (modo fuente). Nuevos componentes M14–M16, V09–V11 |
| 2026-10-04 | #2 | PR #9 fusionado (F1 cerrada). Uso de IA medido con la transcripción: F0 10,38 y F1 15,54 USD eq.; análisis de la RQ.1 y diseño del experimento → PR #10. Fin de la sesión #2. |
| 2026-10-04 | #2 | PR #8 fusionado. El `*` «fantasma» era una edición del usuario (resuelto). F1 paso 5: versión 0.0.2, CHANGELOG, prueba de sincronización de versiones → PR #9. **F1 completada.** |
| 2026-10-04 | #2 | PR #7 fusionado. F1 paso 4: C01 con TDD (mocks), adaptadores Tauri, diálogos propios, atajos y composition root; investigación de un `*` en el título que no se pudo reproducir (jsdom, Edge con y sin ventana, 3 arranques de la app) → PR #8. |
| 2026-10-04 | #2 | PR #6 fusionado. F1 paso 3: spike Milkdown (2 pérdidas de datos descubiertas) → M07 v0.2.0 con TDD → V01 menús → V02 editor real; RAM 113 MB privados; bundle 508 kB → PR #7. |
| 2026-10-04 | #2 | PR #5 fusionado. F1 paso 2: M07 MarkdownCodec + corpus de 24 documentos, TDD (101 tests, todos en verde a la primera) → PR #6. |
| 2026-10-04 | #2 | PR #4 fusionado; ramas de F0 renombradas (`hecha/F0.<n>de4/…`). Plan de F1 aprobado (Milkdown *kit*, 5 pasos). F1 paso 1: interfaces MVC + M01 DocumentState con TDD (PR #5). |
| 2026-10-04 | #2 | PR #3 fusionado por el usuario. Restaurada la rama local `feature/F0-scaffold` (se había borrado antes de la norma). Nueva convención: renombrar las ramas a `hecha/F<m>.<n>de<T>/…` al cerrar la fase. Rama `chore/F0-close` (PR #4). |
| 2026-10-04 | #2 | PR #2 fusionado por el usuario. Etiqueta + release `v0.0.1`. Decidido: ramas conservadas y enlace a cada etiqueta en la web. F0.7: ESLint, Prettier, rustfmt, clippy pedantic, EditorConfig y job lint en CI (rama `feature/F0-linters`, PR #3). |
| 2026-10-04 | #2 | `tauri dev` ✅ (la app arranca). Repositorio público creado; PR #1 con CI verde en los 4 SO; fusionado por el usuario. Rama `chore/release-0.0.1`: CHANGELOG 0.0.1 y actualización de la documentación y de la web. |
| 2026-10-03 | #1 | Toolchain instalada (Rust 1.99, MSVC, gh 2.102). cargo test ✅, rustdoc ✅. Fin de la sesión. |
| 2026-10-03 | #1 | CI GitHub Actions (4 SO) preparada. Decidido: GitHub público y PR F0 tras compilar Rust. Sesión cerrada por el usuario (problemas instalando `gh`). |
| 2026-10-03 | #1 | Convenciones (SemVer 2.0.0, rustdoc/TSDoc, rama por feature + PR, web). Git init + commit en main. Rama `feature/F0-scaffold`: Vite+Svelte+TS, Tauri 2, M00 AppInfo, M17 SemVer (29 tests), TypeDoc, .gitattributes, web `docs/index.html` |

## Checklist de release (se usará en F9)
- [ ] Intercambio entre SO: un fichero creado en Windows (CRLF, Windows-1252/UTF-8 BOM) se abre bien en Linux/macOS y viceversa
- [ ] Windows 11: instalar, abrir .md con doble clic, editar, guardar, exportar PDF, desinstalar
- [ ] Ubuntu: ídem (.deb y AppImage)
- [ ] Fedora: ídem (.rpm)
- [ ] macOS: ídem (.dmg, Intel y Apple Silicon)
- [ ] Objetivos de RAM, arranque y tamaño cumplidos
