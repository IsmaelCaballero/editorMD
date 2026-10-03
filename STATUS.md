# STATUS.md — Estado del proyecto editorMD

> Fotografía del estado actual. Se actualiza **al final de cada sesión de trabajo**.
> Última actualización: 2026-10-03

## Resumen

| Campo | Valor |
|---|---|
| Fase actual | **F0 — Decisiones y entorno** |
| Versión de la app | 0.0.1 (esqueleto F0, rama `feature/F0-scaffold`) |
| Stack | Rust + Tauri 2 · WYSIWYG |
| Salud | 🟡 Todas las decisiones tomadas; **bloqueado por falta de Rust y MSVC Build Tools** para compilar el backend |
| Tests | TS: 29/29 ✅ · Rust: 3 unitarios + 1 doctest escritos, sin ejecutar (falta Rust) |
| Cobertura | TS Modelo: 97,95 % líneas |
| CI | — (no configurado; F0.6) |
| Git | `main` (1 commit) · `feature/F0-scaffold` (abierta, PR pendiente de aprobación) |
| Web del proyecto | `docs/index.html` |

## Progreso por fase

| Fase | Estado | Avance |
|---|---|---|
| F0 Decisiones y entorno | 🔄 En curso | 4/9 (4 en curso) |
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
| M00 AppInfo (Rust) | 3 + 1 doctest | sin ejecutar | — |

## Bloqueos / pendiente del usuario
- **Instalar Visual Studio Build Tools 2022 (C++) y Rust (rustup)**: requieren permisos de administrador.
- **Aprobar el PR** de `feature/F0-scaffold` → `main`.

## Próximos pasos
1. Instalar Rust y Build Tools → `npm run tauri dev`, `cargo test`, `npm run docs:rust`.
2. F0.6 CI en GitHub Actions (cuando exista el remoto) y F0.7 linters.
3. PR de `feature/F0-scaffold` y, después, F1 (esqueleto MVC con Milkdown).

## Registro de sesiones
| Fecha | Sesión | Hecho |
|---|---|---|
| 2026-10-03 | #1 | Plan inicial; creados PLAN.md, STATUS.md y CONTEXT.md; propuesta de stacks y arquitectura MVC |
| 2026-10-03 | #1 | Decididos D-01 (Tauri), D-02 (WYSIWYG), D-03 (formatos), D-15 (codificaciones y fin de línea). Añadido el soporte .Rmd. Plan reescrito para Tauri |
| 2026-10-03 | #1 | Decididos D-05 (extras + barra de accesos directos), D-07 (MIT), D-11 (Typst), D-12 (modo fuente). Nuevos componentes M14–M16, V09–V11 |
| 2026-10-03 | #1 | Convenciones (SemVer 2.0.0, rustdoc/TSDoc, rama por feature + PR, web). Git init + commit en main. Rama `feature/F0-scaffold`: Vite+Svelte+TS, Tauri 2, M00 AppInfo, M17 SemVer (29 tests), TypeDoc, .gitattributes, web `docs/index.html` |

## Checklist de release (se usará en F9)
- [ ] Intercambio entre SO: un fichero creado en Windows (CRLF, Windows-1252/UTF-8 BOM) se abre bien en Linux/macOS y viceversa
- [ ] Windows 11: instalar, abrir .md con doble clic, editar, guardar, exportar PDF, desinstalar
- [ ] Ubuntu: ídem (.deb y AppImage)
- [ ] Fedora: ídem (.rpm)
- [ ] macOS: ídem (.dmg, Intel y Apple Silicon)
- [ ] Objetivos de RAM, arranque y tamaño cumplidos
