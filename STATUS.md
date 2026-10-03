# STATUS.md — Estado del proyecto editorMD

> Fotografía del estado actual. Se actualiza **al final de cada sesión de trabajo**.
> Última actualización: 2026-10-03

## Resumen

| Campo | Valor |
|---|---|
| Fase actual | **F0 — Decisiones y entorno** |
| Versión de la app | 0.0.0 (sin código) |
| Stack | Rust + Tauri 2 · WYSIWYG |
| Salud | 🟢 Casi todo decidido; faltan D-08 (firma), D-13 (Svelte?), D-14 (Milkdown?) |
| Tests | — (aún no hay código) |
| Cobertura | — |
| CI | — (no configurado) |

## Progreso por fase

| Fase | Estado | Avance |
|---|---|---|
| F0 Decisiones y entorno | 🔄 En curso | 1/7 |
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
Todos los componentes (M01–M16, V01–V11, C01–C06) están en **0.0.0**. Ver PLAN.md §4.3.

## Estado de los tests por componente

| Componente | Tests | Pasan | Cobertura |
|---|---|---|---|
| *(ninguno aún)* | | | |

## Bloqueos / pendiente del usuario
- Firma de código (D-08); confirmar Svelte (D-13) y Milkdown (D-14).

## Próximos pasos
1. Cerrar las decisiones de F0.
2. Instalar la toolchain en Windows: Rust, Node LTS, Tauri CLI y Pandoc (opcional).
3. Inicializar git, crear la estructura de carpetas, configurar el CI y una ventana «hola mundo».

## Registro de sesiones
| Fecha | Sesión | Hecho |
|---|---|---|
| 2026-10-03 | #1 | Plan inicial; creados PLAN.md, STATUS.md y CONTEXT.md; propuesta de stacks y arquitectura MVC |
| 2026-10-03 | #1 | Decididos D-01 (Tauri), D-02 (WYSIWYG), D-03 (formatos), D-15 (codificaciones y fin de línea). Añadido el soporte .Rmd. Plan reescrito para Tauri |
| 2026-10-03 | #1 | Decididos D-05 (extras + barra de accesos directos), D-07 (MIT), D-11 (Typst), D-12 (modo fuente). Nuevos componentes M14–M16, V09–V11 |

## Checklist de release (se usará en F9)
- [ ] Intercambio entre SO: un fichero creado en Windows (CRLF, Windows-1252/UTF-8 BOM) se abre bien en Linux/macOS y viceversa
- [ ] Windows 11: instalar, abrir .md con doble clic, editar, guardar, exportar PDF, desinstalar
- [ ] Ubuntu: ídem (.deb y AppImage)
- [ ] Fedora: ídem (.rpm)
- [ ] macOS: ídem (.dmg, Intel y Apple Silicon)
- [ ] Objetivos de RAM, arranque y tamaño cumplidos
