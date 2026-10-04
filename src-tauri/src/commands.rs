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

/// Abre un documento de texto (delega en [`file_service::open_text`]).
///
/// Con `encoding` nulo la codificación se detecta. Se ejecuta fuera del hilo
/// principal para no bloquear la interfaz con ficheros grandes.
///
/// Uso desde TypeScript:
///
/// ```ts
/// import { invoke } from "@tauri-apps/api/core";
/// try {
///   const opened = await invoke<OpenedFile>("open_file", { path, encoding: null });
///   editor.load(opened.file.text, opened.file.encoding, opened.readOnly);
/// } catch (err) {
///   // err: { kind: "not-found" | "decode" | ..., path: string, ... }
/// }
/// ```
///
/// # Errors
///
/// Los de [`file_service::open_text`].
#[tauri::command(async)]
#[allow(clippy::needless_pass_by_value)] // Tauri entrega los argumentos por valor.
pub fn open_file(path: String, encoding: Option<Encoding>) -> Result<OpenedFile, FileError> {
    file_service::open_text(Path::new(&path), encoding)
}

/// Guarda un documento de texto (delega en [`file_service::save_text`]).
///
/// Sin `strategy`, si hay caracteres que no caben en `encoding` devuelve un
/// error `unmappable` con el informe de pérdidas y no escribe nada. Se ejecuta
/// fuera del hilo principal.
///
/// Uso desde TypeScript:
///
/// ```ts
/// import { invoke } from "@tauri-apps/api/core";
/// const saved = await invoke<SavedFile>("save_file", {
///   path, text, encoding: "utf-8", lineEnding: "crlf", strategy: null,
/// });
/// console.log(saved.bytesWritten, saved.losses);
/// ```
///
/// # Errors
///
/// Los de [`file_service::save_text`].
#[tauri::command(async)]
#[allow(clippy::needless_pass_by_value)] // Tauri entrega los argumentos por valor.
pub fn save_file(
    path: String,
    text: String,
    encoding: Encoding,
    line_ending: LineEnding,
    strategy: Option<LossStrategy>,
) -> Result<SavedFile, FileError> {
    file_service::save_text(Path::new(&path), &text, encoding, line_ending, strategy)
}
