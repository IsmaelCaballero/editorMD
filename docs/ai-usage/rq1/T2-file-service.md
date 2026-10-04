# RQ.1 · Tarea T2 — M08 `FileService` y comandos `open_file`/`save_file` (F2 paso 4/8)

> Especificación cerrada del experimento RQ.1 (`docs/ai-usage/README.md` §4.2).
> Es **el mismo encargo** para todas las ejecuciones (3 × Opus 5.5 y 3 × Sonnet 5.5).
> La aceptación se decide con **pruebas ocultas** que no forman parte del repositorio,
> además de `cargo test`, clippy (pedantic, `-D warnings`) y rustfmt.

## 1. Contexto

editorMD es un editor Markdown (Rust + Tauri 2, ver `CONTEXT.md` y `PLAN.md` §2.5 y §4).
El componente **M09 `TextCodec`** (`src-tauri/src/model/text_codec.rs` y su carpeta) ya
convierte bytes ⇄ texto: `read_text`, `read_text_as`, `write_text`, `write_text_lossy`,
`loss_report`. **No lo modifiques.**

Esta tarea crea **M08 `FileService`** (Modelo · Rust): el acceso al disco (lectura, escritura
atómica, solo lectura) y su combinación con M09 para abrir y guardar documentos de texto; y
expone eso al frontend con dos **comandos Tauri**. El frontend (TypeScript) **no** forma parte
de esta tarea.

## 2. Qué hay que entregar

1. Nuevo módulo `src-tauri/src/model/file_service.rs` (puede tener submódulos en
   `src-tauri/src/model/file_service/`), declarado en `src-tauri/src/model/mod.rs` como
   `pub mod file_service;`. Ruta pública: `editormd_lib::model::file_service`.
2. Dos comandos en `src-tauri/src/commands.rs` registrados en `generate_handler!` de
   `src-tauri/src/lib.rs`.
3. `tempfile = "3"` como **dependencia de desarrollo** en `src-tauri/Cargo.toml` (las pruebas
   ocultas la usan). Otras dependencias solo si son imprescindibles y justificadas.
   Actualiza `Cargo.lock`.
4. Pruebas unitarias en el propio módulo (`#[cfg(test)] mod tests`) que cubran esta
   especificación, incluidos los casos borde y los errores. Las pruebas trabajan en carpetas
   temporales (`tempfile::tempdir()`), nunca en el árbol del repositorio.
5. Documentación rustdoc de todo elemento público (el crate tiene `#![warn(missing_docs)]`),
   con al menos un ejemplo (doctest) en `write_atomic` y en `open_text`. El fichero empieza con
   `//! **M08 `FileService`** (Modelo · Rust) — …` como `app_info.rs` y `text_codec.rs`.
   Documentación y comentarios en **español**; identificadores en inglés.

## 3. API pública (obligatoria, exacta)

```rust
use std::path::Path;
use crate::model::text_codec::{Encoding, LineEnding, LossReport, LossStrategy, TextFile};

/// Error al abrir o guardar un fichero. Serializable para el frontend.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", rename_all_fields = "camelCase")]
pub enum FileError {
    /// El fichero (o, al escribir, su carpeta) no existe.
    NotFound { path: String },
    /// El sistema operativo deniega el acceso.
    PermissionDenied { path: String },
    /// El fichero existe y está marcado como de solo lectura (no se intenta escribir).
    ReadOnly { path: String },
    /// La ruta es una carpeta.
    IsDirectory { path: String },
    /// Cualquier otro error de E/S; `message` = el `Display` del `std::io::Error`.
    Io { path: String, message: String },
    /// El contenido no es válido en la codificación usada. `offset` = el del
    /// `DecodeError::InvalidSequence` (`None` en `OddLength`); `message` = el `Display`
    /// del `DecodeError`.
    Decode { path: String, encoding: Encoding, offset: Option<usize>, message: String },
    /// Al guardar sin estrategia, hay caracteres que no caben en la codificación.
    /// `report` = `loss_report(text, encoding)` del texto recibido.
    Unmappable { path: String, report: LossReport },
}
// FileError implementa std::fmt::Display (mensaje en español que incluye la ruta)
// y std::error::Error.

/// Bytes de un fichero leído del disco.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileBytes { pub bytes: Vec<u8>, pub read_only: bool }

/// Documento de texto abierto, listo para el editor.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenedFile { pub file: TextFile, pub read_only: bool }

/// Resultado de guardar.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedFile { pub bytes_written: usize, pub losses: Option<LossReport> }

pub fn read_file(path: &Path) -> Result<FileBytes, FileError>;
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), FileError>;
pub fn open_text(path: &Path, encoding: Option<Encoding>) -> Result<OpenedFile, FileError>;
pub fn save_text(
    path: &Path,
    text: &str,
    encoding: Encoding,
    line_ending: LineEnding,
    strategy: Option<LossStrategy>,
) -> Result<SavedFile, FileError>;
```

En todos los errores, `path` es `path.display().to_string()` de la ruta recibida.

