# STATUS.md — Estado del proyecto editorMD

> Fotografía del estado actual. Se actualiza **al final de cada sesión de trabajo**.
> Última actualización: 2026-10-03

## ⏸ Punto de reanudación (2026-10-03, fin de la sesión #1)

**Dónde estamos:** F0 casi terminada. Rama activa `feature/F0-scaffold` (árbol limpio). PR → `main` **en espera**: el usuario lo aprobará cuando se suba a GitHub y la CI esté en verde.

**Comprobado al cerrar:** Rust 1.99 + cargo, VS Build Tools 2026 (MSVC) y `gh` 2.102 instalados. `cargo test`: 3 tests unitarios + 1 doctest ✅. `npm test`: 29 ✅. `npm run build` ✅. `npm run docs:rust`: sin avisos de `missing_docs` ✅.

**Al retomar (en orden):**
1. El usuario ejecuta `! gh auth login` (aún **no** se ha autenticado).
2. `npm run tauri dev`: abrir por primera vez la ventana nativa y comprobar que se ve.
3. Crear el repositorio **público** (`gh repo create editorMD --public --source . --remote origin`), hacer `git push -u origin main feature/F0-scaffold` y abrir el PR (`gh pr create`).
4. Esperar a que la CI esté en verde en los 4 SO y **preguntar al usuario** antes de fusionar. Después, etiquetar y pasar a F1.
5. Actualizar `docs/index.html` y este fichero.

## Resumen

| Campo | Valor |
|---|---|
| Fase actual | **F0 — Decisiones y entorno** |
| Versión de la app | 0.0.1 (esqueleto F0, rama `feature/F0-scaffold`) |
| Stack | Rust + Tauri 2 · WYSIWYG |
| Salud | 🟢 Toolchain completa; backend compilado y probado. Falta subir a GitHub |
| Tests | TS: 29/29 ✅ · Rust: 3/3 + 1 doctest ✅ |
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
| M00 AppInfo (Rust) | 3 + 1 doctest | 4 | — |

## Bloqueos / pendiente del usuario
- Autenticarse en GitHub (`gh auth login`) para crear el repositorio **público**.
- PR de `feature/F0-scaffold` → `main`: **el usuario decidió esperar a que Rust compile y pasen los tests**.

## Próximos pasos
1. `gh auth login` → crear el repositorio y abrir el PR de F0; `npm run tauri dev`.
2. F0.6 CI en GitHub Actions (cuando exista el remoto) y F0.7 linters.
3. PR de `feature/F0-scaffold` y, después, F1 (esqueleto MVC con Milkdown).

## Registro de sesiones
| Fecha | Sesión | Hecho |
|---|---|---|
| 2026-10-03 | #1 | Plan inicial; creados PLAN.md, STATUS.md y CONTEXT.md; propuesta de stacks y arquitectura MVC |
| 2026-10-03 | #1 | Decididos D-01 (Tauri), D-02 (WYSIWYG), D-03 (formatos), D-15 (codificaciones y fin de línea). Añadido el soporte .Rmd. Plan reescrito para Tauri |
| 2026-10-03 | #1 | Decididos D-05 (extras + barra de accesos directos), D-07 (MIT), D-11 (Typst), D-12 (modo fuente). Nuevos componentes M14–M16, V09–V11 |
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
