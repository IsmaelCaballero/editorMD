//! **M08 `FileService`** (Modelo · Rust) — acceso al disco: lectura, escritura
//! atómica y solo lectura, y su combinación con **M09 `TextCodec`** para abrir y
//! guardar documentos de texto (`PLAN.md` §2.5 y §4).
//!
//! - [`read_file`] / [`write_atomic`]: bytes ⇄ disco. La escritura va a un
//!   temporal en la misma carpeta y se renombra sobre el destino, de modo que un
//!   fallo nunca deja el fichero a medias ni temporales sueltos.
//! - [`open_text`] / [`save_text`]: texto ⇄ disco, con codificación, BOM y fin de
//!   línea (ida y vuelta byte a byte) y tratamiento de los caracteres que no caben.
//!
//! Los errores ([`FileError`]) son serializables para que el frontend los muestre.

use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;

use crate::model::text_codec::{
    DecodeError, Encoding, LineEnding, LossReport, LossStrategy, TextFile, loss_report, read_text,
    read_text_as, write_text, write_text_lossy,
};

/// Error al abrir o guardar un fichero. Serializable para el frontend.
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
    /// Cualquier otro error de E/S; `message` = el `Display` del `std::io::Error`.
    Io {
        /// Ruta recibida.
        path: String,
        /// Mensaje del error de E/S.
        message: String,
    },
    /// El contenido no es válido en la codificación usada. `offset` = el del
    /// `DecodeError::InvalidSequence` (`None` en `OddLength`); `message` = el `Display`
    /// del `DecodeError`.
    Decode {
        /// Ruta recibida.
        path: String,
        /// Codificación con la que se intentó decodificar.
        encoding: Encoding,
        /// Posición en bytes de la secuencia no válida, si se conoce.
        offset: Option<usize>,
        /// Mensaje del error de decodificación.
        message: String,
    },
    /// Al guardar sin estrategia, hay caracteres que no caben en la codificación.
    /// `report` = `loss_report(text, encoding)` del texto recibido.
    Unmappable {
        /// Ruta recibida.
        path: String,
        /// Informe de los caracteres que no caben.
        report: LossReport,
    },
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileError::NotFound { path } => write!(f, "no se encuentra «{path}»"),
            FileError::PermissionDenied { path } => write!(f, "acceso denegado a «{path}»"),
            FileError::ReadOnly { path } => write!(f, "«{path}» es de solo lectura"),
            FileError::IsDirectory { path } => write!(f, "«{path}» es una carpeta"),
            FileError::Io { path, message } => {
                write!(f, "error de E/S con «{path}»: {message}")
            }
            FileError::Decode {
                path,
                encoding,
                message,
                ..
            } => write!(f, "«{path}» no es válido en {}: {message}", encoding.id()),
            FileError::Unmappable { path, report } => write!(
                f,
                "no se puede guardar «{path}» en {}: {} carácter(es) no caben",
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
    /// Texto, codificación y fin de línea del fichero.
    pub file: TextFile,
    /// `true` si el fichero está marcado como de solo lectura.
    pub read_only: bool,
}

/// Resultado de guardar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedFile {
    /// Bytes escritos (BOM incluido).
    pub bytes_written: usize,
    /// Informe de lo perdido si se guardó con una [`LossStrategy`]; `None` si no hubo pérdidas.
    pub losses: Option<LossReport>,
}

/// Traduce un error de E/S al [`FileError`] correspondiente.
fn map_io(path: &Path, err: &io::Error) -> FileError {
    let path = path.display().to_string();
    match err.kind() {
        io::ErrorKind::NotFound => FileError::NotFound { path },
        io::ErrorKind::PermissionDenied => FileError::PermissionDenied { path },
        _ => FileError::Io {
            path,
            message: err.to_string(),
        },
    }
}

