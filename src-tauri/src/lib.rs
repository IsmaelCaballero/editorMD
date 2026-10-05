//! # editorMD — backend Rust
//!
//! Núcleo nativo del editor Markdown **editorMD**. En la arquitectura MVC del
//! proyecto (ver `PLAN.md` §4), este crate implementa la parte **Modelo** que
//! necesita acceso al sistema: ficheros, codificaciones, exportación, Pandoc y
//! preferencias. El frontend (Vista + Controladores, en TypeScript) se
//! comunica con él mediante **comandos Tauri** (IPC).
//!
//! ## Organización
//! - [`model`]: servicios de dominio sin dependencias de Tauri (se prueban
//!   con `cargo test`).
//! - [`commands`]: capa fina que expone el modelo al frontend como comandos
//!   Tauri. No contiene lógica de negocio.
#![warn(missing_docs)]

pub mod commands;
pub mod model;

/// Punto de entrada de la aplicación: construye y ejecuta la ventana Tauri.
///
/// Registra los comandos IPC disponibles para el frontend y el plugin de
/// diálogos nativos (abrir/guardar, F2) y, en compilaciones de depuración,
/// activa el plugin de logging.
///
/// # Panics
///
/// Termina el proceso con un error si Tauri no puede inicializar la ventana
/// (por ejemplo, si falta el WebView del sistema).
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::open_file,
            commands::save_file
        ])
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}
