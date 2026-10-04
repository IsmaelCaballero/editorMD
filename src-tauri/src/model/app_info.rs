//! **M00 `AppInfo`** (Modelo · Rust) — identidad y versión de la aplicación.
//!
//! La versión sale de `Cargo.toml` en tiempo de compilación, que junto con
//! `package.json` y `tauri.conf.json` es una de las tres fuentes que deben
//! coincidir (ver `PLAN.md` §10.1).

use serde::Serialize;

/// Nombre y versión de la aplicación en ejecución.
///
/// Se serializa a JSON para enviarlo al frontend con camelCase
/// (`{ "name": ..., "version": ... }`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    /// Nombre del producto, p. ej. `"editorMD"`.
    pub name: String,
    /// Versión con formato Semantic Versioning 2.0.0, p. ej. `"0.0.1"`.
    pub version: String,
}

impl AppInfo {
    /// Nombre del producto que se muestra al usuario.
    pub const PRODUCT_NAME: &'static str = "editorMD";

    /// Devuelve la información de la compilación actual.
    ///
    /// # Examples
    ///
    /// ```
    /// use editormd_lib::model::app_info::AppInfo;
    ///
    /// let info = AppInfo::current();
    /// assert_eq!(info.name, "editorMD");
    /// assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
    /// ```
    pub fn current() -> Self {
        Self {
            name: Self::PRODUCT_NAME.to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_uses_product_name() {
        assert_eq!(AppInfo::current().name, "editorMD");
    }

    #[test]
    fn version_has_semver_core_format() {
        // MAJOR.MINOR.PATCH: tres enteros no negativos sin ceros a la izquierda.
        let version = AppInfo::current().version;
        let core = version.split(['-', '+']).next().unwrap();
        let parts: Vec<&str> = core.split('.').collect();
        assert_eq!(parts.len(), 3, "versión no SemVer: {version}");
        for p in parts {
            assert!(p.parse::<u64>().is_ok(), "parte no numérica: {p}");
            assert!(p == "0" || !p.starts_with('0'), "cero a la izquierda: {p}");
        }
    }

    #[test]
    fn serializes_as_camel_case_json() {
        let json = serde_json::to_string(&AppInfo {
            name: "x".into(),
            version: "1.2.3".into(),
        })
        .unwrap();
        assert_eq!(json, r#"{"name":"x","version":"1.2.3"}"#);
    }
}
