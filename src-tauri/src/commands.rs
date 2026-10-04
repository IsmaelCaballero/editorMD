//! Comandos Tauri: el «puente» IPC entre el frontend y el [`crate::model`].
//!
//! Cada comando debe ser una función muy fina que solo traduce
//! argumentos/resultados. La lógica va en el modelo, donde se prueba.
//! Desde TypeScript se invocan con `invoke("nombre_comando", { ...args })`.

use std::path::Path;

use crate::model::app_info::AppInfo;
use crate::model::file_service::{self, FileError, OpenedFile, SavedFile};
use crate::model::text_codec::{Encoding, LineEnding, LossStrategy};

/// Devuelve el nombre y la versión de la aplicación al frontend.
///
/// Uso desde TypeScript:
///
/// ```ts
/// import { invoke } from "@tauri-apps/api/core";
/// const info = await invoke<{ name: string; version: string }>("app_info");
/// ```
#[tauri::command]
pub fn app_info() -> AppInfo {
    AppInfo::current()
}

/// Abre un fichero de texto y lo devuelve listo para el editor.
///
/// Con `encoding` a `None` se detecta la codificación. Delega en
/// [`crate::model::file_service::open_text`]; se ejecuta fuera del hilo
/// principal para no bloquear la interfaz con ficheros grandes.
///
/// Uso desde TypeScript:
///
/// ```ts
/// import { invoke } from "@tauri-apps/api/core";
/// const opened = await invoke<{ file: { text: string; encoding: string; lineEnding: string }; readOnly: boolean }>(
///   "open_file",
///   { path: "/ruta/nota.md", encoding: null },
/// );
/// ```
///
/// # Errors
///
/// Devuelve un [`FileError`] (serializado como `{ kind, path, … }`).
#[tauri::command(async)]
#[allow(clippy::needless_pass_by_value)] // Tauri exige argumentos con propiedad (async)
pub fn open_file(path: String, encoding: Option<Encoding>) -> Result<OpenedFile, FileError> {
    file_service::open_text(Path::new(&path), encoding)
}

/// Guarda un texto en disco de forma atómica.
///
/// Delega en [`crate::model::file_service::save_text`]; se ejecuta fuera del
/// hilo principal. `strategy` a `null` falla si hay caracteres que no caben
/// en la codificación.
///
/// Uso desde TypeScript:
///
/// ```ts
/// import { invoke } from "@tauri-apps/api/core";
/// const saved = await invoke<{ bytesWritten: number; losses: unknown | null }>(
///   "save_file",
///   { path: "/ruta/nota.md", text, encoding: "utf-8", lineEnding: "lf", strategy: null },
/// );
/// ```
///
/// # Errors
///
/// Devuelve un [`FileError`] (serializado como `{ kind, path, … }`).
#[tauri::command(async)]
#[allow(clippy::needless_pass_by_value)] // Tauri exige argumentos con propiedad (async)
pub fn save_file(
    path: String,
    text: String,
    encoding: Encoding,
    line_ending: LineEnding,
    strategy: Option<LossStrategy>,
) -> Result<SavedFile, FileError> {
    file_service::save_text(Path::new(&path), &text, encoding, line_ending, strategy)
}
