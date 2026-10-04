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

use std::ffi::{OsStr, OsString};
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;

use crate::model::text_codec::{
    DecodeError, Encoding, LineEnding, LossReport, LossStrategy, TextFile, loss_report, read_text,
    read_text_as, write_text, write_text_lossy,
};

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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileError::NotFound { path } => {
                write!(f, "no existe el fichero (o su carpeta): {path}")
            }
            FileError::PermissionDenied { path } => write!(f, "acceso denegado a {path}"),
            FileError::ReadOnly { path } => write!(f, "el fichero {path} es de solo lectura"),
            FileError::IsDirectory { path } => write!(f, "{path} es una carpeta, no un fichero"),
            FileError::Io { path, message } => write!(f, "error de E/S en {path}: {message}"),
            FileError::Decode {
                path,
                encoding,
                message,
                ..
            } => write!(
                f,
                "no se puede leer {path} como {}: {message}",
                encoding.id()
            ),
            FileError::Unmappable { path, report } => write!(
                f,
                "{} carácter(es) de {path} no caben en la codificación {}",
                report.total,
                report.encoding.id()
            ),
        }
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
/// Comprueba antes de abrirlo si la ruta es una carpeta (en Windows abrir una
/// carpeta da «acceso denegado», que confundiría al usuario).
///
/// # Errors
///
/// [`FileError::IsDirectory`] si la ruta es una carpeta; [`FileError::NotFound`],
/// [`FileError::PermissionDenied`] o [`FileError::Io`] según el error de E/S.
pub fn read_file(path: &Path) -> Result<FileBytes, FileError> {
    let metadata = fs::metadata(path).map_err(|e| map_io(path, &e))?;
    if metadata.is_dir() {
        return Err(is_directory(path));
    }
    let bytes = fs::read(path).map_err(|e| map_io(path, &e))?;
    Ok(FileBytes {
        bytes,
        read_only: metadata.permissions().readonly(),
    })
}

/// Escribe `bytes` en `path` de forma **atómica**: o queda el contenido nuevo
/// completo, o el anterior intacto.
///
/// Los bytes se escriben en un fichero temporal de la misma carpeta, se vuelcan
/// al disco (`sync_all`) y el temporal se renombra sobre el destino. Si algo
/// falla, el temporal se borra. No se crean carpetas. En Unix, si el destino ya
/// existía, se conservan sus permisos.
///
/// # Errors
///
/// - [`FileError::IsDirectory`] si la ruta es una carpeta y
///   [`FileError::ReadOnly`] si el fichero es de solo lectura (en ambos casos no
///   se toca el disco).
/// - [`FileError::NotFound`] si la carpeta del destino no existe;
///   [`FileError::PermissionDenied`] o [`FileError::Io`] para el resto de
///   errores de E/S.
///
/// # Examples
///
/// ```
/// use editormd_lib::model::file_service::{read_file, write_atomic};
///
/// let dir = tempfile::tempdir()?;
/// let path = dir.path().join("nota.md");
/// write_atomic(&path, b"# Hola\n")?;
/// write_atomic(&path, b"# Adios\n")?;
/// assert_eq!(read_file(&path)?.bytes, b"# Adios\n");
/// assert_eq!(std::fs::read_dir(dir.path())?.count(), 1); // sin temporales
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), FileError> {
    write_atomic_with(path, bytes, |from, to| fs::rename(from, to))
}

/// [`write_atomic`] con el renombrado inyectable (las pruebas simulan un fallo).
fn write_atomic_with<F>(path: &Path, bytes: &[u8], rename: F) -> Result<(), FileError>
where
    F: FnOnce(&Path, &Path) -> io::Result<()>,
{
    let previous = match fs::metadata(path) {
        Ok(metadata) if metadata.is_dir() => return Err(is_directory(path)),
        Ok(metadata) if metadata.permissions().readonly() => {
            return Err(FileError::ReadOnly {
                path: path.display().to_string(),
            });
        }
        Ok(metadata) => Some(metadata.permissions()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => None,
        Err(e) => return Err(map_io(path, &e)),
    };

    let (tmp_path, mut tmp) = create_temp(path).map_err(|e| map_io(path, &e))?;
    let result = tmp
        .write_all(bytes)
        .and_then(|()| keep_permissions(&tmp, previous))
        .and_then(|()| tmp.sync_all())
        .and_then(|()| {
            drop(tmp); // en Windows no se puede renombrar un fichero abierto
            rename(&tmp_path, path)
        });
    if let Err(e) = result {
        // Mejor esfuerzo: si no se puede borrar, el error relevante es el original.
        let _ = fs::remove_file(&tmp_path);
        return Err(map_io(path, &e));
    }
    sync_dir(parent_dir(path));
    Ok(())
}

/// Crea (sin sobrescribir nada) un fichero temporal junto al destino, con un
/// nombre oculto del tipo `.nota.md.<pid>.<n>.tmp`.
fn create_temp(path: &Path) -> io::Result<(PathBuf, File)> {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = parent_dir(path);
    let name = path.file_name().unwrap_or_else(|| OsStr::new("editormd"));
    let pid = std::process::id();
    loop {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let mut tmp_name = OsString::from(".");
        tmp_name.push(name);
        tmp_name.push(format!(".{pid}.{n}.tmp"));
        let tmp_path = dir.join(tmp_name);
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp_path)
        {
            Ok(file) => return Ok((tmp_path, file)),
            // Resto de otra ejecución con el mismo nombre: se prueba el siguiente.
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e),
        }
    }
}

/// En Unix, copia al temporal los permisos (modo) del destino que sustituye.
#[cfg(unix)]
fn keep_permissions(tmp: &File, previous: Option<fs::Permissions>) -> io::Result<()> {
    match previous {
        Some(permissions) => tmp.set_permissions(permissions),
        None => Ok(()),
    }
}

