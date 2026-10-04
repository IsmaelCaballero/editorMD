//! Capa **Modelo** del backend (servicios de dominio).
//!
//! Los módulos de aquí no dependen de Tauri ni de la interfaz, así que se
//! prueban como Rust normal. Cada módulo corresponde a un componente del
//! registro de `PLAN.md` §4.3 y lleva su propia versión SemVer.

pub mod app_info;
pub mod text_codec;
