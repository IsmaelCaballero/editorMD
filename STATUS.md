# STATUS.md — Estado del proyecto editorMD

> Fotografía del estado actual. Se actualiza **al final de cada sesión de trabajo**.
> Última actualización: 2026-10-04

## ⏸ Punto de reanudación (2026-10-04, sesión #2)

**Dónde estamos:** F0 cerrada salvo **F0.7 (linters)**. PR #1 fusionado por el usuario en GitHub (merge `1b4d036`). Rama `chore/release-0.0.1` con CHANGELOG `[0.0.1]` y la actualización de la documentación → PR #2. **Tras fusionarlo: etiquetar `v0.0.1`** sobre el merge.

**Repositorio:** https://github.com/IsmaelCaballero/editorMD · CI: Actions (Windows, Ubuntu 22.04, macOS, Fedora).

**Siguientes pasos:**
1. PR #2 (`chore/release-0.0.1`) → aprobación del usuario → `git tag -a v0.0.1` + `git push origin v0.0.1` (+ GitHub Release).
2. Rama `feature/F0-linters`: rustfmt, clippy (`-D warnings`), ESLint + Prettier, integrados en la CI.
3. Rama `feature/F1-mvc-skeleton`: interfaces, M01 DocumentState, M07 MarkdownCodec + corpus de fidelidad, Milkdown, C01 AppController.

## Resumen

| Campo | Valor |
|---|---|
| Fase actual | **F0 — Decisiones y entorno** |
| Versión de la app | 0.0.1 (en `main`; etiqueta `v0.0.1` pendiente del PR #2) |
| Stack | Rust + Tauri 2 · WYSIWYG |
| Salud | 🟢 CI en verde en los 4 SO; app arranca en Windows 11 |
| Tests | TS: 29/29 ✅ · Rust: 3/3 + 1 doctest ✅ |
| Cobertura | TS Modelo: 97,95 % líneas |
| CI | ✅ GitHub Actions: Windows, Ubuntu 22.04, macOS y Fedora |
| Git | https://github.com/IsmaelCaballero/editorMD · `main` con PR #1 fusionado · `chore/release-0.0.1` (PR #2) |
| Web del proyecto | `docs/index.html` |

## Progreso por fase

| Fase | Estado | Avance |
|---|---|---|
| F0 Decisiones y entorno | 🔄 Casi cerrada | 8/9 (falta F0.7) |
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
- Aprobar el PR #2 (`chore/release-0.0.1`).

## Próximos pasos
1. PR #2 y etiqueta `v0.0.1`.
2. F0.7 linters.
3. F1 (esqueleto MVC con Milkdown).

## Registro de sesiones
| Fecha | Sesión | Hecho |
|---|---|---|
| 2026-10-03 | #1 | Plan inicial; creados PLAN.md, STATUS.md y CONTEXT.md; propuesta de stacks y arquitectura MVC |
| 2026-10-03 | #1 | Decididos D-01 (Tauri), D-02 (WYSIWYG), D-03 (formatos), D-15 (codificaciones y fin de línea). Añadido el soporte .Rmd. Plan reescrito para Tauri |
| 2026-10-03 | #1 | Decididos D-05 (extras + barra de accesos directos), D-07 (MIT), D-11 (Typst), D-12 (modo fuente). Nuevos componentes M14–M16, V09–V11 |
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
