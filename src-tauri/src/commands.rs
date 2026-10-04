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
/// Con `encoding` nulo detecta la codificación; con un identificador
/// (p. ej. `"iso-8859-1"`) la fuerza («Reabrir con codificación…»). Se
/// ejecuta fuera del hilo principal para no bloquear la interfaz.
///
/// Uso desde TypeScript:
///
/// ```ts
/// import { invoke } from "@tauri-apps/api/core";
/// try {
///   const opened = await invoke<OpenedFile>("open_file", { path, encoding: null });
///   // opened = { file: { text, encoding, lineEnding, mixedLineEndings, detection }, readOnly }
/// } catch (e) {
///   // e = { kind: "not-found" | "permission-denied" | "is-directory" | "io" | "decode", path, ... }
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
/// Con `strategy` nulo, si hay caracteres que no caben en `encoding` falla
/// con `{ kind: "unmappable", path, report }` sin tocar el disco; el
/// frontend puede mostrar el informe y repetir con una estrategia
/// (`"replace"`, `"transliterate"` o `"html-entities"`). Se ejecuta fuera
/// del hilo principal para no bloquear la interfaz.
///
/// Uso desde TypeScript:
///
/// ```ts
/// import { invoke } from "@tauri-apps/api/core";
/// const saved = await invoke<{ bytesWritten: number; losses: LossReport | null }>(
///   "save_file",
///   { path, text, encoding: "utf-8", lineEnding: "crlf", strategy: null },
/// );
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
