//! Comandos Tauri: el «puente» IPC entre el frontend y el [`crate::model`].
//!
//! Cada comando debe ser una función muy fina que solo traduce
//! argumentos/resultados. La lógica va en el modelo, donde se prueba.
//! Desde TypeScript se invocan con `invoke("nombre_comando", { ...args })`.

use std::path::Path;

use crate::model::app_info::AppInfo;
use crate::model::file_service::{FileError, OpenedFile, SavedFile, open_text, save_text};
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

/// Abre un fichero de texto y lo devuelve decodificado (`encoding` = `None`
/// para detectar la codificación).
///
/// Uso desde TypeScript:
///
/// ```ts
/// import { invoke } from "@tauri-apps/api/core";
/// const opened = await invoke<{ file: { text: string }; readOnly: boolean }>(
///   "open_file",
///   { path: "/ruta/nota.md", encoding: null },
/// );
/// ```
///
/// # Errors
///
/// Un [`FileError`] serializado (ver [`open_text`]).
// Tauri entrega los argumentos por valor; la firma no puede usar `&str`.
#[allow(clippy::needless_pass_by_value)]
#[tauri::command(async)]
pub fn open_file(path: String, encoding: Option<Encoding>) -> Result<OpenedFile, FileError> {
    open_text(Path::new(&path), encoding)
}

/// Guarda un fichero de texto de forma atómica.
///
/// Uso desde TypeScript:
///
/// ```ts
/// import { invoke } from "@tauri-apps/api/core";
/// const saved = await invoke<{ bytesWritten: number }>("save_file", {
///   path: "/ruta/nota.md",
///   text,
///   encoding: "utf-8",
///   lineEnding: "lf",
///   strategy: null,
/// });
/// ```
///
/// # Errors
///
/// Un [`FileError`] serializado (ver [`save_text`]).
// Tauri entrega los argumentos por valor; la firma no puede usar `&str`.
#[allow(clippy::needless_pass_by_value)]
#[tauri::command(async)]
pub fn save_file(
    path: String,
    text: String,
    encoding: Encoding,
    line_ending: LineEnding,
    strategy: Option<LossStrategy>,
) -> Result<SavedFile, FileError> {
    save_text(Path::new(&path), &text, encoding, line_ending, strategy)
}
