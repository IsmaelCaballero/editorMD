//! **M08 `FileService`** (Modelo · Rust) — acceso al disco: lectura, escritura
//! atómica y solo lectura, combinado con [`crate::model::text_codec`] (M09) para
//! abrir y guardar documentos de texto.
//!
//! - [`read_file`] / [`write_atomic`]: bytes ⇄ disco. La escritura va a un
//!   temporal de la misma carpeta y se renombra sobre el destino, de modo que
//!   un fallo nunca deja el fichero a medias.
//! - [`open_text`] / [`save_text`]: ficheros de texto completos, con ida y
//!   vuelta byte a byte y gestión de caracteres que no caben en la codificación.
//!
//! Los errores ([`FileError`]) se serializan para el frontend.

use std::fmt;
use std::path::Path;

use serde::Serialize;

use crate::model::text_codec::{Encoding, LineEnding, LossReport, LossStrategy, TextFile};

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
    /// Cualquier otro error de E/S.
    Io {
        /// Ruta recibida.
        path: String,
        /// `Display` del `std::io::Error`.
        message: String,
    },
    /// El contenido no es válido en la codificación usada.
    Decode {
        /// Ruta recibida.
        path: String,
        /// Codificación con la que se intentó decodificar.
        encoding: Encoding,
        /// Posición del primer byte no válido (`None` si la longitud UTF-16 es impar).
        offset: Option<usize>,
        /// `Display` del `DecodeError`.
        message: String,
    },
    /// Al guardar sin estrategia, hay caracteres que no caben en la codificación.
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
            FileError::Decode { path, message, .. } => {
                write!(f, "no se puede decodificar «{path}»: {message}")
            }
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
    /// Contenido completo.
    pub bytes: Vec<u8>,
    /// `true` si el fichero está marcado como de solo lectura.
    pub read_only: bool,
}

/// Documento de texto abierto, listo para el editor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenedFile {
    /// Texto y metadatos de codificación.
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
    /// Informe de pérdidas si se aplicó una [`LossStrategy`] y había caracteres que no caben.
    pub losses: Option<LossReport>,
}

/// Lee todo el contenido de un fichero.
///
/// # Errors
///
/// [`FileError::IsDirectory`] si es una carpeta, [`FileError::NotFound`],
/// [`FileError::PermissionDenied`] o [`FileError::Io`] según el error de E/S.
pub fn read_file(path: &Path) -> Result<FileBytes, FileError> {
    todo!("{path:?}")
}

/// Escribe `bytes` en `path` de forma atómica: temporal en la misma carpeta,
/// `sync_all` y renombrado sobre el destino. Si algo falla, el destino queda
/// intacto y no queda ningún temporal. No crea carpetas. En Unix conserva los
/// permisos del destino si ya existía.
///
/// # Errors
///
/// [`FileError::IsDirectory`] si la ruta es una carpeta, [`FileError::ReadOnly`]
/// si el destino existe y es de solo lectura, y [`FileError::NotFound`],
/// [`FileError::PermissionDenied`] o [`FileError::Io`] según el error de E/S.
///
/// # Examples
///
/// ```
/// use editormd_lib::model::file_service::{read_file, write_atomic};
///
/// let dir = std::env::temp_dir().join("editormd-doc-write-atomic");
/// std::fs::create_dir_all(&dir).unwrap();
/// let path = dir.join("nota.txt");
/// write_atomic(&path, b"hola").unwrap();
/// assert_eq!(read_file(&path).unwrap().bytes, b"hola");
/// std::fs::remove_dir_all(&dir).unwrap();
/// ```
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), FileError> {
    todo!("{path:?} {bytes:?}")
}

/// Abre un fichero de texto: lo lee y lo decodifica, detectando la codificación
/// si `encoding` es `None`.
///
/// # Errors
///
/// Los de [`read_file`] y [`FileError::Decode`] si el contenido no es válido.
///
/// # Examples
///
/// ```
/// use editormd_lib::model::file_service::{open_text, write_atomic};
/// use editormd_lib::model::text_codec::{Encoding, LineEnding};
///
/// let dir = std::env::temp_dir().join("editormd-doc-open-text");
/// std::fs::create_dir_all(&dir).unwrap();
/// let path = dir.join("nota.md");
/// write_atomic(&path, b"uno\r\ndos\r\n").unwrap();
/// let opened = open_text(&path, None).unwrap();
/// assert_eq!(opened.file.text, "uno\ndos\n");
/// assert_eq!(opened.file.encoding, Encoding::Utf8);
/// assert_eq!(opened.file.line_ending, LineEnding::Crlf);
/// assert!(!opened.read_only);
/// std::fs::remove_dir_all(&dir).unwrap();
/// ```
pub fn open_text(path: &Path, encoding: Option<Encoding>) -> Result<OpenedFile, FileError> {
    todo!("{path:?} {encoding:?}")
}