/// Fuera de Unix solo existe el atributo de solo lectura, que el destino no
/// tiene (se ha comprobado antes): no hay nada que copiar.
#[cfg(not(unix))]
#[allow(clippy::unnecessary_wraps, clippy::needless_pass_by_value)]
fn keep_permissions(_tmp: &File, _previous: Option<fs::Permissions>) -> io::Result<()> {
    Ok(())
}

/// En Unix, vuelca la carpeta para que el renombrado sobreviva a un corte de
/// luz (mejor esfuerzo: el fichero ya está escrito y renombrado).
#[cfg(unix)]
fn sync_dir(dir: &Path) {
    if let Ok(handle) = File::open(dir) {
        let _ = handle.sync_all();
    }
}

/// Fuera de Unix no se pueden abrir carpetas como ficheros: no se hace nada.
#[cfg(not(unix))]
fn sync_dir(_dir: &Path) {}

/// Traduce un error de E/S a [`FileError`] según su `ErrorKind`.
fn map_io(path: &Path, error: &io::Error) -> FileError {
    let path = path.display().to_string();
    match error.kind() {
        io::ErrorKind::NotFound => FileError::NotFound { path },
        io::ErrorKind::PermissionDenied => FileError::PermissionDenied { path },
        _ => FileError::Io {
            path,
            message: error.to_string(),
        },
    }
}

/// [`FileError::IsDirectory`] para `path`.
fn is_directory(path: &Path) -> FileError {
    FileError::IsDirectory {
        path: path.display().to_string(),
    }
}

/// Carpeta donde se crea el temporal: la del destino, o `.` si la ruta no tiene.
fn parent_dir(path: &Path) -> &Path {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    }
}

/// Abre un documento de texto: lee el fichero y lo decodifica con **M09**.
///
/// Con `encoding = None` la codificación se detecta ([`read_text`]); con
/// `Some(e)` se usa `e` («Reabrir con codificación…», [`read_text_as`]).
///
/// # Errors
///
/// Los de [`read_file`] y [`FileError::Decode`] si el contenido no es válido en
/// la codificación usada.
///
/// # Examples
///
/// ```
/// use editormd_lib::model::file_service::open_text;
/// use editormd_lib::model::text_codec::{Encoding, LineEnding};
///
/// let dir = tempfile::tempdir()?;
/// let path = dir.path().join("nota.md");
/// std::fs::write(&path, b"\xef\xbb\xbfuno\r\ndos\r\n")?;
///
/// let opened = open_text(&path, None)?;
/// assert_eq!(opened.file.text, "uno\ndos\n");
/// assert_eq!(opened.file.encoding, Encoding::Utf8Bom);
/// assert_eq!(opened.file.line_ending, LineEnding::Crlf);
/// assert!(!opened.read_only);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn open_text(path: &Path, encoding: Option<Encoding>) -> Result<OpenedFile, FileError> {
    let FileBytes { bytes, read_only } = read_file(path)?;
    let decoded = match encoding {
        None => read_text(&bytes),
        Some(encoding) => read_text_as(&bytes, encoding),
    };
    let file = decoded.map_err(|e| decode_error(path, &e))?;
    Ok(OpenedFile { file, read_only })
}

/// Traduce un [`DecodeError`] a [`FileError::Decode`].
fn decode_error(path: &Path, error: &DecodeError) -> FileError {
    let (encoding, offset) = match *error {
        DecodeError::InvalidSequence { encoding, offset } => (encoding, Some(offset)),
        DecodeError::OddLength { encoding } => (encoding, None),
    };
    FileError::Decode {
        path: path.display().to_string(),
        encoding,
        offset,
        message: error.to_string(),
    }
}

/// Guarda un documento de texto con la codificación y el fin de línea dados.
///
/// Primero se preparan los bytes **sin tocar el disco**: si algún carácter no
/// cabe en `encoding` y no hay `strategy`, se devuelve
/// [`FileError::Unmappable`] y no se crea ni modifica nada; con `strategy`, se
/// aplica ([`write_text_lossy`]) y el informe vuelve en [`SavedFile::losses`].
/// Después se escribe con [`write_atomic`].
///
/// Para un [`OpenedFile`] sin editar, guardarlo con su misma codificación y fin
/// de línea deja el fichero idéntico byte a byte (salvo finales de línea
/// mezclados, que quedan normalizados).
///
/// # Errors
///
/// [`FileError::Unmappable`] y los de [`write_atomic`].
pub fn save_text(
    path: &Path,
    text: &str,
    encoding: Encoding,
    line_ending: LineEnding,
    strategy: Option<LossStrategy>,
) -> Result<SavedFile, FileError> {
    let report = loss_report(text, encoding);
    let (bytes, losses) = if report.is_lossless() {
        match write_text(text, encoding, line_ending) {
            Ok(bytes) => (bytes, None),
            // No debería ocurrir (el informe dice que todo cabe), pero sin pánico.
            Err(_) => return Err(unmappable(path, report)),
        }
    } else if let Some(strategy) = strategy {
        let bytes = write_text_lossy(text, encoding, line_ending, strategy);
        (bytes, Some(report))
    } else {
        return Err(unmappable(path, report));
    };
    write_atomic(path, &bytes)?;
    Ok(SavedFile {
        bytes_written: bytes.len(),
        losses,
    })
}

/// [`FileError::Unmappable`] para `path` con su informe.
fn unmappable(path: &Path, report: LossReport) -> FileError {
    FileError::Unmappable {
        path: path.display().to_string(),
        report,
    }
}

#[cfg(test)]
mod tests;
