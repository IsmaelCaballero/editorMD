//! **M08 `FileService`** (Modelo · Rust) — acceso al disco para los documentos
//! de texto (`PLAN.md` §2.5 y §4).
//!
//! - **Bytes** ([`read_file`], [`write_atomic`]): lectura completa con el
//!   indicador de solo lectura y escritura atómica (fichero temporal en la
//!   misma carpeta + `sync_all` + renombrado), que nunca deja el destino a
//!   medias ni temporales sueltos.
//! - **Texto** ([`open_text`], [`save_text`]): combinación con
//!   **M09 `TextCodec`** para abrir y guardar documentos conservando la
//!   codificación y los finales de línea (ida y vuelta byte a byte).
//! - **Errores** ([`FileError`]): serializables para el frontend, con la ruta
//!   afectada en todos los casos.

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
/// Se serializa con la variante en el campo `kind` (kebab-case) y los campos
/// en camelCase, p. ej. `{"kind":"not-found","path":"…"}`.
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
    /// Cualquier otro error de E/S; `message` = el `Display` del `std::io::Error`.
    Io {
        /// Ruta afectada.
        path: String,
        /// Mensaje del error de E/S original.
        message: String,
    },
    /// El contenido no es válido en la codificación usada. `offset` = el del
    /// `DecodeError::InvalidSequence` (`None` en `OddLength`); `message` = el `Display`
    /// del `DecodeError`.
    Decode {
        /// Ruta afectada.
        path: String,
        /// Codificación con la que se intentó decodificar.
        encoding: Encoding,
        /// Posición en bytes de la secuencia no válida, si la hay.
        offset: Option<usize>,
        /// Mensaje del error de decodificación original.
        message: String,
    },
    /// Al guardar sin estrategia, hay caracteres que no caben en la codificación.
    /// `report` = `loss_report(text, encoding)` del texto recibido.
    Unmappable {
        /// Ruta afectada.
        path: String,
        /// Informe de los caracteres que no caben.
        report: LossReport,
    },
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileError::NotFound { path } => write!(f, "no existe «{path}»"),
            FileError::PermissionDenied { path } => {
                write!(f, "permiso denegado para acceder a «{path}»")
            }
            FileError::ReadOnly { path } => write!(f, "«{path}» es de solo lectura"),
            FileError::IsDirectory { path } => write!(f, "«{path}» es una carpeta"),
            FileError::Io { path, message } => write!(f, "error de E/S en «{path}»: {message}"),
            FileError::Decode {
                path,
                encoding,
                message,
                ..
            } => write!(
                f,
                "no se puede leer «{path}» como {}: {message}",
                encoding.id()
            ),
            FileError::Unmappable { path, report } => write!(
                f,
                "no se puede guardar «{path}» en {}: {} caracteres no caben en la codificación",
                report.encoding.id(),
                report.total
            ),
        }
    }
}

impl std::error::Error for FileError {}

impl FileError {
    /// Convierte un [`DecodeError`] de **M09** en [`FileError::Decode`].
    fn decode(path: &Path, err: &DecodeError) -> FileError {
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
}

/// Bytes de un fichero leído del disco.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileBytes {
    /// Contenido completo del fichero.
    pub bytes: Vec<u8>,
    /// `true` si el fichero está marcado como de solo lectura.
    pub read_only: bool,
}

/// Documento de texto abierto, listo para el editor.
///
/// Se serializa como `{"file":{…},"readOnly":false}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenedFile {
    /// Texto, codificación y finales de línea del documento.
    pub file: TextFile,
    /// `true` si el fichero está marcado como de solo lectura.
    pub read_only: bool,
}

/// Resultado de guardar.
///
/// Se serializa como `{"bytesWritten":12,"losses":null}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedFile {
    /// Número de bytes escritos (BOM incluido).
    pub bytes_written: usize,
    /// Informe de pérdidas si se guardó con una [`LossStrategy`]; `None` si no hubo pérdidas.
    pub losses: Option<LossReport>,
}

