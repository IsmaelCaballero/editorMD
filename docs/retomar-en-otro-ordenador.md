# Cómo retomar editorMD en otro ordenador

> Guía para continuar el desarrollo en un equipo distinto del original, partiendo de cero.
> Escrita al cerrar la sesión A de F2 (2026-10-04). Se puede seguir paso a paso aunque tengas poca experiencia con Git o con Claude Code.

## 0. Por qué funciona

Todo el estado del proyecto está **en el repositorio de GitHub**: el código, el plan (PLAN.md), el estado (STATUS.md), el contexto y las normas (CONTEXT.md), las métricas de IA y los resultados del experimento. Claude Code **no guarda nada útil en el otro ordenador**: su «memoria» y las transcripciones de las sesiones son locales. Por eso, al empezar una sesión, se le pide que lea esos ficheros. Así recupera el contexto completo sin depender de la conversación anterior.

## 1. Instalar las herramientas (solo la primera vez)

| Herramienta | Versión usada | Para qué | Cómo comprobarlo |
|---|---|---|---|
| Git | 2.51 o superior | Control de versiones | `git --version` |
| Node.js (LTS) | 24.x | Frontend, pruebas (Vitest), lint | `node -v` y `npm -v` |
| Rust (rustup) | 1.99 (mínimo 1.90) | Backend (Tauri) | `cargo -V` |
| Dependencias de Tauri del SO | — | Compilar la app de escritorio | ver abajo |
| GitHub CLI (`gh`) | 2.102 o superior | Abrir PR, ver la CI | `gh --version` |
| Claude Code | la última | El asistente | `claude --version` |

**Dependencias de Tauri según el sistema** (guía oficial: <https://v2.tauri.app/start/prerequisites/>):
- **Windows 10/11:** «Visual Studio Build Tools» con la carga de trabajo *Desarrollo para el escritorio con C++* y WebView2 (ya viene en Windows 11). Rust con la *toolchain* MSVC (la opción por defecto de `rustup`).
- **Ubuntu / Debian:** `sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`
- **Fedora:** `sudo dnf install webkit2gtk4.1-devel openssl-devel curl wget file libappindicator-gtk3-devel librsvg2-devel` y `sudo dnf group install "c-development"`
- **macOS:** `xcode-select --install`

Después de instalar, **cierra y vuelve a abrir el terminal** para que se actualice el PATH.

## 2. Identificarte (solo la primera vez)

```bash
git config --global user.name "Ismael Caballero"
git config --global user.email "<tu correo de GitHub>"
gh auth login          # GitHub.com → HTTPS → «Login with a web browser»
claude                 # la primera vez te pide iniciar sesión con tu cuenta de Claude
```

`gh auth login` también configura Git para que `git push` use tus credenciales de GitHub.

## 3. Clonar el repositorio y comprobar que todo funciona

```bash
cd <carpeta donde guardas tus proyectos>        # p. ej. C:\ProgsConIA
git clone https://github.com/IsmaelCaballero/editorMD.git
cd editorMD
git log --oneline -5      # debe aparecer el merge del último PR fusionado
npm install               # dependencias del frontend (crea node_modules/)
npm test                  # pruebas TS: todas en verde
cargo test --manifest-path src-tauri/Cargo.toml   # pruebas Rust (la 1.ª compilación tarda varios minutos)
npm run lint              # ESLint + Prettier
```

Si todo pasa, el entorno está listo. Opcional: `npm run tauri dev` abre la aplicación.

**Sobre las ramas:** al clonar solo tienes `main` en local, pero todas las ramas existen en GitHub (`git branch -r` las lista). No hace falta descargarlas: los pasos ya hechos están fusionados en `main`, y cada paso nuevo crea su propia rama desde `main`.

## 4. Abrir la sesión de Claude Code y pegar el prompt de arranque

Desde la carpeta `editorMD`:

```bash
claude
```

Copia el prompt que figura en **STATUS.md → «⏸ Punto de reanudación»**. Siempre es el más actualizado. Al cerrar la sesión A era este:

> Retomamos editorMD, F2 sesión B, en un ordenador nuevo recién clonado. Lee CONTEXT.md, STATUS.md, PLAN.md (§7 F2) y docs/ai-usage/README.md. Comprueba el estado de git y de los PR abiertos, verifica que el entorno funciona (npm test, cargo test, npm run lint) y preséntame el paso 4/8 (FileService con el experimento RQ.1 T2) con su estimación de tokens. No empieces hasta que te dé el ok.

## 5. Por qué una sesión nueva

- **Coste.** En cada llamada, Claude vuelve a leer toda la conversación. Al final de la sesión A cada llamada releía unos 315 000 tokens, frente a unos 60 000 al principio. La misma tarea cuesta varias veces más al final de una sesión larga (ver `docs/ai-usage/README.md` §3).
- **Calidad.** Un contexto corto y ordenado (los documentos del proyecto) es más fiable que una conversación larga llena de salidas de comandos antiguas.
- **Continuidad garantizada.** CONTEXT, STATUS y PLAN existen precisamente para esto: si con ellos se puede retomar el trabajo, el proyecto no depende de la memoria de nadie. Esto vale para Claude y también para un compañero de equipo.

## 6. Normas que Claude seguirá (resumen de CONTEXT.md)

- **Tú fusionas los PR** desde GitHub. Claude abre el PR, espera a que la CI esté en verde y te avisa con una guía de revisión.
- **Las ramas no se borran.** Al cerrar la fase se renombran a `hecha/F<m>.<n>de<T>/…`, con tu ok.
- En cada paso se hace una **estimación previa de tokens** y después se mide el coste real.
- Antes de medir con la aplicación abierta, Claude te avisará para que no toques la ventana.

## 7. Si algo falla

| Síntoma | Causa probable | Solución |
|---|---|---|
| `cargo` o `gh` «no se reconoce» | PATH sin actualizar | Cierra y abre el terminal |
| `cargo test` falla al enlazar en Windows | Faltan las Build Tools de C++ | Instálalas (§1) |
| `npm test` falla nada más clonar | Faltan dependencias | `npm install` |
| `git push` pide usuario y contraseña | `gh` sin autenticar | `gh auth login` |
| La primera compilación de Rust tarda mucho | Es normal (compila Tauri). En Windows, el antivirus puede ralentizarla | Espera; las siguientes son rápidas |
