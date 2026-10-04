//! **M08 `FileService`** (Modelo · Rust) — acceso al disco de los documentos
//! de texto (`PLAN.md` §2.5 y §4).
//!
//! - **Bytes** ([`read_file`], [`write_atomic`]): lectura completa con el
//!   indicador de solo lectura, y escritura **atómica** (fichero temporal en la
//!   misma carpeta, `sync_all` y renombrado), que nunca deja el destino a medias.
//! - **Documentos** ([`open_text`], [`save_text`]): combinan lo anterior con
//!   **M09 [`TextCodec`](crate::model::text_codec)** para abrir y guardar texto
//!   respetando su codificación y su fin de línea.
//!
//! Los errores ([`FileError`]) se serializan para el frontend con un campo
//! `kind` en *kebab-case* (`{"kind":"not-found","path":"…"}`).

use std::fmt;
use std::path::Path;

use serde::Serialize;

use crate::model::text_codec::{Encoding, LineEnding, LossReport, LossStrategy, TextFile};

/// Error al abrir o guardar un fichero. Serializable para el frontend.
///
/// En todos los casos, `path` es la ruta recibida (`path.display().to_string()`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum FileError {
    /// El fichero (o, al escribir, su carpeta) no existe.
    NotFound {
        /// Ruta afectada.
        path: String,
    },
    /// El sistema operativo deniega el acceso.
    PermissionDenied {
        /// Ruta afectada.
        path: String,
    },
    /// El fichero existe y está marcado como de solo lectura (no se intenta escribir).
    ReadOnly {
        /// Ruta afectada.
        path: String,
    },
    /// La ruta es una carpeta.
    IsDirectory {
        /// Ruta afectada.
        path: String,
    },
    /// Cualquier otro error de E/S.
    Io {
        /// Ruta afectada.
        path: String,
        /// El `Display` del [`std::io::Error`] original.
        message: String,
    },
    /// El contenido no es válido en la codificación usada.
    Decode {
        /// Ruta afectada.
        path: String,
        /// Codificación con la que se intentó leer.
        encoding: Encoding,
        /// Posición en bytes de la secuencia no válida (`None` si la longitud
        /// de un UTF-16 es impar).
        offset: Option<usize>,
        /// El `Display` del [`DecodeError`](crate::model::text_codec::DecodeError).
        message: String,
    },
    /// Al guardar sin estrategia, hay caracteres que no caben en la codificación.
    Unmappable {
        /// Ruta afectada.
        path: String,
        /// Informe de pérdidas del texto recibido.
        report: LossReport,
    },
}

impl fmt::Display for FileError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl std::error::Error for FileError {}

/// Bytes de un fichero leído del disco.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileBytes {
    /// Contenido completo del fichero.
    pub bytes: Vec<u8>,
    /// `true` si el fichero está marcado como de solo lectura.
    pub read_only: bool,
}

/// Documento de texto abierto, listo para el editor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenedFile {
    /// Texto, codificación y fin de línea del documento.
    pub file: TextFile,
    /// `true` si el fichero está marcado como de solo lectura.
    pub read_only: bool,
}

/// Resultado de guardar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedFile {
    /// Bytes escritos en el disco (BOM incluido).
    pub bytes_written: usize,
    /// Informe de pérdidas si se guardó con una [`LossStrategy`]; `None` si
    /// el texto cabía entero en la codificación.
    pub losses: Option<LossReport>,
}

/// Lee el fichero completo.
///
/// # Errors
///
/// [`FileError::IsDirectory`], [`FileError::NotFound`],
/// [`FileError::PermissionDenied`] o [`FileError::Io`].
pub fn read_file(_path: &Path) -> Result<FileBytes, FileError> {
    todo!()
}

/// Escribe `bytes` en `path` de forma atómica.
///
/// # Errors
///
/// [`FileError::IsDirectory`], [`FileError::ReadOnly`], [`FileError::NotFound`],
/// [`FileError::PermissionDenied`] o [`FileError::Io`].
pub fn write_atomic(_path: &Path, _bytes: &[u8]) -> Result<(), FileError> {
    todo!()
}

/// [`write_atomic`] con el renombrado inyectable (las pruebas simulan un fallo).
fn write_atomic_with<F>(_path: &Path, _bytes: &[u8], _rename: F) -> Result<(), FileError>
where
    F: FnOnce(&Path, &Path) -> std::io::Result<()>,
{
    todo!()
}

/// Traduce un error de E/S a [`FileError`] según su `ErrorKind`.
fn map_io(_path: &Path, _error: &std::io::Error) -> FileError {
    todo!()
}

/// Carpeta donde se crea el temporal: la del destino, o `.` si la ruta no tiene.
fn parent_dir(_path: &Path) -> &Path {
    todo!()
}

/// Abre un documento de texto.
///
/// # Errors
///
/// Los de [`read_file`] y [`FileError::Decode`].
pub fn open_text(_path: &Path, _encoding: Option<Encoding>) -> Result<OpenedFile, FileError> {
    todo!()
}

/// Guarda un documento de texto.
///
/// # Errors
///
/// [`FileError::Unmappable`] y los de [`write_atomic`].
pub fn save_text(
    _path: &Path,
    _text: &str,
    _encoding: Encoding,
    _line_ending: LineEnding,
    _strategy: Option<LossStrategy>,
) -> Result<SavedFile, FileError> {
    todo!()
}

#[cfg(test)]
mod tests;