/// Traduce un error de decodificación al [`FileError`] correspondiente.
fn map_decode(path: &Path, err: &DecodeError) -> FileError {
    let (encoding, offset) = match err {
        DecodeError::InvalidSequence { encoding, offset } => (*encoding, Some(*offset)),
        DecodeError::OddLength { encoding } => (*encoding, None),
    };
    FileError::Decode {
        path: path.display().to_string(),
        encoding,
        offset,
        message: err.to_string(),
    }
}

/// Lee todo el contenido de un fichero y si es de solo lectura.
///
/// # Errors
///
/// [`FileError::IsDirectory`] si la ruta es una carpeta; [`FileError::NotFound`],
/// [`FileError::PermissionDenied`] o [`FileError::Io`] según el error de E/S.
///
/// # Examples
///
/// ```
/// use editormd_lib::model::file_service::{read_file, write_atomic};
///
/// let dir = std::env::temp_dir().join("editormd_doc_read_file");
/// std::fs::create_dir_all(&dir).unwrap();
/// let path = dir.join("a.txt");
/// write_atomic(&path, b"hola").unwrap();
/// let leido = read_file(&path).unwrap();
/// assert_eq!(leido.bytes, b"hola");
/// assert!(!leido.read_only);
/// std::fs::remove_dir_all(&dir).unwrap();
/// ```
pub fn read_file(path: &Path) -> Result<FileBytes, FileError> {
    todo!()
}

/// Contador para que los nombres de temporales no coincidan dentro del proceso.
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Crea un fichero temporal nuevo (`create_new`) en `dir` y devuelve su ruta y el fichero.
fn create_temp(dir: &Path, name: &std::ffi::OsStr) -> io::Result<(PathBuf, fs::File)> {
    let mut last = io::Error::new(io::ErrorKind::AlreadyExists, "sin nombre temporal libre");
    for _ in 0..100 {
        let n = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let mut temp_name = std::ffi::OsString::from(".");
        temp_name.push(name);
        temp_name.push(format!(".{}.{n}.tmp", std::process::id()));
        let temp_path = dir.join(temp_name);
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
        {
            Ok(file) => return Ok((temp_path, file)),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => last = e,
            Err(e) => return Err(e),
        }
    }
    Err(last)
}

/// Escribe `bytes` en un temporal, lo vuelca al disco y lo renombra con `rename`.
/// Si algo falla, borra el temporal.
#[allow(dead_code)]
fn write_via_temp(
    path: &Path,
    bytes: &[u8],
    existing: Option<&fs::Permissions>,
    rename: impl FnOnce(&Path, &Path) -> io::Result<()>,
) -> Result<(), FileError> {
    let Some(name) = path.file_name() else {
        return Err(FileError::Io {
            path: path.display().to_string(),
            message: "la ruta no indica un nombre de fichero".to_string(),
        });
    };
    let dir = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    };
    let (temp_path, mut file) = create_temp(dir, name).map_err(|e| map_io(path, &e))?;
    let result = (|| {
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        if cfg!(unix) {
            if let Some(perms) = existing {
                fs::set_permissions(&temp_path, perms.clone())?;
            }
        }
        rename(&temp_path, path)
    })();
    if let Err(e) = result {
        // Mejor esfuerzo: el error relevante es el original.
        let _ = fs::remove_file(&temp_path);
        return Err(map_io(path, &e));
    }
    Ok(())
}

/// Escribe `bytes` en `path` de forma **atómica**: temporal en la misma carpeta,
/// `sync_all` y renombrado sobre el destino.
///
/// Si falla, el destino queda intacto y no queda ningún temporal. Si el destino
/// existía (en Unix) se conservan sus permisos. No crea carpetas.
///
/// # Errors
///
/// [`FileError::IsDirectory`] si la ruta es una carpeta; [`FileError::ReadOnly`] si
/// el fichero existe y es de solo lectura; [`FileError::NotFound`] si no existe la
/// carpeta; [`FileError::PermissionDenied`] o [`FileError::Io`] en otro caso.
///
/// # Examples
///
/// ```
/// use editormd_lib::model::file_service::write_atomic;
///
/// let dir = std::env::temp_dir().join("editormd_doc_write_atomic");
/// std::fs::create_dir_all(&dir).unwrap();
/// let path = dir.join("nota.txt");
/// write_atomic(&path, b"uno").unwrap();
/// write_atomic(&path, b"dos").unwrap();
/// assert_eq!(std::fs::read(&path).unwrap(), b"dos");
/// std::fs::remove_dir_all(&dir).unwrap();
/// ```
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), FileError> {
    todo!()
}

