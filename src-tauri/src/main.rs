//! Ejecutable de escritorio de editorMD. Toda la lógica vive en la librería
//! `editormd_lib`, así puede probarse y reutilizarse.

// Evita que se abra una consola extra en Windows en modo release. NO QUITAR.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    editormd_lib::run();
}
