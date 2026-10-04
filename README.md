# editorMD

Editor Markdown **WYSIWYG** de escritorio, ligero y multiplataforma (Windows, Ubuntu, Fedora, macOS), construido con **Rust + Tauri 2 + Svelte 5 + Milkdown** y arquitectura **MVC**.

> Estado: desarrollo inicial (`0.x`). Consulta [STATUS.md](STATUS.md).

## Documentación del proyecto
| Fichero | Para qué |
|---|---|
| [CONTEXT.md](CONTEXT.md) | Contexto, requisitos y decisiones. **Léelo primero.** |
| [docs/retomar-en-otro-ordenador.md](docs/retomar-en-otro-ordenador.md) | Cómo instalar, clonar y continuar el desarrollo en otro equipo |
| [STATUS.md](STATUS.md) | Estado actual y registro de sesiones |
| [PLAN.md](PLAN.md) | Plan por fases, arquitectura, componentes y versiones, convenciones |
| [CHANGELOG.md](CHANGELOG.md) | Cambios por versión (Keep a Changelog + SemVer 2.0.0) |
| `docs/index.html` | Web del proyecto: plan, progreso y diario de aprendizaje |

## Versiones y ramas
- Versiones publicadas (SemVer 2.0.0): [Releases](https://github.com/IsmaelCaballero/editorMD/releases). La primera es [v0.0.1](https://github.com/IsmaelCaballero/editorMD/releases/tag/v0.0.1).
- Flujo: una rama por feature → pull request → CI en 4 SO → fusión revisada por una persona.
- Al cerrar cada fase, sus ramas se renombran a `hecha/F<m>.<n>de<T>/<nombre>` (paso *n* de *T* de la fase *m*): la lista de ramas muestra la secuencia temporal del desarrollo.
- En un desarrollo normal se recomienda **borrar las ramas una vez fusionadas**. En este proyecto **se conservan deliberadamente**: por razones académicas (para mostrar a los alumnos el proceso completo) y por **trazabilidad del esfuerzo de desarrollo**, ya que en el futuro el trabajo se repartirá entre varios miembros del equipo, cada uno trabajando sobre una parte del sistema con su propio arnés.

## Licencia
[MIT](LICENSE)