/// Como [`write_atomic`] con el renombrado inyectable (para probar los fallos).
fn write_atomic_with(
    path: &Path,
    bytes: &[u8],
    rename: impl FnOnce(&Path, &Path) -> io::Result<()>,
) -> Result<(), FileError> {
    todo!()
}

/// Abre un fichero de texto: [`read_file`] y después [`read_text`] (detección
/// automática si `encoding` es `None`) o [`read_text_as`] (`Some`).
///
/// # Errors
///
/// Los de [`read_file`], y [`FileError::Decode`] si el contenido no es válido.
///
/// # Examples
///
/// ```
/// use editormd_lib::model::file_service::{open_text, write_atomic};
/// use editormd_lib::model::text_codec::{Encoding, LineEnding};
///
/// let dir = std::env::temp_dir().join("editormd_doc_open_text");
/// std::fs::create_dir_all(&dir).unwrap();
/// let path = dir.join("a.md");
/// write_atomic(&path, b"uno\r\ndos\r\n").unwrap();
/// let abierto = open_text(&path, None).unwrap();
/// assert_eq!(abierto.file.text, "uno\ndos\n");
/// assert_eq!(abierto.file.encoding, Encoding::Utf8);
/// assert_eq!(abierto.file.line_ending, LineEnding::Crlf);
/// assert!(!abierto.read_only);
/// std::fs::remove_dir_all(&dir).unwrap();
/// ```
pub fn open_text(path: &Path, encoding: Option<Encoding>) -> Result<OpenedFile, FileError> {
    todo!()
}