/// Lee el fichero completo e indica si está marcado como de solo lectura.
///
/// # Errors
///
/// [`FileError::IsDirectory`] si la ruta es una carpeta (se comprueba antes de
/// abrirla); [`FileError::NotFound`], [`FileError::PermissionDenied`] o
/// [`FileError::Io`] según el error de E/S.
///
/// # Examples
///
/// ```
/// use editormd_lib::model::file_service::read_file;
///
/// let dir = tempfile::tempdir().unwrap();
/// let path = dir.path().join("nota.md");
/// std::fs::write(&path, b"# Hola").unwrap();
///
/// let read = read_file(&path).unwrap();
/// assert_eq!(read.bytes, b"# Hola");
/// assert!(!read.read_only);
/// ```
pub fn read_file(path: &Path) -> Result<FileBytes, FileError> {
    let metadata = fs::metadata(path).map_err(|e| io_error(path, &e))?;
    if metadata.is_dir() {
        return Err(FileError::IsDirectory { path: shown(path) });
    }
    let bytes = fs::read(path).map_err(|e| io_error(path, &e))?;
    Ok(FileBytes {
        bytes,
        read_only: metadata.permissions().readonly(),
    })
}

/// Escribe `bytes` en `path` de forma atómica.
///
/// Los bytes se escriben en un fichero temporal de la misma carpeta, se
/// vuelcan al disco (`sync_all`) y el temporal se renombra sobre el destino.
/// Así, el destino tiene o el contenido anterior o el nuevo, nunca uno a
/// medias. Si algo falla, el temporal se borra. No se crean carpetas.
///
/// En Unix, si el destino ya existía, conserva sus permisos.
///
/// # Errors
///
/// [`FileError::IsDirectory`] si la ruta es una carpeta y
/// [`FileError::ReadOnly`] si el fichero existe y es de solo lectura (en
/// ambos casos sin tocar nada); [`FileError::NotFound`] si la carpeta no
/// existe; [`FileError::PermissionDenied`] o [`FileError::Io`] según el error
/// de E/S.
///
/// # Examples
///
/// ```
/// use editormd_lib::model::file_service::write_atomic;
///
/// let dir = tempfile::tempdir().unwrap();
/// let path = dir.path().join("nota.md");
///
/// write_atomic(&path, b"primera version").unwrap();
/// write_atomic(&path, b"segunda").unwrap();
/// assert_eq!(std::fs::read(&path).unwrap(), b"segunda");
///
/// // No queda ningún fichero temporal en la carpeta.
/// assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
/// ```
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), FileError> {
    let existing = match fs::metadata(path) {
        Ok(metadata) => {
            if metadata.is_dir() {
                return Err(FileError::IsDirectory { path: shown(path) });
            }
            if metadata.permissions().readonly() {
                return Err(FileError::ReadOnly { path: shown(path) });
            }
            Some(metadata)
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => None,
        Err(e) => return Err(io_error(path, &e)),
    };

    let (temp_path, file) = create_temp(path).map_err(|e| io_error(path, &e))?;
    let result = fill_temp(file, bytes, existing.as_ref(), &temp_path)
        .and_then(|()| fs::rename(&temp_path, path));
    if let Err(e) = result {
        // Mejor esfuerzo: si no se puede borrar, el error relevante es el original.
        let _ = fs::remove_file(&temp_path);
        return Err(io_error(path, &e));
    }
    sync_parent(path);
    Ok(())
}

/// Contador para que los temporales de un mismo proceso no coincidan.
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Número de intentos de crear un temporal con un nombre libre.
const TEMP_ATTEMPTS: u32 = 16;

/// Crea (sin sobrescribir nada) un fichero temporal oculto junto a `path`:
/// `.<nombre>.<pid>-<n>.tmp`.
fn create_temp(path: &Path) -> io::Result<(PathBuf, File)> {
    let name = path.file_name().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "la ruta no termina en un nombre de fichero",
        )
    })?;
    let dir = parent_dir(path);
    let mut last_error = io::Error::from(io::ErrorKind::AlreadyExists);
    for _ in 0..TEMP_ATTEMPTS {
        let n = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let mut temp_name = std::ffi::OsString::from(".");
        temp_name.push(name);
        temp_name.push(format!(".{}-{n}.tmp", std::process::id()));
        let temp_path = dir.join(temp_name);
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
        {
            Ok(file) => return Ok((temp_path, file)),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => last_error = e,
            Err(e) => return Err(e),
        }
    }
    Err(last_error)
}

/// Escribe y vuelca al disco el temporal; en Unix le copia los permisos del
/// destino si ya existía.
#[cfg_attr(not(unix), allow(unused_variables))]
fn fill_temp(
    mut file: File,
    bytes: &[u8],
    existing: Option<&fs::Metadata>,
    temp_path: &Path,
) -> io::Result<()> {
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    #[cfg(unix)]
    if let Some(metadata) = existing {
        fs::set_permissions(temp_path, metadata.permissions())?;
    }
    Ok(())
}

