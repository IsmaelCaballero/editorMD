//! Script de compilación de Cargo: genera el contexto de Tauri (configuración,
//! iconos, permisos) que usa `tauri::generate_context!()` en `lib.rs`.

fn main() {
    tauri_build::build();
}