/// Guarda `text` en `path` con la codificación y el fin de línea indicados.
///
/// Los bytes se preparan antes de tocar el disco. Si hay caracteres que no
/// caben en `encoding` y no hay `strategy`, se devuelve error sin escribir nada.
///
/// # Errors
///
/// [`FileError::Unmappable`] (ver arriba) y los de [`write_atomic`].
pub fn save_text(
    path: &Path,
    text: &str,
    encoding: Encoding,
    line_ending: LineEnding,
    strategy: Option<LossStrategy>,
) -> Result<SavedFile, FileError> {
    todo!("{path:?} {text:?} {encoding:?} {line_ending:?} {strategy:?}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::text_codec::{DetectionMethod, loss_report, read_text};
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::{TempDir, tempdir};

    fn p(dir: &TempDir, name: &str) -> PathBuf {
        dir.path().join(name)
    }

    fn entries(dir: &TempDir) -> Vec<String> {
        let mut v: Vec<String> = fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        v.sort();
        v
    }

    fn set_readonly(path: &Path, value: bool) {
        let mut perms = fs::metadata(path).unwrap().permissions();
        #[allow(clippy::permissions_set_readonly_false)]
        perms.set_readonly(value);
        fs::set_permissions(path, perms).unwrap();
    }

    // ---- read_file ----

    #[test]
    fn read_file_returns_bytes_and_writable_flag() {
        let dir = tempdir().unwrap();
        let path = p(&dir, "a.txt");
        fs::write(&path, b"abc\xff").unwrap();
        let f = read_file(&path).unwrap();
        assert_eq!(f.bytes, b"abc\xff");
        assert!(!f.read_only);
    }

    #[test]
    fn read_file_empty_file() {
        let dir = tempdir().unwrap();
        let path = p(&dir, "vacio");
        fs::write(&path, b"").unwrap();
        assert_eq!(read_file(&path).unwrap().bytes, Vec::<u8>::new());
    }

    #[test]
    fn read_file_detects_read_only() {
        let dir = tempdir().unwrap();
        let path = p(&dir, "ro.txt");
        fs::write(&path, b"x").unwrap();
        set_readonly(&path, true);
        let result = read_file(&path);
        set_readonly(&path, false);
        let f = result.unwrap();
        assert_eq!(f.bytes, b"x");
        assert!(f.read_only);
    }

    #[test]
    fn read_file_not_found() {
        let dir = tempdir().unwrap();
        let path = p(&dir, "nada.txt");
        assert_eq!(
            read_file(&path),
            Err(FileError::NotFound {
                path: path.display().to_string()
            })
        );
    }

    #[test]
    fn read_file_directory() {
        let dir = tempdir().unwrap();
        assert_eq!(
            read_file(dir.path()),
            Err(FileError::IsDirectory {
                path: dir.path().display().to_string()
            })
        );
    }

    // ---- write_atomic ----

    #[test]
    fn write_atomic_creates_new_file_without_leftovers() {
        let dir = tempdir().unwrap();
        let path = p(&dir, "nuevo.txt");
        write_atomic(&path, b"hola").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"hola");
        assert_eq!(entries(&dir), ["nuevo.txt"]);
    }

    #[test]
    fn write_atomic_replaces_existing_file() {
        let dir = tempdir().unwrap();
        let path = p(&dir, "a.txt");
        fs::write(&path, b"viejo y largo").unwrap();
        fs::write(p(&dir, "otro.txt"), b"otro").unwrap();
        write_atomic(&path, b"nuevo").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"nuevo");
        assert_eq!(entries(&dir), ["a.txt", "otro.txt"]);
    }

    #[test]
    fn write_atomic_empty_content() {
        let dir = tempdir().unwrap();
        let path = p(&dir, "a.txt");
        fs::write(&path, b"algo").unwrap();
        write_atomic(&path, b"").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"");
    }

    #[test]
    fn write_atomic_repeated_writes_leave_no_temporaries() {
        let dir = tempdir().unwrap();
        let path = p(&dir, "a.txt");
        for i in 0..20u8 {
            write_atomic(&path, &[i; 100]).unwrap();
        }
        assert_eq!(fs::read(&path).unwrap(), [19u8; 100]);
        assert_eq!(entries(&dir), ["a.txt"]);
    }

    #[test]
    fn write_atomic_large_content() {
        let dir = tempdir().unwrap();
        let path = p(&dir, "grande.bin");
        let data: Vec<u8> = (0..3_000_000u32).map(|i| (i % 251) as u8).collect();
        write_atomic(&path, &data).unwrap();
        assert_eq!(fs::read(&path).unwrap(), data);
    }

    #[test]
    fn write_atomic_missing_folder_is_not_found_and_creates_nothing() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("no-existe").join("a.txt");
        assert_eq!(
            write_atomic(&path, b"x"),
            Err(FileError::NotFound {
                path: path.display().to_string()
            })
        );
        assert!(entries(&dir).is_empty());
    }

    #[test]
    fn write_atomic_directory_target() {
        let dir = tempdir().unwrap();
        let sub = p(&dir, "carpeta");
        fs::create_dir(&sub).unwrap();
        assert_eq!(
            write_atomic(&sub, b"x"),
            Err(FileError::IsDirectory {
                path: sub.display().to_string()
            })
        );
        assert_eq!(entries(&dir), ["carpeta"]);
    }

    #[test]
    fn write_atomic_read_only_target_is_untouched() {
        let dir = tempdir().unwrap();
        let path = p(&dir, "ro.txt");
        fs::write(&path, b"original").unwrap();
        set_readonly(&path, true);
        let result = write_atomic(&path, b"cambio");
        set_readonly(&path, false);
        assert_eq!(
            result,
            Err(FileError::ReadOnly {
                path: path.display().to_string()
            })
        );
        assert_eq!(fs::read(&path).unwrap(), b"original");
        assert_eq!(entries(&dir), ["ro.txt"]);
    }

    #[cfg(unix)]
    #[test]
    fn write_atomic_preserves_mode_of_existing_file() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempdir().unwrap();
        let path = p(&dir, "script.sh");
        fs::write(&path, b"#!/bin/sh").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o751)).unwrap();
        write_atomic(&path, b"#!/bin/sh\necho").unwrap();
        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o751);
    }

    // ---- open_text ----

    #[test]
    fn open_text_detects_encoding_and_line_ending() {
        let dir = tempdir().unwrap();
        let path = p(&dir, "a.md");
        fs::write(&path, b"\xef\xbb\xbfuno\r\ndos\r\n").unwrap();
        let o = open_text(&path, None).unwrap();
        assert_eq!(o.file, read_text(b"\xef\xbb\xbfuno\r\ndos\r\n").unwrap());
        assert_eq!(o.file.text, "uno\ndos\n");
        assert_eq!(o.file.encoding, Encoding::Utf8Bom);
        assert_eq!(o.file.line_ending, LineEnding::Crlf);
        assert_eq!(o.file.detection, Some(DetectionMethod::Bom));
        assert!(!o.read_only);
    }

    #[test]
    fn open_text_with_explicit_encoding() {
        let dir = tempdir().unwrap();
        let path = p(&dir, "a.txt");
        fs::write(&path, b"caf\xe9").unwrap();
        let o = open_text(&path, Some(Encoding::Iso8859_1)).unwrap();
        assert_eq!(o.file.text, "café");
        assert_eq!(o.file.encoding, Encoding::Iso8859_1);
        assert_eq!(o.file.detection, None);
    }

    #[test]
    fn open_text_read_only_flag() {
        let dir = tempdir().unwrap();
        let path = p(&dir, "ro.txt");
        fs::write(&path, b"hola").unwrap();
        set_readonly(&path, true);
        let result = open_text(&path, None);
        set_readonly(&path, false);
        assert!(result.unwrap().read_only);
    }

    #[test]
    fn open_text_decode_error_with_offset() {
        let dir = tempdir().unwrap();
        let path = p(&dir, "a.txt");
        fs::write(&path, b"abc\xe9").unwrap();
        let err = open_text(&path, Some(Encoding::Ascii)).unwrap_err();
        assert_eq!(
            err,
            FileError::Decode {
                path: path.display().to_string(),
                encoding: Encoding::Ascii,
                offset: Some(3),
                message: "secuencia no válida para la codificación ascii en el byte 3".into(),
            }
        );
    }

    #[test]
    fn open_text_decode_error_odd_length_has_no_offset() {
        let dir = tempdir().unwrap();
        let path = p(&dir, "a.txt");
        fs::write(&path, b"\xff\xfea\x00b").unwrap();
        match open_text(&path, None).unwrap_err() {
            FileError::Decode {
                encoding, offset, ..
            } => {
                assert_eq!(encoding, Encoding::Utf16Le);
                assert_eq!(offset, None);
            }
            other => panic!("error inesperado: {other:?}"),
        }
    }

    #[test]
    fn open_text_errors_from_read_file() {
        let dir = tempdir().unwrap();
        let missing = p(&dir, "nada");
        assert!(matches!(
            open_text(&missing, None),
            Err(FileError::NotFound { .. })
        ));
        assert!(matches!(
            open_text(dir.path(), None),
            Err(FileError::IsDirectory { .. })
        ));
    }

    // ---- save_text ----

    #[test]
    fn save_text_lossless_writes_encoded_bytes() {
        let dir = tempdir().unwrap();
        let path = p(&dir, "a.txt");
        let s = save_text(
            &path,
            "uno\ndos\n",
            Encoding::Utf8Bom,
            LineEnding::Crlf,
            None,
        )
        .unwrap();
        let expected = b"\xef\xbb\xbfuno\r\ndos\r\n";
        assert_eq!(fs::read(&path).unwrap(), expected);
        assert_eq!(s.bytes_written, expected.len());
        assert_eq!(s.losses, None);
    }

    #[test]
    fn save_text_ignores_strategy_when_lossless() {
        let dir = tempdir().unwrap();
        let path = p(&dir, "a.txt");
        let s = save_text(
            &path,
            "hola",
            Encoding::Ascii,
            LineEnding::Lf,
            Some(LossStrategy::Transliterate),
        )
        .unwrap();
        assert_eq!(s.losses, None);
        assert_eq!(fs::read(&path).unwrap(), b"hola");
    }

    #[test]
    fn save_text_unmappable_without_strategy_touches_nothing() {
        let dir = tempdir().unwrap();
        let path = p(&dir, "a.txt");
        fs::write(&path, b"previo").unwrap();
        let err = save_text(&path, "10 €", Encoding::Ascii, LineEnding::Lf, None).unwrap_err();
        assert_eq!(
            err,
            FileError::Unmappable {
                path: path.display().to_string(),
                report: loss_report("10 €", Encoding::Ascii),
            }
        );
        assert_eq!(fs::read(&path).unwrap(), b"previo");
        assert_eq!(entries(&dir), ["a.txt"]);
    }

    #[test]
    fn save_text_unmappable_does_not_create_new_file() {
        let dir = tempdir().unwrap();
        let path = p(&dir, "nuevo.txt");
        assert!(save_text(&path, "ñ", Encoding::Ascii, LineEnding::Lf, None).is_err());
        assert!(entries(&dir).is_empty());
    }

    #[test]
    fn save_text_with_strategy_reports_losses() {
        let dir = tempdir().unwrap();
        let path = p(&dir, "a.txt");
        let s = save_text(
            &path,
            "10 €\n",
            Encoding::Ascii,
            LineEnding::Crlf,
            Some(LossStrategy::Transliterate),
        )
        .unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"10 EUR\r\n");
        assert_eq!(s.bytes_written, 8);
        assert_eq!(s.losses, Some(loss_report("10 €\n", Encoding::Ascii)));
    }

    #[test]
    fn save_text_into_missing_folder() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("x").join("a.txt");
        assert!(matches!(
            save_text(&path, "a", Encoding::Utf8, LineEnding::Lf, None),
            Err(FileError::NotFound { .. })
        ));
    }

    #[test]
    fn save_text_read_only_target() {
        let dir = tempdir().unwrap();
        let path = p(&dir, "ro.txt");
        fs::write(&path, b"x").unwrap();
        set_readonly(&path, true);
        let result = save_text(&path, "y", Encoding::Utf8, LineEnding::Lf, None);
        set_readonly(&path, false);
        assert!(matches!(result, Err(FileError::ReadOnly { .. })));
        assert_eq!(fs::read(&path).unwrap(), b"x");
    }

    #[test]
    fn roundtrip_is_byte_identical_for_every_encoding() {
        let dir = tempdir().unwrap();
        let path = p(&dir, "a.txt");
        let samples: [(&[u8], &str); 4] = [
            (b"\xef\xbb\xbfcaf\xc3\xa9\r\nx\r\n", "bom"),
            (b"\xff\xfeh\x00\xe9\x00\n\x00", "utf16le"),
            (b"caf\xe9\r\nn\xf1", "latin"),
            (b"plain\nascii\n", "ascii"),
        ];
        for (bytes, label) in samples {
            fs::write(&path, bytes).unwrap();
            let o = open_text(&path, None).unwrap();
            save_text(&path, &o.file.text, o.file.encoding, o.file.line_ending, None).unwrap();
            assert_eq!(fs::read(&path).unwrap(), bytes, "{label}");
        }
    }

    // ---- serialización y mensajes ----

    #[test]
    fn file_error_serialization() {
        let e = FileError::NotFound { path: "a".into() };
        assert_eq!(
            serde_json::to_value(&e).unwrap(),
            json!({"kind": "not-found", "path": "a"})
        );
        let e = FileError::PermissionDenied { path: "a".into() };
        assert_eq!(
            serde_json::to_value(&e).unwrap(),
            json!({"kind": "permission-denied", "path": "a"})
        );
        let e = FileError::ReadOnly { path: "a".into() };
        assert_eq!(
            serde_json::to_value(&e).unwrap(),
            json!({"kind": "read-only", "path": "a"})
        );
        let e = FileError::IsDirectory { path: "a".into() };
        assert_eq!(
            serde_json::to_value(&e).unwrap(),
            json!({"kind": "is-directory", "path": "a"})
        );
        let e = FileError::Io {
            path: "a".into(),
            message: "m".into(),
        };
        assert_eq!(
            serde_json::to_value(&e).unwrap(),
            json!({"kind": "io", "path": "a", "message": "m"})
        );
        let e = FileError::Decode {
            path: "a".into(),
            encoding: Encoding::Ascii,
            offset: Some(3),
            message: "m".into(),
        };
        assert_eq!(
            serde_json::to_value(&e).unwrap(),
            json!({"kind": "decode", "path": "a", "encoding": "ascii", "offset": 3, "message": "m"})
        );
        let report = loss_report("€", Encoding::Ascii);
        let e = FileError::Unmappable {
            path: "a".into(),
            report: report.clone(),
        };
        assert_eq!(
            serde_json::to_value(&e).unwrap(),
            json!({"kind": "unmappable", "path": "a", "report": serde_json::to_value(&report).unwrap()})
        );
    }

    #[test]
    fn result_structs_serialization() {
        let s = SavedFile {
            bytes_written: 12,
            losses: None,
        };
        assert_eq!(
            serde_json::to_value(&s).unwrap(),
            json!({"bytesWritten": 12, "losses": null})
        );
        let file = read_text(b"hola").unwrap();
        let o = OpenedFile {
            file: file.clone(),
            read_only: false,
        };
        assert_eq!(
            serde_json::to_value(&o).unwrap(),
            json!({"file": serde_json::to_value(&file).unwrap(), "readOnly": false})
        );
    }

    #[test]
    fn display_messages_include_the_path() {
        let report = loss_report("€", Encoding::Ascii);
        let errors = [
            FileError::NotFound { path: "ruta".into() },
            FileError::PermissionDenied { path: "ruta".into() },
            FileError::ReadOnly { path: "ruta".into() },
            FileError::IsDirectory { path: "ruta".into() },
            FileError::Io {
                path: "ruta".into(),
                message: "fallo".into(),
            },
            FileError::Decode {
                path: "ruta".into(),
                encoding: Encoding::Utf8,
                offset: None,
                message: "m".into(),
            },
            FileError::Unmappable {
                path: "ruta".into(),
                report,
            },
        ];
        for e in errors {
            assert!(e.to_string().contains("ruta"), "{e}");
        }
        let _: &dyn std::error::Error = &FileError::NotFound { path: String::new() };
    }
}