/// En Unix, vuelca al disco la carpeta para que el renombrado sea duradero.
/// Es de mejor esfuerzo: los datos ya están escritos y el renombrado, hecho.
#[cfg_attr(not(unix), allow(unused_variables))]
fn sync_parent(path: &Path) {
    #[cfg(unix)]
    if let Ok(dir) = File::open(parent_dir(path)) {
        let _ = dir.sync_all();
    }
}

/// Abre un documento de texto: lee el fichero y lo decodifica con **M09**.
///
/// Con `encoding = None` la codificación se detecta ([`read_text`]); con
/// `Some` se usa la indicada ([`read_text_as`]).
///
/// # Errors
///
/// Los de [`read_file`] y [`FileError::Decode`] si el contenido no es válido
/// en la codificación.
///
/// # Examples
///
/// ```
/// use editormd_lib::model::file_service::open_text;
/// use editormd_lib::model::text_codec::{Encoding, LineEnding};
///
/// let dir = tempfile::tempdir().unwrap();
/// let path = dir.path().join("nota.md");
/// std::fs::write(&path, "# Título\r\nAño\r\n").unwrap();
///
/// let opened = open_text(&path, None).unwrap();
/// assert_eq!(opened.file.text, "# Título\nAño\n");
/// assert_eq!(opened.file.encoding, Encoding::Utf8);
/// assert_eq!(opened.file.line_ending, LineEnding::Crlf);
/// assert!(!opened.read_only);
/// ```
pub fn open_text(path: &Path, encoding: Option<Encoding>) -> Result<OpenedFile, FileError> {
    let FileBytes { bytes, read_only } = read_file(path)?;
    let file = match encoding {
        None => read_text(&bytes),
        Some(encoding) => read_text_as(&bytes, encoding),
    }
    .map_err(|e| FileError::decode(path, &e))?;
    Ok(OpenedFile { file, read_only })
}

