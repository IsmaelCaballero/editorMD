//! **M08 `FileService`** (Modelo · Rust) — acceso al disco de los documentos
//! de texto (`PLAN.md` §2.5 y §4).
//!
//! - **Bytes** ([`read_file`], [`write_atomic`]): lectura completa con el
//!   indicador de solo lectura y escritura **atómica** (fichero temporal en la
//!   misma carpeta, `sync_all` y renombrado), de modo que un fallo nunca deja
//!   el destino a medias.
//! - **Documentos** ([`open_text`], [`save_text`]): combinan lo anterior con
//!   **M09 `TextCodec`** para abrir y guardar texto conservando la codificación
//!   y el fin de línea (ida y vuelta byte a byte).
//! - **Errores** ([`FileError`]): serializables para que el frontend muestre el
//!   mensaje adecuado a cada caso.

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
/// Se serializa con una etiqueta `kind` en *kebab-case* y los campos en
/// *camelCase*, p. ej. `{"kind":"not-found","path":"notas.md"}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum FileError {
    /// El fichero (o, al escribir, su carpeta) no existe.
    NotFound {
        /// Ruta recibida.
        path: String,
    },
    /// El sistema operativo deniega el acceso.
    PermissionDenied {
        /// Ruta recibida.
        path: String,
    },
    /// El fichero existe y está marcado como de solo lectura (no se intenta escribir).
    ReadOnly {
        /// Ruta recibida.
        path: String,
    },
    /// La ruta es una carpeta.
    IsDirectory {
        /// Ruta recibida.
        path: String,
    },
    /// Cualquier otro error de E/S.
    Io {
        /// Ruta recibida.
        path: String,
        /// El `Display` del [`std::io::Error`].
        message: String,
    },
    /// El contenido no es válido en la codificación usada.
    Decode {
        /// Ruta recibida.
        path: String,
        /// Codificación con la que se intentó decodificar.
        encoding: Encoding,
        /// Posición en bytes de la secuencia no válida (`None` si UTF-16 tiene
        /// longitud impar).
        offset: Option<usize>,
        /// El `Display` del [`DecodeError`](crate::model::text_codec::DecodeError).
        message: String,
    },
    /// Al guardar sin estrategia, hay caracteres que no caben en la codificación.
    Unmappable {
        /// Ruta recibida.
        path: String,
        /// Informe de pérdidas del texto recibido.
        report: LossReport,
    },
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileError::NotFound { path } => {
                write!(f, "no existe el fichero o la carpeta: «{path}»")
            }
            FileError::PermissionDenied { path } => {
                write!(f, "acceso denegado a «{path}»")
            }
            FileError::ReadOnly { path } => {
                write!(f, "el fichero «{path}» es de solo lectura")
            }
            FileError::IsDirectory { path } => {
                write!(f, "«{path}» es una carpeta, no un fichero")
            }
            FileError::Io { path, message } => {
                write!(f, "error de entrada/salida en «{path}»: {message}")
            }
            FileError::Decode { path, message, .. } => {
                write!(f, "no se puede leer «{path}»: {message}")
            }
            FileError::Unmappable { path, report } => write!(
                f,
                "no se puede guardar «{path}» en {}: {} carácter(es) no caben en la codificación",
                report.encoding.id(),
                report.total
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
    /// Informe de pérdidas si se guardó con una estrategia de conversión y
    /// había caracteres que no cabían; `None` si el guardado no tuvo pérdidas.
    pub losses: Option<LossReport>,
}

/// Lee un fichero completo e indica si está marcado como de solo lectura.
///
/// Comprueba antes de abrir si la ruta es una carpeta (en Windows, abrir una
/// carpeta da «acceso denegado» en lugar de un error claro).
///
/// # Errors
///
/// [`FileError::IsDirectory`] si la ruta es una carpeta; [`FileError::NotFound`],
/// [`FileError::PermissionDenied`] o [`FileError::Io`] según el error de E/S.
pub fn read_file(path: &Path) -> Result<FileBytes, FileError> {
    let metadata = fs::metadata(path).map_err(|e| io_error(path, &e))?;
    if metadata.is_dir() {
        return Err(is_directory(path));
    }
    let bytes = fs::read(path).map_err(|e| io_error(path, &e))?;
    Ok(FileBytes {
        bytes,
        read_only: metadata.permissions().readonly(),
    })
}

