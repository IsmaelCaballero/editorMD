# STATUS.md — Estado del proyecto editorMD

> Fotografía del estado actual. Se actualiza **al final de cada sesión de trabajo**.
> Última actualización: 2026-10-04

## ⏸ Punto de reanudación (2026-10-04, sesión #2)

**Dónde estamos:** **F1 en curso, paso 4/5.** PR #7 (paso 3) fusionado (`efc4b42`). Rama `feature/F1-app-controller` → **PR #8**: C01 `AppController` (TDD con mocks), adaptadores A01/A02, diálogos V12, atajos y composition root. Siguiente: paso 5/5 `chore/F1-close` (versión 0.0.2, etiqueta y renombrado de las ramas de F1).

**Plan de F1 (aprobado, Milkdown *kit*):**

| Paso | Rama | Contenido | Nombre al cerrar F1 |
|---|---|---|---|
| 1/5 | `feature/F1-document-model` | Interfaces MVC (`ports.ts`) + M01 `DocumentState` | `hecha/F1.1de5/feature/F1-document-model` |
| 2/5 | `feature/F1-markdown-codec` | M07 `MarkdownCodec` (GFM, front matter, Rmd) + corpus de fidelidad | `hecha/F1.2de5/feature/F1-markdown-codec` |
| 3/5 | `feature/F1-milkdown-editor` | V02 editor Milkdown *kit* + V01 menús declarativos | `hecha/F1.3de5/feature/F1-milkdown-editor` |
| 4/5 | `feature/F1-app-controller` | C01 `AppController` + composition root (Nuevo, Acerca de, marca `*`) | `hecha/F1.4de5/feature/F1-app-controller` |
| 5/5 | `chore/F1-close` | Versión 0.0.2, etiqueta, documentación y renombrado | `hecha/F1.5de5/chore/F1-close` |

**Reglas de trabajo** (CONTEXT.md 16-18): el usuario fusiona los PR; las ramas no se borran: se renombran al cerrar la fase; cada etiqueta se enlaza en la web.

## Resumen

| Campo | Valor |
|---|---|
| Fase actual | **F1 — Esqueleto MVC** (paso 4/5) |
| Versión de la app | **0.0.1** · etiqueta [v0.0.1](https://github.com/IsmaelCaballero/editorMD/releases/tag/v0.0.1) |
| Stack | Rust + Tauri 2 · WYSIWYG |
| Salud | 🟢 CI en verde en los 4 SO; app arranca en Windows 11 |
| Tests | TS: 336/336 ✅ · Rust: 3/3 + 1 doctest ✅ |
| Cobertura | Modelo + Controlador + Adaptadores: **100 % de líneas** |
| CI | ✅ GitHub Actions: Windows, Ubuntu 22.04, macOS y Fedora |
| Git | https://github.com/IsmaelCaballero/editorMD · F0: PR #1-#4 fusionados y ramas renombradas `hecha/F0.*de4/…` · F1: pasos 1-3 fusionados (PR #5-#7) · `feature/F1-app-controller` (PR #8) |
| Web del proyecto | `docs/index.html` |

## Progreso por fase

| Fase | Estado | Avance |
|---|---|---|
| F0 Decisiones y entorno | ✅ Cerrada (ramas renombradas) | 9/9 |
| F1 Esqueleto MVC | 🔄 En curso (paso 4/5) | 5/5 tareas · falta el cierre |
| F2 Ficheros y codificaciones | ⏳ Pendiente | 0/8 |
| F3 Edición | ⏳ Pendiente | 0/3 |
| F4 Formato | ⏳ Pendiente | 0/4 |
| F5 Objetos | ⏳ Pendiente | 0/3 |
| F6 Importar / Exportar | ⏳ Pendiente | 0/5 |
| F7 Preferencias y extras | ⏳ Pendiente | 0/6 |
| F8 Empaquetado | ⏳ Pendiente | 0/6 |
| F9 QA y release | ⏳ Pendiente | 0/4 |

## Versiones de componentes (resumen)
Implementados: **M00 AppInfo 0.1.0**, **M01 DocumentState 0.2.0**, **M07 MarkdownCodec 0.2.0**, **V02 WysiwygEditorView 0.1.0**, **M17 SemVer 0.1.0**, **V01 MainWindow 0.3.0**, **V12 DialogService 0.1.0**, **C01 AppController 0.1.0**, **A01 TauriBackend 0.1.0**, **A02 TauriWindow 0.1.0**. El resto, en 0.0.0. Ver PLAN.md §4.3.

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

## Bloqueos / pendiente del usuario
- Revisar y fusionar el PR #8 (`feature/F1-app-controller`).

## Próximos pasos
1. PR #8.
2. F1 paso 5/5: `chore/F1-close` (0.0.2 + etiqueta + renombrado).

## Registro de sesiones
| Fecha | Sesión | Hecho |
|---|---|---|
| 2026-10-03 | #1 | Plan inicial; creados PLAN.md, STATUS.md y CONTEXT.md; propuesta de stacks y arquitectura MVC |
| 2026-10-03 | #1 | Decididos D-01 (Tauri), D-02 (WYSIWYG), D-03 (formatos), D-15 (codificaciones y fin de línea). Añadido el soporte .Rmd. Plan reescrito para Tauri |
| 2026-10-03 | #1 | Decididos D-05 (extras + barra de accesos directos), D-07 (MIT), D-11 (Typst), D-12 (modo fuente). Nuevos componentes M14–M16, V09–V11 |
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