**Serialización (serde)** de `FileError`, por ejemplo:
`{"kind":"not-found","path":"…"}`, `{"kind":"is-directory","path":"…"}`,
`{"kind":"io","path":"…","message":"…"}`,
`{"kind":"decode","path":"…","encoding":"ascii","offset":3,"message":"…"}`,
`{"kind":"unmappable","path":"…","report":{…}}`.
`OpenedFile` → `{"file":{…TextFile…},"readOnly":false}`;
`SavedFile` → `{"bytesWritten":12,"losses":null}`.

## 4. Comportamiento

### 4.1 `read_file(path)`
1. Si la ruta es una carpeta → `IsDirectory` (compruébalo antes de abrir: en Windows abrir una
   carpeta da «acceso denegado»).
2. Lee todo el contenido. `read_only` = `metadata.permissions().readonly()`.
3. Errores de E/S → `NotFound` (`ErrorKind::NotFound`), `PermissionDenied`
   (`ErrorKind::PermissionDenied`) o `Io` (el resto).

### 4.2 `write_atomic(path, bytes)` — escritura atómica
1. Si la ruta es una carpeta → `IsDirectory`. Si el fichero existe y es de solo lectura →
   `ReadOnly`. En ambos casos no se crea ni se modifica nada.
2. Escribe los bytes en un fichero **temporal en la misma carpeta** que el destino,
   lo vuelca al disco (`sync_all`) y lo **renombra** sobre el destino (sustituyéndolo si existe).
3. Si cualquier paso falla, el destino queda **intacto** (contenido anterior o inexistente) y
   **no queda ningún fichero temporal** en la carpeta. Tras un éxito, en la carpeta solo hay
   el destino y lo que ya había antes (ningún temporal).
4. Si la carpeta del destino no existe → `NotFound` (no se crean carpetas).
5. En Unix, si el destino ya existía, el resultado conserva sus permisos (modo).
   Si es nuevo, los permisos por defecto del sistema.
6. Mapeo de errores de E/S como en 4.1.
7. Los enlaces simbólicos quedan fuera de alcance (no hace falta tratarlos de forma especial).

### 4.3 `open_text(path, encoding)`
- `read_file`; después `read_text` (detección automática) si `encoding` es `None`, o
  `read_text_as` si es `Some`. Devuelve el `TextFile` y el `read_only` del fichero.
- Un `DecodeError` se convierte en `FileError::Decode` (ver §3).

### 4.4 `save_text(path, text, encoding, line_ending, strategy)`
1. **Primero se preparan los bytes, sin tocar el disco:**
   - Si `loss_report(text, encoding)` no tiene pérdidas: `write_text` → `losses: None`.
   - Si las tiene y `strategy` es `None` → `Unmappable` con ese informe; no se crea ni se
     modifica ningún fichero.
   - Si las tiene y `strategy` es `Some(s)` → `write_text_lossy(…, s)` → `losses: Some(informe)`.
2. Después `write_atomic`. `bytes_written` = longitud de los bytes escritos (BOM incluido).
3. Ida y vuelta: para un `OpenedFile` sin editar, `save_text(path, &f.text, f.encoding,
   f.line_ending, None)` deja el fichero **idéntico byte a byte** (salvo finales de línea
   mezclados, que quedan normalizados).

### 4.5 Comandos Tauri (en `commands.rs`, sin lógica: solo delegan)
```rust
#[tauri::command(async)]
pub fn open_file(path: String, encoding: Option<Encoding>) -> Result<OpenedFile, FileError>;

#[tauri::command(async)]
pub fn save_file(
    path: String,
    text: String,
    encoding: Encoding,
    line_ending: LineEnding,
    strategy: Option<LossStrategy>,
) -> Result<SavedFile, FileError>;
```
`(async)` hace que se ejecuten fuera del hilo principal (no bloquean la interfaz con ficheros
grandes). Desde TypeScript: `invoke("save_file", { path, text, encoding, lineEnding, strategy })`.
Documenta en cada uno un ejemplo de uso desde TS, como en `app_info`.

## 5. Criterios de aceptación
- Pruebas ocultas de aceptación (las ejecuta el orquestador al final) en verde.
- `cargo test --manifest-path src-tauri/Cargo.toml` en verde (incluidos los doctests).
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` sin avisos
  (el crate ya activa `pedantic`).
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check` sin cambios.
- Sin `unsafe` (el crate tiene `unsafe_code = "forbid"`), sin `unwrap()`/`expect()` fuera de
  las pruebas.
- Las pruebas que cambian permisos (solo lectura) los restauran al terminar, para que la
  carpeta temporal se pueda borrar.
- Cobertura alta de las ramas del módulo con tus propias pruebas.

## 6. Forma de trabajar
- Trabaja **solo** dentro de tu *worktree*. No toques otros directorios del disco (salvo las
  carpetas temporales de las pruebas).
- Proyecto con **TDD**: primer commit con las pruebas en rojo (`test(M08): …`), después la
  implementación en verde (`feat(M08): …`) y, si hace falta, `refactor`/`style`. Mensajes en
  Conventional Commits, en español.
- No modifiques documentación del proyecto (PLAN.md, STATUS.md, web…): eso lo hace el orquestador.
- No hagas `push` ni abras PR.
- Para `cargo`, usa un *timeout* largo (hasta 600 000 ms): la primera compilación de Tauri
  en un *worktree* nuevo tarda varios minutos.
