# STATUS.md — Estado del proyecto editorMD

> Fotografía del estado actual. Se actualiza **al final de cada sesión de trabajo**.
> Última actualización: 2026-10-04

## ⏸ Punto de reanudación (2026-10-04, sesión #2)

**Dónde estamos:** F0 completa. PR #3 (linters) fusionado (`666d7c3`). Rama **`chore/F0-close`** (paso 4/4 de F0) → **PR #4**: documenta la convención de renombrado de ramas al cerrar la fase.

**Al fusionar el PR #4, con el ok del usuario:** renombrar las 4 ramas de F0 (local + GitHub):

| Paso | Rama actual | Nombre al cerrar F0 | PR |
|---|---|---|---|
| 1/4 | `feature/F0-scaffold` | `hecha/F0.1de4/feature/F0-scaffold` | #1 |
| 2/4 | `chore/release-0.0.1` | `hecha/F0.2de4/chore/release-0.0.1` | #2 |
| 3/4 | `feature/F0-linters` | `hecha/F0.3de4/feature/F0-linters` | #3 |
| 4/4 | `chore/F0-close` | `hecha/F0.4de4/chore/F0-close` | #4 |

**Después:** el usuario revisa el plan detallado de F1 y da su ok → rama `feature/F1-...` (paso 1 de F1).

**Reglas de trabajo** (CONTEXT.md 16-18): el usuario fusiona los PR desde GitHub; las ramas no se borran: se renombran al cerrar la fase; cada etiqueta se enlaza en `docs/index.html`.

## Resumen

| Campo | Valor |
|---|---|
| Fase actual | **F0 — Decisiones y entorno** |
| Versión de la app | **0.0.1** · etiqueta [v0.0.1](https://github.com/IsmaelCaballero/editorMD/releases/tag/v0.0.1) |
| Stack | Rust + Tauri 2 · WYSIWYG |
| Salud | 🟢 CI en verde en los 4 SO; app arranca en Windows 11 |
| Tests | TS: 29/29 ✅ · Rust: 3/3 + 1 doctest ✅ |
| Cobertura | TS Modelo: 97,95 % líneas |
| CI | ✅ GitHub Actions: Windows, Ubuntu 22.04, macOS y Fedora |
| Git | https://github.com/IsmaelCaballero/editorMD · PR #1, #2 y #3 fusionados · `chore/F0-close` (PR #4) · ramas conservadas y renombradas al cerrar cada fase |
| Web del proyecto | `docs/index.html` |

## Progreso por fase

| Fase | Estado | Avance |
|---|---|---|
| F0 Decisiones y entorno | ✅ Completa · cierre en PR #4 | 9/9 |
| F1 Esqueleto MVC | ⏳ Pendiente | 0/5 |
| F2 Ficheros y codificaciones | ⏳ Pendiente | 0/8 |
| F3 Edición | ⏳ Pendiente | 0/3 |
| F4 Formato | ⏳ Pendiente | 0/4 |
| F5 Objetos | ⏳ Pendiente | 0/3 |
| F6 Importar / Exportar | ⏳ Pendiente | 0/5 |
| F7 Preferencias y extras | ⏳ Pendiente | 0/6 |
| F8 Empaquetado | ⏳ Pendiente | 0/6 |
| F9 QA y release | ⏳ Pendiente | 0/4 |

## Versiones de componentes (resumen)
Implementados: **M00 AppInfo 0.1.0**, **M17 SemVer 0.1.0**, **V01 MainWindow 0.1.0** (esqueleto). El resto, en 0.0.0. Ver PLAN.md §4.3.

## Estado de los tests por componente

| Componente | Tests | Pasan | Cobertura |
|---|---|---|---|
| M17 SemVer (TS) | 29 | 29 | 97,95 % |
| M00 AppInfo (Rust) | 3 + 1 doctest | 4 | — |

## Bloqueos / pendiente del usuario
- Revisar y fusionar el PR #4 (`chore/F0-close`) y dar el ok al renombrado de las ramas de F0.
- Dar el ok al plan de F1.

## Próximos pasos
1. PR #4 y renombrado de las ramas de F0.
2. F1 (esqueleto MVC con Milkdown), tras el ok del usuario al plan.

## Registro de sesiones
| Fecha | Sesión | Hecho |
|---|---|---|
| 2026-10-03 | #1 | Plan inicial; creados PLAN.md, STATUS.md y CONTEXT.md; propuesta de stacks y arquitectura MVC |
| 2026-10-03 | #1 | Decididos D-01 (Tauri), D-02 (WYSIWYG), D-03 (formatos), D-15 (codificaciones y fin de línea). Añadido el soporte .Rmd. Plan reescrito para Tauri |
| 2026-10-03 | #1 | Decididos D-05 (extras + barra de accesos directos), D-07 (MIT), D-11 (Typst), D-12 (modo fuente). Nuevos componentes M14–M16, V09–V11 |
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