/// Guarda `text` en `path` con la codificación y el fin de línea indicados.
///
/// Los bytes se preparan primero (sin tocar el disco) y luego se escriben con
/// [`write_atomic`]. Si hay caracteres que no caben en `encoding` y no hay
/// `strategy`, se devuelve [`FileError::Unmappable`] sin tocar el disco.
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
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn names(dir: &Path) -> Vec<String> {
        let mut v: Vec<String> = fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        v.sort();
        v
    }

    fn set_readonly(path: &Path, ro: bool) {
        let mut p = fs::metadata(path).unwrap().permissions();
        p.set_readonly(ro);
        fs::set_permissions(path, p).unwrap();
    }

    // --- read_file ---

    #[test]
    fn read_file_lee_bytes() {
        let d = tempdir().unwrap();
        let p = d.path().join("a.bin");
        fs::write(&p, [0u8, 1, 2, 255]).unwrap();
        let r = read_file(&p).unwrap();
        assert_eq!(r.bytes, vec![0, 1, 2, 255]);
        assert!(!r.read_only);
    }

    #[test]
    fn read_file_vacio() {
        let d = tempdir().unwrap();
        let p = d.path().join("v");
        fs::write(&p, b"").unwrap();
        assert!(read_file(&p).unwrap().bytes.is_empty());
    }

    #[test]
    fn read_file_no_existe() {
        let d = tempdir().unwrap();
        let p = d.path().join("nada");
        assert_eq!(
            read_file(&p),
            Err(FileError::NotFound {
                path: p.display().to_string()
            })
        );
    }

    #[test]
    fn read_file_carpeta() {
        let d = tempdir().unwrap();
        assert_eq!(
            read_file(d.path()),
            Err(FileError::IsDirectory {
                path: d.path().display().to_string()
            })
        );
    }

    #[test]
    fn read_file_solo_lectura() {
        let d = tempdir().unwrap();
        let p = d.path().join("ro.txt");
        fs::write(&p, b"x").unwrap();
        set_readonly(&p, true);
        let r = read_file(&p);
        set_readonly(&p, false);
        let r = r.unwrap();
        assert_eq!(r.bytes, b"x");
        assert!(r.read_only);
    }

    // --- write_atomic ---

    #[test]
    fn write_atomic_crea_fichero_nuevo() {
        let d = tempdir().unwrap();
        let p = d.path().join("n.txt");
        write_atomic(&p, b"hola").unwrap();
        assert_eq!(fs::read(&p).unwrap(), b"hola");
        assert_eq!(names(d.path()), vec!["n.txt"]);
    }

    #[test]
    fn write_atomic_sustituye_existente_sin_temporales() {
        let d = tempdir().unwrap();
        let p = d.path().join("e.txt");
        fs::write(&p, b"viejo contenido largo").unwrap();
        fs::write(d.path().join("otro.txt"), b"o").unwrap();
        write_atomic(&p, b"nuevo").unwrap();
        assert_eq!(fs::read(&p).unwrap(), b"nuevo");
        assert_eq!(names(d.path()), vec!["e.txt", "otro.txt"]);
    }

    #[test]
    fn write_atomic_bytes_vacios() {
        let d = tempdir().unwrap();
        let p = d.path().join("v.txt");
        fs::write(&p, b"algo").unwrap();
        write_atomic(&p, b"").unwrap();
        assert!(fs::read(&p).unwrap().is_empty());
    }

    #[test]
    fn write_atomic_carpeta_destino() {
        let d = tempdir().unwrap();
        let sub = d.path().join("sub");
        fs::create_dir(&sub).unwrap();
        assert_eq!(
            write_atomic(&sub, b"x"),
            Err(FileError::IsDirectory {
                path: sub.display().to_string()
            })
        );
        assert_eq!(names(d.path()), vec!["sub"]);
    }

    #[test]
    fn write_atomic_solo_lectura_no_modifica() {
        let d = tempdir().unwrap();
        let p = d.path().join("ro.txt");
        fs::write(&p, b"intacto").unwrap();
        set_readonly(&p, true);
        let r = write_atomic(&p, b"cambio");
        let contenido = fs::read(&p).unwrap();
        let nombres = names(d.path());
        set_readonly(&p, false);
        assert_eq!(
            r,
            Err(FileError::ReadOnly {
                path: p.display().to_string()
            })
        );
        assert_eq!(contenido, b"intacto");
        assert_eq!(nombres, vec!["ro.txt"]);
    }

    #[test]
    fn write_atomic_carpeta_inexistente() {
        let d = tempdir().unwrap();
        let p = d.path().join("falta").join("a.txt");
        assert_eq!(
            write_atomic(&p, b"x"),
            Err(FileError::NotFound {
                path: p.display().to_string()
            })
        );
        assert!(!d.path().join("falta").exists());
    }

    #[test]
    fn write_atomic_ruta_sin_nombre_de_fichero() {
        // Un destino inexistente cuya ruta termina en «..» no tiene nombre de fichero.
        let p = Path::new("carpeta_que_no_existe_editormd").join("..");
        let e = write_atomic_with(&p, b"x", |_, _| Ok(()));
        assert!(matches!(
            e,
            Err(FileError::Io { .. } | FileError::IsDirectory { .. })
        ));
        let e = write_via_temp(Path::new(".."), b"x", None, |_, _| Ok(()));
        assert!(matches!(e, Err(FileError::Io { .. })));
    }

    #[test]
    fn write_atomic_fallo_al_renombrar_deja_destino_y_sin_temporales() {
        let d = tempdir().unwrap();
        let p = d.path().join("f.txt");
        fs::write(&p, b"original").unwrap();
        let r = write_atomic_with(&p, b"nuevo", |_, _| Err(io::Error::other("fallo simulado")));
        assert_eq!(
            r,
            Err(FileError::Io {
                path: p.display().to_string(),
                message: "fallo simulado".to_string()
            })
        );
        assert_eq!(fs::read(&p).unwrap(), b"original");
        assert_eq!(names(d.path()), vec!["f.txt"]);
    }

    #[test]
    fn write_atomic_fallo_al_renombrar_destino_nuevo_no_lo_crea() {
        let d = tempdir().unwrap();
        let p = d.path().join("nuevo.txt");
        let r = write_atomic_with(&p, b"x", |_, _| {
            Err(io::Error::from(io::ErrorKind::PermissionDenied))
        });
        assert_eq!(
            r,
            Err(FileError::PermissionDenied {
                path: p.display().to_string()
            })
        );
        assert!(names(d.path()).is_empty());
    }

    #[test]
    fn write_atomic_ruta_relativa_sin_carpeta() {
        // Un nombre sin carpeta usa «.» (el directorio actual).
        let nombre = format!("editormd_t_{}.tmp_rel", std::process::id());
        let p = Path::new(&nombre);
        write_atomic(p, b"rel").unwrap();
        let leido = fs::read(p).unwrap();
        fs::remove_file(p).unwrap();
        assert_eq!(leido, b"rel");
    }

    #[cfg(unix)]
    #[test]
    fn write_atomic_conserva_permisos_en_unix() {
        use std::os::unix::fs::PermissionsExt;
        let d = tempdir().unwrap();
        let p = d.path().join("x.sh");
        fs::write(&p, b"a").unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o751)).unwrap();
        write_atomic(&p, b"b").unwrap();
        let modo = fs::metadata(&p).unwrap().permissions().mode() & 0o777;
        assert_eq!(modo, 0o751);
    }

    // --- open_text ---

    #[test]
    fn open_text_detecta_codificacion_y_fin_de_linea() {
        let d = tempdir().unwrap();
        let p = d.path().join("a.md");
        fs::write(&p, b"\xef\xbb\xbfuno\r\ndos\r\n").unwrap();
        let o = open_text(&p, None).unwrap();
        assert_eq!(o.file.text, "uno\ndos\n");
        assert_eq!(o.file.encoding, Encoding::Utf8Bom);
        assert_eq!(o.file.line_ending, LineEnding::Crlf);
        assert!(!o.read_only);
    }

    #[test]
    fn open_text_con_codificacion_explicita() {
        let d = tempdir().unwrap();
        let p = d.path().join("l.txt");
        fs::write(&p, b"caf\xe9").unwrap();
        let o = open_text(&p, Some(Encoding::Iso8859_1)).unwrap();
        assert_eq!(o.file.text, "café");
        assert_eq!(o.file.encoding, Encoding::Iso8859_1);
        assert_eq!(o.file.detection, None);
    }

    #[test]
    fn open_text_error_de_decodificacion_con_offset() {
        let d = tempdir().unwrap();
        let p = d.path().join("a.txt");
        fs::write(&p, b"abc\xe9").unwrap();
        let e = open_text(&p, Some(Encoding::Ascii)).unwrap_err();
        let FileError::Decode {
            path,
            encoding,
            offset,
            message,
        } = e
        else {
            panic!("se esperaba Decode");
        };
        assert_eq!(path, p.display().to_string());
        assert_eq!(encoding, Encoding::Ascii);
        assert_eq!(offset, Some(3));
        assert!(!message.is_empty());
    }

    #[test]
    fn open_text_utf16_longitud_impar_sin_offset() {
        let d = tempdir().unwrap();
        let p = d.path().join("u.txt");
        fs::write(&p, [0x61u8, 0x00, 0x62]).unwrap();
        let e = open_text(&p, Some(Encoding::Utf16Le)).unwrap_err();
        assert!(matches!(
            e,
            FileError::Decode {
                encoding: Encoding::Utf16Le,
                offset: None,
                ..
            }
        ));
    }

    #[test]
    fn open_text_no_existe_y_carpeta() {
        let d = tempdir().unwrap();
        assert!(matches!(
            open_text(&d.path().join("no"), None),
            Err(FileError::NotFound { .. })
        ));
        assert!(matches!(
            open_text(d.path(), None),
            Err(FileError::IsDirectory { .. })
        ));
    }

    #[test]
    fn open_text_solo_lectura() {
        let d = tempdir().unwrap();
        let p = d.path().join("ro.md");
        fs::write(&p, b"x").unwrap();
        set_readonly(&p, true);
        let o = open_text(&p, None);
        set_readonly(&p, false);
        assert!(o.unwrap().read_only);
    }

    // --- save_text ---

    #[test]
    fn save_text_escribe_con_bom_y_fin_de_linea() {
        let d = tempdir().unwrap();
        let p = d.path().join("s.txt");
        let s = save_text(&p, "a\nb\n", Encoding::Utf8Bom, LineEnding::Crlf, None).unwrap();
        let esperado = b"\xef\xbb\xbfa\r\nb\r\n";
        assert_eq!(fs::read(&p).unwrap(), esperado);
        assert_eq!(s.bytes_written, esperado.len());
        assert_eq!(s.losses, None);
    }

    #[test]
    fn save_text_sin_estrategia_y_con_perdidas_no_toca_el_disco() {
        let d = tempdir().unwrap();
        let p = d.path().join("s.txt");
        let e = save_text(&p, "10 €", Encoding::Ascii, LineEnding::Lf, None).unwrap_err();
        assert_eq!(
            e,
            FileError::Unmappable {
                path: p.display().to_string(),
                report: loss_report("10 €", Encoding::Ascii)
            }
        );
        assert!(names(d.path()).is_empty());
    }

    #[test]
    fn save_text_sin_estrategia_conserva_el_fichero_existente() {
        let d = tempdir().unwrap();
        let p = d.path().join("s.txt");
        fs::write(&p, b"previo").unwrap();
        assert!(save_text(&p, "€", Encoding::Ascii, LineEnding::Lf, None).is_err());
        assert_eq!(fs::read(&p).unwrap(), b"previo");
    }

    #[test]
    fn save_text_con_estrategia_devuelve_informe() {
        let d = tempdir().unwrap();
        let p = d.path().join("s.txt");
        let s = save_text(
            &p,
            "10 €\n",
            Encoding::Ascii,
            LineEnding::Crlf,
            Some(LossStrategy::Transliterate),
        )
        .unwrap();
        assert_eq!(fs::read(&p).unwrap(), b"10 EUR\r\n");
        assert_eq!(s.bytes_written, 8);
        assert_eq!(s.losses, Some(loss_report("10 €\n", Encoding::Ascii)));
    }

    #[test]
    fn save_text_con_estrategia_sin_perdidas_no_informa() {
        let d = tempdir().unwrap();
        let p = d.path().join("s.txt");
        let s = save_text(
            &p,
            "hola",
            Encoding::Utf8,
            LineEnding::Lf,
            Some(LossStrategy::Transliterate),
        )
        .unwrap();
        assert_eq!(s.losses, None);
    }

    #[test]
    fn save_text_solo_lectura_y_carpeta() {
        let d = tempdir().unwrap();
        assert!(matches!(
            save_text(d.path(), "x", Encoding::Utf8, LineEnding::Lf, None),
            Err(FileError::IsDirectory { .. })
        ));
        let p = d.path().join("ro.txt");
        fs::write(&p, b"x").unwrap();
        set_readonly(&p, true);
        let r = save_text(&p, "y", Encoding::Utf8, LineEnding::Lf, None);
        set_readonly(&p, false);
        assert!(matches!(r, Err(FileError::ReadOnly { .. })));
    }

    #[test]
    fn ida_y_vuelta_byte_a_byte_en_todas_las_codificaciones() {
        let d = tempdir().unwrap();
        for enc in Encoding::ALL {
            for le in LineEnding::ALL {
                let original = write_text("uno\ndos\n", enc, le).unwrap();
                let p = d.path().join("rt.txt");
                fs::write(&p, &original).unwrap();
                let o = open_text(&p, Some(enc)).unwrap();
                save_text(&p, &o.file.text, o.file.encoding, o.file.line_ending, None).unwrap();
                assert_eq!(fs::read(&p).unwrap(), original, "{enc:?} {le:?}");
            }
        }
    }

    #[test]
    fn ida_y_vuelta_con_deteccion_automatica() {
        let d = tempdir().unwrap();
        let p = d.path().join("rt.md");
        let original = "# Título ñ\r\ntexto\r\n".as_bytes().to_vec();
        fs::write(&p, &original).unwrap();
        let o = open_text(&p, None).unwrap();
        save_text(&p, &o.file.text, o.file.encoding, o.file.line_ending, None).unwrap();
        assert_eq!(fs::read(&p).unwrap(), original);
    }

    // --- errores: Display y serde ---

    #[test]
    fn display_incluye_la_ruta() {
        let errores = [
            FileError::NotFound { path: "R".into() },
            FileError::PermissionDenied { path: "R".into() },
            FileError::ReadOnly { path: "R".into() },
            FileError::IsDirectory { path: "R".into() },
            FileError::Io {
                path: "R".into(),
                message: "m".into(),
            },
            FileError::Decode {
                path: "R".into(),
                encoding: Encoding::Ascii,
                offset: Some(1),
                message: "m".into(),
            },
            FileError::Unmappable {
                path: "R".into(),
                report: loss_report("€", Encoding::Ascii),
            },
        ];
        for e in errores {
            assert!(e.to_string().contains('R'), "{e:?}");
            let _: &dyn std::error::Error = &e;
        }
    }

    #[test]
    fn serializacion_de_errores() {
        let v = serde_json::to_value(FileError::NotFound { path: "p".into() }).unwrap();
        assert_eq!(v, serde_json::json!({"kind":"not-found","path":"p"}));
        let v = serde_json::to_value(FileError::IsDirectory { path: "p".into() }).unwrap();
        assert_eq!(v, serde_json::json!({"kind":"is-directory","path":"p"}));
        let v = serde_json::to_value(FileError::PermissionDenied { path: "p".into() }).unwrap();
        assert_eq!(v["kind"], "permission-denied");
        let v = serde_json::to_value(FileError::ReadOnly { path: "p".into() }).unwrap();
        assert_eq!(v["kind"], "read-only");
        let v = serde_json::to_value(FileError::Io {
            path: "p".into(),
            message: "m".into(),
        })
        .unwrap();
        assert_eq!(v, serde_json::json!({"kind":"io","path":"p","message":"m"}));
        let v = serde_json::to_value(FileError::Decode {
            path: "p".into(),
            encoding: Encoding::Ascii,
            offset: Some(3),
            message: "m".into(),
        })
        .unwrap();
        assert_eq!(
            v,
            serde_json::json!({"kind":"decode","path":"p","encoding":"ascii","offset":3,"message":"m"})
        );
        let v = serde_json::to_value(FileError::Unmappable {
            path: "p".into(),
            report: loss_report("€", Encoding::Ascii),
        })
        .unwrap();
        assert_eq!(v["kind"], "unmappable");
        assert_eq!(v["report"]["total"], 1);
    }

    #[test]
    fn serializacion_de_resultados() {
        let d = tempdir().unwrap();
        let p = d.path().join("a.txt");
        let s = save_text(&p, "hola\n", Encoding::Utf8, LineEnding::Lf, None).unwrap();
        assert_eq!(
            serde_json::to_value(&s).unwrap(),
            serde_json::json!({"bytesWritten":5,"losses":null})
        );
        let o = open_text(&p, None).unwrap();
        let v = serde_json::to_value(&o).unwrap();
        assert_eq!(v["readOnly"], false);
        assert_eq!(v["file"]["text"], "hola\n");
    }
}