/// Escribe `bytes` en `path` de forma **atómica**: los escribe en un fichero
/// temporal de la misma carpeta, lo vuelca al disco (`sync_all`) y lo renombra
/// sobre el destino.
///
/// Si algo falla, el destino queda intacto (con su contenido anterior o sin
/// existir) y el temporal se borra. No se crean carpetas. En Unix, si el
/// destino ya existía, se conservan sus permisos.
///
/// # Errors
///
/// [`FileError::IsDirectory`] si la ruta es una carpeta, [`FileError::ReadOnly`]
/// si el fichero es de solo lectura (en ambos casos no se toca nada),
/// [`FileError::NotFound`] si no existe la carpeta, y
/// [`FileError::PermissionDenied`] o [`FileError::Io`] según el error de E/S.
///
/// # Examples
///
/// ```
/// use editormd_lib::model::file_service::write_atomic;
///
/// let dir = tempfile::tempdir().unwrap();
/// let path = dir.path().join("notas.md");
/// write_atomic(&path, b"# Hola\n").unwrap();
/// assert_eq!(std::fs::read(&path).unwrap(), b"# Hola\n");
/// ```
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), FileError> {
    write_atomic_with(path, bytes, |tmp, target| fs::rename(tmp, target))
}

/// Abre un documento de texto: lo lee ([`read_file`]) y lo decodifica con
/// detección automática ([`read_text`]) si `encoding` es `None`, o con la
/// codificación indicada ([`read_text_as`]) si es `Some`.
///
/// # Errors
///
/// Los de [`read_file`] y [`FileError::Decode`] si el contenido no es válido
/// en la codificación usada.
///
/// # Examples
///
/// ```
/// use editormd_lib::model::file_service::open_text;
/// use editormd_lib::model::text_codec::{Encoding, LineEnding};
///
/// let dir = tempfile::tempdir().unwrap();
/// let path = dir.path().join("notas.md");
/// std::fs::write(&path, b"uno\r\ndos\r\n").unwrap();
/// let opened = open_text(&path, None).unwrap();
/// assert_eq!(opened.file.text, "uno\ndos\n");
/// assert_eq!(opened.file.encoding, Encoding::Utf8);
/// assert_eq!(opened.file.line_ending, LineEnding::Crlf);
/// assert!(!opened.read_only);
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

/// Guarda un documento de texto con la codificación y el fin de línea dados.
///
/// Primero prepara los bytes sin tocar el disco: si algún carácter no cabe en
/// `encoding`, sin estrategia se devuelve [`FileError::Unmappable`] y con
/// estrategia se convierte ([`write_text_lossy`]) y se devuelve el informe en
/// [`SavedFile::losses`]. Después escribe con [`write_atomic`].
///
/// Para un [`OpenedFile`] sin editar, guardarlo con su misma codificación y
/// fin de línea deja el fichero idéntico byte a byte (salvo finales de línea
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
        // Sin pérdidas, `write_text` no puede fallar; por si acaso, se informa igual.
        let bytes = write_text(text, encoding, line_ending).map_err(|_| FileError::Unmappable {
            path: shown(path),
            report: report.clone(),
        })?;
        (bytes, None)
    } else {
        let Some(strategy) = strategy else {
            return Err(FileError::Unmappable {
                path: shown(path),
                report,
            });
        };
        (
            write_text_lossy(text, encoding, line_ending, strategy),
            Some(report),
        )
    };
    write_atomic(path, &bytes)?;
    Ok(SavedFile {
        bytes_written: bytes.len(),
        losses,
    })
}

/// Ruta tal como se muestra en los errores.
fn shown(path: &Path) -> String {
    path.display().to_string()
}

/// [`FileError::IsDirectory`] de `path`.
fn is_directory(path: &Path) -> FileError {
    FileError::IsDirectory { path: shown(path) }
}

/// Convierte un error de E/S en un [`FileError`] de `path`.
fn io_error(path: &Path, err: &io::Error) -> FileError {
    match err.kind() {
        io::ErrorKind::NotFound => FileError::NotFound { path: shown(path) },
        io::ErrorKind::PermissionDenied => FileError::PermissionDenied { path: shown(path) },
        _ => FileError::Io {
            path: shown(path),
            message: err.to_string(),
        },
    }
}

