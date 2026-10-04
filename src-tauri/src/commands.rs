//! Comandos Tauri: el «puente» IPC entre el frontend y el [`crate::model`].
//!
//! Cada comando debe ser una función muy fina que solo traduce
//! argumentos/resultados. La lógica va en el modelo, donde se prueba.
//! Desde TypeScript se invocan con `invoke("nombre_comando", { ...args })`.

use crate::model::app_info::AppInfo;

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