/// Guarda un documento de texto con la codificación y los finales de línea
/// indicados.
///
/// Primero se preparan los bytes sin tocar el disco: si hay caracteres que no
/// caben en `encoding`, hace falta una [`LossStrategy`]; sin ella no se
/// escribe nada. Después se escriben con [`write_atomic`].
///
/// Para un [`OpenedFile`] sin editar, guardar con su `encoding` y su
/// `line_ending` deja el fichero idéntico byte a byte (salvo finales de línea
/// mezclados, que quedan normalizados).
///
/// # Errors
///
/// [`FileError::Unmappable`] si hay pérdidas y `strategy` es `None`; los de
/// [`write_atomic`].
///
/// # Examples
///
/// ```
/// use editormd_lib::model::file_service::save_text;
/// use editormd_lib::model::text_codec::{Encoding, LineEnding, LossStrategy};
///
/// let dir = tempfile::tempdir().unwrap();
/// let path = dir.path().join("nota.txt");
///
/// let saved = save_text(
///     &path,
///     "5 €\n",
///     Encoding::Ascii,
///     LineEnding::Crlf,
///     Some(LossStrategy::Transliterate),
/// )
/// .unwrap();
/// assert_eq!(std::fs::read(&path).unwrap(), b"5 EUR\r\n");
/// assert_eq!(saved.bytes_written, 7);
/// assert_eq!(saved.losses.unwrap().total, 1);
/// ```
pub fn save_text(
    path: &Path,
    text: &str,
    encoding: Encoding,
    line_ending: LineEnding,
    strategy: Option<LossStrategy>,
) -> Result<SavedFile, FileError> {
    let report = loss_report(text, encoding);
    let strict = if report.is_lossless() {
        write_text(text, encoding, line_ending).ok()
    } else {
        None
    };
    let (bytes, losses) = match (strict, strategy) {
        (Some(bytes), _) => (bytes, None),
        (None, Some(strategy)) => (
            write_text_lossy(text, encoding, line_ending, strategy),
            Some(report),
        ),
        (None, None) => {
            return Err(FileError::Unmappable {
                path: shown(path),
                report,
            });
        }
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

/// Convierte un error de E/S en [`FileError`] según su [`io::ErrorKind`].
fn io_error(path: &Path, err: &io::Error) -> FileError {
    let path = shown(path);
    match err.kind() {
        io::ErrorKind::NotFound => FileError::NotFound { path },
        io::ErrorKind::PermissionDenied => FileError::PermissionDenied { path },
        _ => FileError::Io {
            path,
            message: err.to_string(),
        },
    }
}

/// Carpeta en la que se crea el temporal: la del destino o, si la ruta no
/// tiene carpeta (`"doc.md"`), la actual.
fn parent_dir(path: &Path) -> &Path {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io;
    use std::path::{Path, PathBuf};

    use serde_json::json;

    use super::*;
    use crate::model::text_codec::{DetectionMethod, loss_report};

    /// Nombres de las entradas de `dir`, ordenados.
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

    /// Marca `path` como de solo lectura (o lo desmarca).
    fn set_readonly(path: &Path, readonly: bool) {
        let mut perms = fs::metadata(path).unwrap().permissions();
        #[allow(clippy::permissions_set_readonly_false)]
        perms.set_readonly(readonly);
        fs::set_permissions(path, perms).unwrap();
    }

    /// Restaura los permisos de escritura al salir de ámbito (también si la
    /// prueba falla), para que la carpeta temporal se pueda borrar.
    struct RestoreWritable(PathBuf);

    impl Drop for RestoreWritable {
        fn drop(&mut self) {
            if self.0.exists() {
                set_readonly(&self.0, false);
            }
        }
    }

    fn fixtures() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/encodings")
    }

    fn fixture_files() -> Vec<PathBuf> {
        let mut files: Vec<PathBuf> = fs::read_dir(fixtures())
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.file_name().is_some_and(|n| n != "README.txt"))
            .collect();
        files.sort();
        files
    }

    // ---------------------------------------------------------------- read_file

    #[test]
    fn read_file_returns_bytes_and_writable_flag() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.md");
        fs::write(&path, b"hola\xff\x00").unwrap();
        assert_eq!(
            read_file(&path).unwrap(),
            FileBytes {
                bytes: b"hola\xff\x00".to_vec(),
                read_only: false
            }
        );
    }

    #[test]
    fn read_file_of_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vacio.txt");
        fs::write(&path, b"").unwrap();
        let read = read_file(&path).unwrap();
        assert_eq!(read.bytes, Vec::<u8>::new());
        assert!(!read.read_only);
    }

    #[test]
    fn read_file_reports_read_only() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ro.md");
        fs::write(&path, b"x").unwrap();
        set_readonly(&path, true);
        let _restore = RestoreWritable(path.clone());
        let read = read_file(&path).unwrap();
        assert_eq!(read.bytes, b"x");
        assert!(read.read_only);
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
    fn read_file_in_missing_folder_is_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("falta").join("a.md");
        assert_eq!(
            read_file(&path),
            Err(FileError::NotFound { path: shown(&path) })
        );
    }

    #[test]
    fn read_file_of_directory() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            read_file(dir.path()),
            Err(FileError::IsDirectory {
                path: shown(dir.path())
            })
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
        if result.is_ok() {
            return; // Como root no se deniega el acceso.
        }
        assert_eq!(
            result,
            Err(FileError::PermissionDenied { path: shown(&path) })
        );
    }

    // ------------------------------------------------------------- write_atomic

    #[test]
    fn write_atomic_creates_new_file_without_leftovers() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("otro.txt"), b"previo").unwrap();
        let path = dir.path().join("nuevo.md");
        write_atomic(&path, b"contenido").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"contenido");
        assert_eq!(entries(dir.path()), ["nuevo.md", "otro.txt"]);
    }

    #[test]
    fn write_atomic_replaces_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.md");
        fs::write(&path, b"un contenido anterior bastante largo").unwrap();
        write_atomic(&path, b"corto").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"corto");
        assert_eq!(entries(dir.path()), ["doc.md"]);
    }

    #[test]
    fn write_atomic_empty_content() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.md");
        fs::write(&path, b"algo").unwrap();
        write_atomic(&path, b"").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"");
        assert_eq!(entries(dir.path()), ["doc.md"]);
    }

    #[test]
    fn write_atomic_twice_in_a_row() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.md");
        write_atomic(&path, b"uno").unwrap();
        write_atomic(&path, b"dos").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"dos");
        assert_eq!(entries(dir.path()), ["doc.md"]);
    }

    #[test]
    fn write_atomic_large_content() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("grande.bin");
        let bytes: Vec<u8> = (0..3_000_000u32)
            .map(|i| u8::try_from(i % 251).unwrap())
            .collect();
        write_atomic(&path, &bytes).unwrap();
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }

    #[test]
    fn write_atomic_missing_folder_is_not_found_and_creates_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("falta").join("doc.md");
        assert_eq!(
            write_atomic(&path, b"x"),
            Err(FileError::NotFound { path: shown(&path) })
        );
        assert_eq!(entries(dir.path()), Vec::<String>::new());
    }

    #[test]
    fn write_atomic_to_directory_fails_without_changes() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("carpeta");
        fs::create_dir(&sub).unwrap();
        assert_eq!(
            write_atomic(&sub, b"x"),
            Err(FileError::IsDirectory { path: shown(&sub) })
        );
        assert_eq!(entries(dir.path()), ["carpeta"]);
        assert_eq!(entries(&sub), Vec::<String>::new());
    }

    #[test]
    fn write_atomic_read_only_target_is_left_intact() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ro.md");
        fs::write(&path, b"original").unwrap();
        set_readonly(&path, true);
        let _restore = RestoreWritable(path.clone());
        assert_eq!(
            write_atomic(&path, b"nuevo"),
            Err(FileError::ReadOnly { path: shown(&path) })
        );
        assert_eq!(fs::read(&path).unwrap(), b"original");
        assert_eq!(entries(dir.path()), ["ro.md"]);
    }

    /// En Windows, un fichero abierto sin compartir no se puede sustituir:
    /// el renombrado falla y no debe quedar ni el destino cambiado ni el temporal.
    #[cfg(windows)]
    #[test]
    fn write_atomic_failed_rename_keeps_target_and_removes_temp() {
        use std::os::windows::fs::OpenOptionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bloqueado.md");
        fs::write(&path, b"original").unwrap();
        let lock = fs::OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(&path)
            .unwrap();
        let result = write_atomic(&path, b"nuevo");
        drop(lock);
        let err = result.unwrap_err();
        assert!(
            matches!(
                err,
                FileError::Io { .. } | FileError::PermissionDenied { .. }
            ),
            "{err:?}"
        );
        assert_eq!(fs::read(&path).unwrap(), b"original");
        assert_eq!(entries(dir.path()), ["bloqueado.md"]);
    }

    #[cfg(unix)]
    #[test]
    fn write_atomic_keeps_mode_of_existing_file() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.md");
        fs::write(&path, b"x").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
        write_atomic(&path, b"y").unwrap();
        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o640);
        assert_eq!(fs::read(&path).unwrap(), b"y");
    }

    #[cfg(unix)]
    #[test]
    fn write_atomic_in_read_only_folder_leaves_nothing() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("cerrada");
        fs::create_dir(&sub).unwrap();
        let path = sub.join("doc.md");
        fs::write(&path, b"original").unwrap();
        fs::set_permissions(&sub, fs::Permissions::from_mode(0o555)).unwrap();
        let result = write_atomic(&path, b"nuevo");
        fs::set_permissions(&sub, fs::Permissions::from_mode(0o755)).unwrap();
        if result.is_ok() {
            return; // Como root no se deniega el acceso.
        }
        assert_eq!(
            result,
            Err(FileError::PermissionDenied { path: shown(&path) })
        );
        assert_eq!(fs::read(&path).unwrap(), b"original");
        assert_eq!(entries(&sub), ["doc.md"]);
    }

    // ---------------------------------------------------------------- open_text

    #[test]
    fn open_text_autodetects() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.md");
        fs::write(&path, "# Título\r\nñ\r\n").unwrap();
        let opened = open_text(&path, None).unwrap();
        assert!(!opened.read_only);
        assert_eq!(opened.file.text, "# Título\nñ\n");
        assert_eq!(opened.file.encoding, Encoding::Utf8);
        assert_eq!(opened.file.line_ending, LineEnding::Crlf);
        assert!(!opened.file.mixed_line_endings);
        assert_eq!(opened.file.detection, Some(DetectionMethod::Utf8Valid));
    }

    #[test]
    fn open_text_with_explicit_encoding() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("latin1.txt");
        fs::write(&path, b"caf\xe9\n").unwrap();
        let opened = open_text(&path, Some(Encoding::Iso8859_1)).unwrap();
        assert_eq!(opened.file.text, "café\n");
        assert_eq!(opened.file.encoding, Encoding::Iso8859_1);
        assert_eq!(opened.file.detection, None);
    }

    #[test]
    fn open_text_reports_read_only() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ro.md");
        fs::write(&path, b"x").unwrap();
        set_readonly(&path, true);
        let _restore = RestoreWritable(path.clone());
        let opened = open_text(&path, None).unwrap();
        assert!(opened.read_only);
        assert_eq!(opened.file.text, "x");
    }

    #[test]
    fn open_text_invalid_sequence_is_decode_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mal.txt");
        fs::write(&path, b"abc\xffd").unwrap();
        let err = open_text(&path, Some(Encoding::Ascii)).unwrap_err();
        let FileError::Decode {
            path: p,
            encoding,
            offset,
            message,
        } = err
        else {
            panic!("se esperaba Decode: {err:?}");
        };
        assert_eq!(p, shown(&path));
        assert_eq!(encoding, Encoding::Ascii);
        assert_eq!(offset, Some(3));
        let expected = crate::model::text_codec::read_text_as(b"abc\xffd", Encoding::Ascii)
            .unwrap_err()
            .to_string();
        assert_eq!(message, expected);
    }

    #[test]
    fn open_text_autodetected_bom_with_invalid_content() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bom.txt");
        fs::write(&path, b"\xef\xbb\xbfab\xff").unwrap();
        let err = open_text(&path, None).unwrap_err();
        assert!(
            matches!(
                err,
                FileError::Decode {
                    encoding: Encoding::Utf8Bom,
                    offset: Some(5),
                    ..
                }
            ),
            "{err:?}"
        );
    }

    #[test]
    fn open_text_odd_length_has_no_offset() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("impar.txt");
        fs::write(&path, b"\xff\xfea\x00b").unwrap();
        let err = open_text(&path, None).unwrap_err();
        let FileError::Decode {
            encoding,
            offset,
            message,
            ..
        } = err
        else {
            panic!("se esperaba Decode: {err:?}");
        };
        assert_eq!(encoding, Encoding::Utf16Le);
        assert_eq!(offset, None);
        assert!(message.contains("impar"), "{message}");
    }

    #[test]
    fn open_text_propagates_file_errors() {
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

    // ---------------------------------------------------------------- save_text

    #[test]
    fn save_text_lossless() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.md");
        let saved = save_text(&path, "a\nñ\n", Encoding::Utf8, LineEnding::Crlf, None).unwrap();
        assert_eq!(
            saved,
            SavedFile {
                bytes_written: 7,
                losses: None
            }
        );
        assert_eq!(fs::read(&path).unwrap(), "a\r\nñ\r\n".as_bytes());
        assert_eq!(entries(dir.path()), ["doc.md"]);
    }

    #[test]
    fn save_text_counts_bom() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.md");
        let saved = save_text(&path, "ab", Encoding::Utf16Be, LineEnding::Lf, None).unwrap();
        assert_eq!(saved.bytes_written, 6);
        assert_eq!(fs::read(&path).unwrap(), b"\xfe\xff\x00a\x00b");
        let saved = save_text(&path, "", Encoding::Utf8Bom, LineEnding::Lf, None).unwrap();
        assert_eq!(saved.bytes_written, 3);
        assert_eq!(fs::read(&path).unwrap(), b"\xef\xbb\xbf");
    }

    #[test]
    fn save_text_lossless_ignores_strategy() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.md");
        let saved = save_text(
            &path,
            "abc",
            Encoding::Ascii,
            LineEnding::Lf,
            Some(LossStrategy::Replace),
        )
        .unwrap();
        assert_eq!(saved.losses, None);
        assert_eq!(fs::read(&path).unwrap(), b"abc");
    }

    #[test]
    fn save_text_unmappable_without_strategy_creates_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.md");
        let text = "precio: 5 €\notra línea €\n";
        assert_eq!(
            save_text(&path, text, Encoding::Iso8859_1, LineEnding::Lf, None),
            Err(FileError::Unmappable {
                path: shown(&path),
                report: loss_report(text, Encoding::Iso8859_1),
            })
        );
        assert_eq!(entries(dir.path()), Vec::<String>::new());
    }

    #[test]
    fn save_text_unmappable_without_strategy_keeps_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.md");
        fs::write(&path, b"original").unwrap();
        let err = save_text(&path, "😀", Encoding::Ascii, LineEnding::Lf, None).unwrap_err();
        assert!(matches!(err, FileError::Unmappable { .. }), "{err:?}");
        assert_eq!(fs::read(&path).unwrap(), b"original");
        assert_eq!(entries(dir.path()), ["doc.md"]);
    }

    #[test]
    fn save_text_unmappable_is_checked_before_disk_errors() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("falta").join("doc.md");
        let err = save_text(&path, "€", Encoding::Ascii, LineEnding::Lf, None).unwrap_err();
        assert!(matches!(err, FileError::Unmappable { .. }), "{err:?}");
    }

    #[test]
    fn save_text_with_strategy_reports_losses() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.md");
        let text = "5 €\nñ\n";
        let saved = save_text(
            &path,
            text,
            Encoding::Ascii,
            LineEnding::Crlf,
            Some(LossStrategy::Transliterate),
        )
        .unwrap();
        let expected = b"5 EUR\r\nn\r\n";
        assert_eq!(fs::read(&path).unwrap(), expected);
        assert_eq!(
            saved,
            SavedFile {
                bytes_written: expected.len(),
                losses: Some(loss_report(text, Encoding::Ascii)),
            }
        );
    }

    #[test]
    fn save_text_with_replace_strategy() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.md");
        let saved = save_text(
            &path,
            "a€b",
            Encoding::Iso8859_1,
            LineEnding::Lf,
            Some(LossStrategy::Replace),
        )
        .unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"a?b");
        assert_eq!(saved.bytes_written, 3);
        assert_eq!(saved.losses.unwrap().total, 1);
    }

    #[test]
    fn save_text_propagates_write_errors() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("falta").join("doc.md");
        assert_eq!(
            save_text(&missing, "x", Encoding::Utf8, LineEnding::Lf, None),
            Err(FileError::NotFound {
                path: shown(&missing)
            })
        );
        assert_eq!(
            save_text(dir.path(), "x", Encoding::Utf8, LineEnding::Lf, None),
            Err(FileError::IsDirectory {
                path: shown(dir.path())
            })
        );
        let ro = dir.path().join("ro.md");
        fs::write(&ro, b"original").unwrap();
        set_readonly(&ro, true);
        let _restore = RestoreWritable(ro.clone());
        assert_eq!(
            save_text(&ro, "x", Encoding::Utf8, LineEnding::Lf, None),
            Err(FileError::ReadOnly { path: shown(&ro) })
        );
        assert_eq!(fs::read(&ro).unwrap(), b"original");
    }

    #[test]
    fn open_then_save_is_byte_identical_for_all_fixtures() {
        let dir = tempfile::tempdir().unwrap();
        for source in fixture_files() {
            let name = source.file_name().unwrap().to_owned();
            let original = fs::read(&source).unwrap();
            let path = dir.path().join(&name);
            fs::write(&path, &original).unwrap();

            let f = open_text(&path, None).unwrap().file;
            let saved = save_text(&path, &f.text, f.encoding, f.line_ending, None).unwrap();
            let written = fs::read(&path).unwrap();
            assert_eq!(saved.bytes_written, written.len(), "{name:?}");
            assert_eq!(saved.losses, None, "{name:?}");
            if f.mixed_line_endings {
                assert_eq!(written, b"uno\ndos\ntres\ncuatro\ncinco", "{name:?}");
            } else {
                assert_eq!(written, original, "{name:?}");
            }
        }
    }

    #[test]
    fn open_then_save_with_declared_encoding_is_byte_identical() {
        let dir = tempfile::tempdir().unwrap();
        for encoding in Encoding::ALL {
            for le in LineEnding::ALL {
                let name = format!("{}-{}.txt", encoding.id(), le.id());
                let original = fs::read(fixtures().join(&name)).unwrap();
                let path = dir.path().join(&name);
                fs::write(&path, &original).unwrap();
                let f = open_text(&path, Some(encoding)).unwrap().file;
                assert_eq!(f.encoding, encoding);
                save_text(&path, &f.text, f.encoding, f.line_ending, None).unwrap();
                assert_eq!(fs::read(&path).unwrap(), original, "{name}");
            }
        }
        assert_eq!(entries(dir.path()).len(), 27);
    }

    // ------------------------------------------------------------- serialización

    #[test]
    fn file_error_serialization() {
        let p = "C:/docs/a.md".to_owned();
        let cases = [
            (
                FileError::NotFound { path: p.clone() },
                json!({"kind": "not-found", "path": p}),
            ),
            (
                FileError::PermissionDenied { path: p.clone() },
                json!({"kind": "permission-denied", "path": p}),
            ),
            (
                FileError::ReadOnly { path: p.clone() },
                json!({"kind": "read-only", "path": p}),
            ),
            (
                FileError::IsDirectory { path: p.clone() },
                json!({"kind": "is-directory", "path": p}),
            ),
            (
                FileError::Io {
                    path: p.clone(),
                    message: "fallo".to_owned(),
                },
                json!({"kind": "io", "path": p, "message": "fallo"}),
            ),
            (
                FileError::Decode {
                    path: p.clone(),
                    encoding: Encoding::Ascii,
                    offset: Some(3),
                    message: "m".to_owned(),
                },
                json!({"kind": "decode", "path": p, "encoding": "ascii", "offset": 3, "message": "m"}),
            ),
            (
                FileError::Decode {
                    path: p.clone(),
                    encoding: Encoding::Utf16Be,
                    offset: None,
                    message: "m".to_owned(),
                },
                json!({"kind": "decode", "path": p, "encoding": "utf-16be", "offset": null, "message": "m"}),
            ),
        ];
        for (err, expected) in cases {
            assert_eq!(serde_json::to_value(&err).unwrap(), expected, "{err:?}");
        }
    }

    #[test]
    fn unmappable_serialization_embeds_report() {
        let report = loss_report("€", Encoding::Ascii);
        let err = FileError::Unmappable {
            path: "a.md".to_owned(),
            report: report.clone(),
        };
        assert_eq!(
            serde_json::to_value(&err).unwrap(),
            json!({
                "kind": "unmappable",
                "path": "a.md",
                "report": serde_json::to_value(&report).unwrap(),
            })
        );
    }

    #[test]
    fn opened_and_saved_serialization() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("doc.md");
        fs::write(&path, b"hola\n").unwrap();
        let opened = open_text(&path, None).unwrap();
        assert_eq!(
            serde_json::to_value(&opened).unwrap(),
            json!({
                "file": serde_json::to_value(&opened.file).unwrap(),
                "readOnly": false,
            })
        );
        let saved = SavedFile {
            bytes_written: 12,
            losses: None,
        };
        assert_eq!(
            serde_json::to_string(&saved).unwrap(),
            r#"{"bytesWritten":12,"losses":null}"#
        );
        let lossy = SavedFile {
            bytes_written: 1,
            losses: Some(loss_report("€", Encoding::Ascii)),
        };
        assert_eq!(
            serde_json::to_value(&lossy).unwrap()["losses"]["total"],
            json!(1)
        );
    }

    // ----------------------------------------------------- Display y auxiliares

    #[test]
    fn display_is_spanish_and_includes_path() {
        let p = "/tmp/carpeta/doc.md".to_owned();
        let errors = [
            FileError::NotFound { path: p.clone() },
            FileError::PermissionDenied { path: p.clone() },
            FileError::ReadOnly { path: p.clone() },
            FileError::IsDirectory { path: p.clone() },
            FileError::Io {
                path: p.clone(),
                message: "disco lleno".to_owned(),
            },
            FileError::Decode {
                path: p.clone(),
                encoding: Encoding::Ascii,
                offset: Some(3),
                message: "secuencia no válida".to_owned(),
            },
            FileError::Unmappable {
                path: p.clone(),
                report: loss_report("€€", Encoding::Ascii),
            },
        ];
        let mut messages = Vec::new();
        for err in &errors {
            let msg = err.to_string();
            assert!(msg.contains(&p), "{msg}");
            messages.push(msg);
        }
        assert!(messages[4].contains("disco lleno"));
        assert!(messages[5].contains("ascii"));
        assert!(messages[5].contains("secuencia no válida"));
        assert!(messages[6].contains('2'));
        assert!(messages[6].contains("ascii"));
        let mut unique = messages.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), messages.len(), "mensajes repetidos");
        let as_error: &dyn std::error::Error = &errors[0];
        assert!(as_error.source().is_none());
    }

    #[test]
    fn io_error_mapping() {
        let path = Path::new("x/y.md");
        assert_eq!(
            io_error(path, &io::Error::from(io::ErrorKind::NotFound)),
            FileError::NotFound { path: shown(path) }
        );
        assert_eq!(
            io_error(path, &io::Error::from(io::ErrorKind::PermissionDenied)),
            FileError::PermissionDenied { path: shown(path) }
        );
        let other = io::Error::other("disco lleno");
        assert_eq!(
            io_error(path, &other),
            FileError::Io {
                path: shown(path),
                message: other.to_string(),
            }
        );
    }

    #[test]
    fn parent_dir_of_bare_file_name_is_current_dir() {
        assert_eq!(parent_dir(Path::new("doc.md")), Path::new("."));
        assert_eq!(parent_dir(Path::new("a/doc.md")), Path::new("a"));
    }
}