/// Convierte un [`DecodeError`] en [`FileError::Decode`].
fn decode_error(path: &Path, err: &DecodeError) -> FileError {
    let (encoding, offset) = match *err {
        DecodeError::InvalidSequence { encoding, offset } => (encoding, Some(offset)),
        DecodeError::OddLength { encoding } => (encoding, None),
    };
    FileError::Decode {
        path: shown(path),
        encoding,
        offset,
        message: err.to_string(),
    }
}

/// Contador para que los nombres de los temporales no se repitan dentro del proceso.
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Intentos de crear un temporal con nombre libre antes de rendirse.
const TEMP_ATTEMPTS: u32 = 64;

/// [`write_atomic`] con el paso final (`persist(temporal, destino)`, el
/// renombrado) inyectable para poder probar sus fallos.
fn write_atomic_with(
    path: &Path,
    bytes: &[u8],
    persist: impl FnOnce(&Path, &Path) -> io::Result<()>,
) -> Result<(), FileError> {
    let existing = match fs::metadata(path) {
        Ok(metadata) if metadata.is_dir() => return Err(is_directory(path)),
        Ok(metadata) if metadata.permissions().readonly() => {
            return Err(FileError::ReadOnly { path: shown(path) });
        }
        Ok(metadata) => Some(metadata.permissions()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => None,
        Err(e) => return Err(io_error(path, &e)),
    };
    let (tmp, mut file) = create_temp(path).map_err(|e| io_error(path, &e))?;
    let result = file
        .write_all(bytes)
        .and_then(|()| file.sync_all())
        .and_then(|()| match existing {
            // Conserva los permisos (el modo, en Unix) del destino anterior.
            Some(permissions) => fs::set_permissions(&tmp, permissions),
            None => Ok(()),
        })
        .and_then(|()| {
            // Se cierra antes de renombrar (en Windows no se puede con el fichero abierto).
            drop(file);
            persist(&tmp, path)
        });
    if let Err(e) = result {
        // El destino no se ha tocado; solo queda quitar el temporal.
        let _ = fs::remove_file(&tmp);
        return Err(io_error(path, &e));
    }
    Ok(())
}

/// Crea (sin sobrescribir nada) un fichero temporal junto a `path`.
fn create_temp(path: &Path) -> io::Result<(PathBuf, File)> {
    let mut last = io::Error::from(io::ErrorKind::AlreadyExists);
    for _ in 0..TEMP_ATTEMPTS {
        let tmp = temp_path(path, TEMP_COUNTER.fetch_add(1, Ordering::Relaxed));
        match OpenOptions::new().write(true).create_new(true).open(&tmp) {
            Ok(file) => return Ok((tmp, file)),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => last = e,
            Err(e) => return Err(e),
        }
    }
    Err(last)
}

/// Ruta del fichero temporal número `n` para escribir `path`: oculto y en la
/// misma carpeta (`.nombre.<pid>.<n>.tmp`), para que el renombrado sea atómico.
fn temp_path(path: &Path, n: u64) -> PathBuf {
    let name = path
        .file_name()
        .map_or_else(|| "editormd".into(), |name| name.to_string_lossy());
    let tmp_name = format!(".{name}.{}.{n}.tmp", std::process::id());
    match path.parent() {
        Some(parent) => parent.join(tmp_name),
        None => PathBuf::from(tmp_name),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io;
    use std::path::PathBuf;

    use super::*;
    use crate::model::text_codec::{DetectionMethod, loss_report};

    /// Marca un fichero como de solo lectura y lo restaura al soltarse.
    struct ReadOnlyGuard(PathBuf);

    impl ReadOnlyGuard {
        fn new(path: &Path) -> Self {
            set_readonly(path, true);
            ReadOnlyGuard(path.to_path_buf())
        }
    }

    impl Drop for ReadOnlyGuard {
        fn drop(&mut self) {
            set_readonly(&self.0, false);
        }
    }

    #[allow(clippy::permissions_set_readonly_false)]
    fn set_readonly(path: &Path, readonly: bool) {
        let mut perms = fs::metadata(path).unwrap().permissions();
        perms.set_readonly(readonly);
        fs::set_permissions(path, perms).unwrap();
    }

    /// Nombres de las entradas de una carpeta, ordenados.
    fn entries(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    fn shown(path: &Path) -> String {
        path.display().to_string()
    }

    // ---- read_file ----

    #[test]
    fn read_file_reads_all_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.md");
        let content: Vec<u8> = (0..=255).collect();
        fs::write(&path, &content).unwrap();
        let r = read_file(&path).unwrap();
        assert_eq!(
            r,
            FileBytes {
                bytes: content,
                read_only: false
            }
        );
    }

    #[test]
    fn read_file_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vacio.txt");
        fs::write(&path, b"").unwrap();
        assert_eq!(read_file(&path).unwrap().bytes, b"");
    }

    #[test]
    fn read_file_reports_read_only() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ro.md");
        fs::write(&path, b"x").unwrap();
        let _guard = ReadOnlyGuard::new(&path);
        let r = read_file(&path).unwrap();
        assert!(r.read_only);
        assert_eq!(r.bytes, b"x");
    }

    #[test]
    fn read_file_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("no-existe.md");
        assert_eq!(
            read_file(&path),
            Err(FileError::NotFound { path: shown(&path) })
        );
    }

    #[test]
    fn read_file_missing_folder() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("no").join("existe.md");
        assert_eq!(
            read_file(&path),
            Err(FileError::NotFound { path: shown(&path) })
        );
    }

    #[test]
    fn read_file_directory() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            read_file(dir.path()),
            Err(FileError::IsDirectory {
                path: shown(dir.path())
            })
        );
    }

    #[test]
    fn io_error_mapping() {
        let p = Path::new("x/y.md");
        assert_eq!(
            io_error(p, &io::Error::from(io::ErrorKind::NotFound)),
            FileError::NotFound { path: shown(p) }
        );
        assert_eq!(
            io_error(p, &io::Error::from(io::ErrorKind::PermissionDenied)),
            FileError::PermissionDenied { path: shown(p) }
        );
        let other = io::Error::other("disco lleno");
        assert_eq!(
            io_error(p, &other),
            FileError::Io {
                path: shown(p),
                message: other.to_string()
            }
        );
    }

    #[cfg(unix)]
    #[test]
    fn read_file_permission_denied() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("secreto.md");
        fs::write(&path, b"x").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
        let result = read_file(&path);
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        // Como root no hay denegación posible.
        if result.is_err() {
            assert_eq!(
                result,
                Err(FileError::PermissionDenied { path: shown(&path) })
            );
        }
    }

    // ---- write_atomic ----

    #[test]
    fn write_atomic_creates_new_file_without_leftovers() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("otro.txt"), b"o").unwrap();
        let path = dir.path().join("nuevo.md");
        write_atomic(&path, b"hola").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"hola");
        assert_eq!(entries(dir.path()), ["nuevo.md", "otro.txt"]);
    }

    #[test]
    fn write_atomic_replaces_existing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.md");
        fs::write(&path, b"contenido anterior y largo").unwrap();
        write_atomic(&path, b"nuevo").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"nuevo");
        assert_eq!(entries(dir.path()), ["doc.md"]);
    }

    #[test]
    fn write_atomic_empty_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.md");
        fs::write(&path, b"algo").unwrap();
        write_atomic(&path, b"").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"");
        assert_eq!(entries(dir.path()), ["doc.md"]);
    }

    #[test]
    fn write_atomic_directory() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("carpeta");
        fs::create_dir(&sub).unwrap();
        assert_eq!(
            write_atomic(&sub, b"x"),
            Err(FileError::IsDirectory { path: shown(&sub) })
        );
        assert!(sub.is_dir());
        assert_eq!(entries(&sub), Vec::<String>::new());
        assert_eq!(entries(dir.path()), ["carpeta"]);
    }

    #[test]
    fn write_atomic_read_only() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ro.md");
        fs::write(&path, b"original").unwrap();
        let _guard = ReadOnlyGuard::new(&path);
        assert_eq!(
            write_atomic(&path, b"nuevo"),
            Err(FileError::ReadOnly { path: shown(&path) })
        );
        assert_eq!(fs::read(&path).unwrap(), b"original");
        assert_eq!(entries(dir.path()), ["ro.md"]);
    }

    #[test]
    fn write_atomic_missing_folder() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("no-existe").join("doc.md");
        assert_eq!(
            write_atomic(&path, b"x"),
            Err(FileError::NotFound { path: shown(&path) })
        );
        assert_eq!(entries(dir.path()), Vec::<String>::new());
    }

    #[test]
    fn write_atomic_failure_keeps_existing_target_and_cleans_up() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.md");
        fs::write(&path, b"original").unwrap();
        let err = write_atomic_with(&path, b"nuevo", |_, _| Err(io::Error::other("simulado")));
        assert_eq!(
            err,
            Err(FileError::Io {
                path: shown(&path),
                message: "simulado".into()
            })
        );
        assert_eq!(fs::read(&path).unwrap(), b"original");
        assert_eq!(entries(dir.path()), ["doc.md"]);
    }

    #[test]
    fn write_atomic_failure_creates_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.md");
        let err = write_atomic_with(&path, b"nuevo", |_, _| {
            Err(io::Error::from(io::ErrorKind::PermissionDenied))
        });
        assert_eq!(err, Err(FileError::PermissionDenied { path: shown(&path) }));
        assert_eq!(entries(dir.path()), Vec::<String>::new());
    }

    #[test]
    fn write_atomic_temp_is_in_same_folder() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.md");
        let mut seen = None;
        write_atomic_with(&path, b"abc", |tmp, target| {
            seen = Some((tmp.to_path_buf(), fs::read(tmp)?));
            assert_eq!(target, path.as_path());
            fs::rename(tmp, target)
        })
        .unwrap();
        let (tmp, content) = seen.unwrap();
        assert_eq!(tmp.parent(), Some(dir.path()));
        assert_ne!(tmp, path);
        assert_eq!(content, b"abc");
        assert_eq!(entries(dir.path()), ["doc.md"]);
    }

    #[test]
    fn temp_names_do_not_repeat() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.md");
        let a = temp_path(&path, 0);
        let b = temp_path(&path, 1);
        assert_ne!(a, b);
        assert_eq!(a.parent(), Some(dir.path()));
        // Una ruta relativa sin carpeta usa la carpeta actual.
        let rel = temp_path(Path::new("doc.md"), 0);
        assert_eq!(rel.parent(), Some(Path::new("")));
    }

    #[cfg(unix)]
    #[test]
    fn write_atomic_preserves_unix_mode() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("script.sh");
        fs::write(&path, b"#!/bin/sh\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o754)).unwrap();
        write_atomic(&path, b"#!/bin/sh\necho hola\n").unwrap();
        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o754);
    }

    // ---- open_text ----

    #[test]
    fn open_text_detects_encoding_and_line_ending() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.md");
        fs::write(&path, b"\xEF\xBB\xBFuno\r\ndos\r\n").unwrap();
        let f = open_text(&path, None).unwrap();
        assert_eq!(f.file.text, "uno\ndos\n");
        assert_eq!(f.file.encoding, Encoding::Utf8Bom);
        assert_eq!(f.file.line_ending, LineEnding::Crlf);
        assert_eq!(f.file.detection, Some(DetectionMethod::Bom));
        assert!(!f.read_only);
    }

    #[test]
    fn open_text_with_explicit_encoding() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.txt");
        fs::write(&path, b"caf\xe9\n").unwrap();
        let f = open_text(&path, Some(Encoding::Iso8859_1)).unwrap();
        assert_eq!(f.file.text, "café\n");
        assert_eq!(f.file.encoding, Encoding::Iso8859_1);
        assert_eq!(f.file.detection, None);
    }

    #[test]
    fn open_text_read_only() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ro.md");
        fs::write(&path, b"hola").unwrap();
        let _guard = ReadOnlyGuard::new(&path);
        let f = open_text(&path, None).unwrap();
        assert!(f.read_only);
        assert_eq!(f.file.text, "hola");
    }

    #[test]
    fn open_text_invalid_sequence() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.txt");
        fs::write(&path, b"abc\xe9").unwrap();
        let err = open_text(&path, Some(Encoding::Ascii)).unwrap_err();
        assert_eq!(
            err,
            FileError::Decode {
                path: shown(&path),
                encoding: Encoding::Ascii,
                offset: Some(3),
                message: "secuencia no válida para la codificación ascii en el byte 3".into(),
            }
        );
    }

    #[test]
    fn open_text_odd_length() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.txt");
        fs::write(&path, [0xFF, 0xFE, 0x61]).unwrap();
        let err = open_text(&path, None).unwrap_err();
        assert_eq!(
            err,
            FileError::Decode {
                path: shown(&path),
                encoding: Encoding::Utf16Le,
                offset: None,
                message: "longitud impar de bytes para la codificación utf-16le".into(),
            }
        );
    }

    #[test]
    fn open_text_propagates_io_errors() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("no.md");
        assert_eq!(
            open_text(&path, None),
            Err(FileError::NotFound { path: shown(&path) })
        );
        assert_eq!(
            open_text(dir.path(), Some(Encoding::Utf8)),
            Err(FileError::IsDirectory {
                path: shown(dir.path())
            })
        );
    }

    // ---- save_text ----

    #[test]
    fn save_text_lossless() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.md");
        let saved = save_text(&path, "a\nb\n", Encoding::Utf8Bom, LineEnding::Crlf, None).unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"\xEF\xBB\xBFa\r\nb\r\n");
        assert_eq!(
            saved,
            SavedFile {
                bytes_written: 9,
                losses: None
            }
        );
    }

    #[test]
    fn save_text_lossless_ignores_strategy() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.md");
        let saved = save_text(
            &path,
            "abc",
            Encoding::Ascii,
            LineEnding::Lf,
            Some(LossStrategy::Replace),
        )
        .unwrap();
        assert_eq!(saved.losses, None);
        assert_eq!(saved.bytes_written, 3);
    }

    #[test]
    fn save_text_unmappable_creates_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.txt");
        let text = "10 €\n";
        let err = save_text(&path, text, Encoding::Ascii, LineEnding::Lf, None).unwrap_err();
        assert_eq!(
            err,
            FileError::Unmappable {
                path: shown(&path),
                report: loss_report(text, Encoding::Ascii),
            }
        );
        assert_eq!(entries(dir.path()), Vec::<String>::new());
    }

    #[test]
    fn save_text_unmappable_keeps_existing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.txt");
        fs::write(&path, b"antes").unwrap();
        let r = save_text(&path, "ñ", Encoding::Ascii, LineEnding::Lf, None);
        assert!(matches!(r, Err(FileError::Unmappable { .. })));
        assert_eq!(fs::read(&path).unwrap(), b"antes");
        assert_eq!(entries(dir.path()), ["a.txt"]);
    }

    #[test]
    fn save_text_with_strategy() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.txt");
        let text = "10 €\nniño\n";
        let saved = save_text(
            &path,
            text,
            Encoding::Ascii,
            LineEnding::Crlf,
            Some(LossStrategy::Transliterate),
        )
        .unwrap();
        let expected = b"10 EUR\r\nnino\r\n";
        assert_eq!(fs::read(&path).unwrap(), expected);
        assert_eq!(saved.bytes_written, expected.len());
        assert_eq!(saved.losses, Some(loss_report(text, Encoding::Ascii)));
    }

    #[test]
    fn save_text_bytes_written_counts_bom_utf16() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.txt");
        let saved = save_text(&path, "hi", Encoding::Utf16Be, LineEnding::Lf, None).unwrap();
        assert_eq!(fs::read(&path).unwrap(), [0xFE, 0xFF, 0, b'h', 0, b'i']);
        assert_eq!(saved.bytes_written, 6);
    }

    #[test]
    fn save_text_read_only_and_directory() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ro.md");
        fs::write(&path, b"original").unwrap();
        let _guard = ReadOnlyGuard::new(&path);
        assert_eq!(
            save_text(&path, "x", Encoding::Utf8, LineEnding::Lf, None),
            Err(FileError::ReadOnly { path: shown(&path) })
        );
        assert_eq!(fs::read(&path).unwrap(), b"original");
        assert_eq!(
            save_text(dir.path(), "x", Encoding::Utf8, LineEnding::Lf, None),
            Err(FileError::IsDirectory {
                path: shown(dir.path())
            })
        );
    }

    #[test]
    fn roundtrip_is_byte_identical() {
        let cases: [&[u8]; 7] = [
            b"# T\xc3\xadtulo\n\ntexto\n",
            b"\xEF\xBB\xBFuno\r\ndos\r\n",
            &[0xFF, 0xFE, b'a', 0, b'\r', 0, b'b', 0],
            &[0xFE, 0xFF, 0, b'a', 0, b'\n'],
            b"El ni\xf1o comi\xf3 una pi\xf1a en la ma\xf1ana y despu\xe9s se fue.\r",
            b"",
            b"sin salto final",
        ];
        let dir = tempfile::tempdir().unwrap();
        for (i, original) in cases.into_iter().enumerate() {
            let path = dir.path().join(format!("f{i}.txt"));
            fs::write(&path, original).unwrap();
            let f = open_text(&path, None).unwrap().file;
            let saved = save_text(&path, &f.text, f.encoding, f.line_ending, None).unwrap();
            assert_eq!(fs::read(&path).unwrap(), original, "caso {i}");
            assert_eq!(saved.bytes_written, original.len());
        }
    }

    #[test]
    fn roundtrip_normalizes_mixed_line_endings() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mixto.md");
        fs::write(&path, b"a\r\nb\r\nc\n").unwrap();
        let f = open_text(&path, None).unwrap().file;
        assert!(f.mixed_line_endings);
        save_text(&path, &f.text, f.encoding, f.line_ending, None).unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"a\r\nb\r\nc\r\n");
    }

    // ---- FileError / serialización ----

    #[test]
    fn file_error_serialization() {
        let json = |e: &FileError| serde_json::to_string(e).unwrap();
        assert_eq!(
            json(&FileError::NotFound { path: "a".into() }),
            r#"{"kind":"not-found","path":"a"}"#
        );
        assert_eq!(
            json(&FileError::PermissionDenied { path: "a".into() }),
            r#"{"kind":"permission-denied","path":"a"}"#
        );
        assert_eq!(
            json(&FileError::ReadOnly { path: "a".into() }),
            r#"{"kind":"read-only","path":"a"}"#
        );
        assert_eq!(
            json(&FileError::IsDirectory { path: "a".into() }),
            r#"{"kind":"is-directory","path":"a"}"#
        );
        assert_eq!(
            json(&FileError::Io {
                path: "a".into(),
                message: "m".into()
            }),
            r#"{"kind":"io","path":"a","message":"m"}"#
        );
        assert_eq!(
            json(&FileError::Decode {
                path: "a".into(),
                encoding: Encoding::Ascii,
                offset: Some(3),
                message: "m".into()
            }),
            r#"{"kind":"decode","path":"a","encoding":"ascii","offset":3,"message":"m"}"#
        );
        assert_eq!(
            json(&FileError::Decode {
                path: "a".into(),
                encoding: Encoding::Utf16Le,
                offset: None,
                message: "m".into()
            }),
            r#"{"kind":"decode","path":"a","encoding":"utf-16le","offset":null,"message":"m"}"#
        );
        let report = loss_report("€", Encoding::Ascii);
        let expected = format!(
            r#"{{"kind":"unmappable","path":"a","report":{}}}"#,
            serde_json::to_string(&report).unwrap()
        );
        assert_eq!(
            json(&FileError::Unmappable {
                path: "a".into(),
                report
            }),
            expected
        );
    }

    #[test]
    fn file_error_display_includes_path() {
        let p = "carpeta/doc.md".to_string();
        let all = [
            FileError::NotFound { path: p.clone() },
            FileError::PermissionDenied { path: p.clone() },
            FileError::ReadOnly { path: p.clone() },
            FileError::IsDirectory { path: p.clone() },
            FileError::Io {
                path: p.clone(),
                message: "disco lleno".into(),
            },
            FileError::Decode {
                path: p.clone(),
                encoding: Encoding::Ascii,
                offset: Some(1),
                message: "secuencia no válida".into(),
            },
            FileError::Unmappable {
                path: p.clone(),
                report: loss_report("€€", Encoding::Ascii),
            },
        ];
        for e in &all {
            assert!(e.to_string().contains(&p), "{e}");
        }
        assert!(all[4].to_string().contains("disco lleno"));
        assert!(all[5].to_string().contains("secuencia no válida"));
        assert!(all[6].to_string().contains("ascii"));
        let _: &dyn std::error::Error = &all[0];
    }

    #[test]
    fn opened_and_saved_serialization() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.md");
        fs::write(&path, b"hola").unwrap();
        let opened = open_text(&path, None).unwrap();
        let expected = format!(
            r#"{{"file":{},"readOnly":false}}"#,
            serde_json::to_string(&opened.file).unwrap()
        );
        assert_eq!(serde_json::to_string(&opened).unwrap(), expected);
        let saved = SavedFile {
            bytes_written: 12,
            losses: None,
        };
        assert_eq!(
            serde_json::to_string(&saved).unwrap(),
            r#"{"bytesWritten":12,"losses":null}"#
        );
    }
}
