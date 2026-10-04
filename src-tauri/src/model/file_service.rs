//! **M08 `FileService`** (Modelo · Rust) — acceso al disco: lectura, escritura
//! atómica y modo solo lectura, combinado con **M09 `TextCodec`** para abrir y
//! guardar documentos de texto.
//!
//! - [`read_file`] / [`write_atomic`]: bytes ⇄ disco. La escritura es atómica:
//!   se escribe en un temporal de la misma carpeta y se renombra sobre el destino.
//! - [`open_text`] / [`save_text`]: documentos de texto con su codificación y
//!   fin de línea, con ida y vuelta byte a byte.
//! - [`FileError`]: error serializable para el frontend.

use std::fmt;
use std::path::Path;

use crate::model::text_codec::{Encoding, LineEnding, LossReport, LossStrategy, TextFile};

/// Error al abrir o guardar un fichero. Serializable para el frontend.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
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
        /// Posición del byte no válido (`None` en longitud impar de UTF-16).
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
        let _ = f;
        todo!()
    }
}

impl std::error::Error for FileError {}

/// Bytes de un fichero leído del disco.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileBytes {
    /// Contenido completo.
    pub bytes: Vec<u8>,
    /// El fichero está marcado como de solo lectura.
    pub read_only: bool,
}

/// Documento de texto abierto, listo para el editor.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenedFile {
    /// Texto, codificación y fin de línea.
    pub file: TextFile,
    /// El fichero está marcado como de solo lectura.
    pub read_only: bool,
}

/// Resultado de guardar.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedFile {
    /// Bytes escritos (BOM incluido).
    pub bytes_written: usize,
    /// Informe de pérdidas si se aplicó una estrategia y hubo caracteres que no caben.
    pub losses: Option<LossReport>,
}

fn map_io(path: &Path, err: &std::io::Error) -> FileError {
    let _ = (path, err);
    todo!()
}

fn write_via_temp(
    path: &Path,
    bytes: &[u8],
    rename: impl FnOnce(&Path, &Path) -> std::io::Result<()>,
) -> Result<(), FileError> {
    let _ = (path, bytes, rename);
    todo!()
}

/// Lee un fichero completo.
///
/// # Errors
///
/// [`FileError::IsDirectory`], [`FileError::NotFound`], [`FileError::PermissionDenied`]
/// o [`FileError::Io`].
pub fn read_file(path: &Path) -> Result<FileBytes, FileError> {
    let _ = path;
    todo!()
}

/// Escribe `bytes` en `path` de forma atómica.
///
/// # Errors
///
/// [`FileError::IsDirectory`], [`FileError::ReadOnly`], [`FileError::NotFound`],
/// [`FileError::PermissionDenied`] o [`FileError::Io`].
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), FileError> {
    let _ = (path, bytes);
    todo!()
}

/// Abre un fichero de texto.
///
/// # Errors
///
/// Los de [`read_file`] y [`FileError::Decode`].
pub fn open_text(path: &Path, encoding: Option<Encoding>) -> Result<OpenedFile, FileError> {
    let _ = (path, encoding);
    todo!()
}

/// Guarda un texto.
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
    let _ = (path, text, encoding, line_ending, strategy);
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::text_codec::{loss_report, read_text, write_text};
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn names(dir: &Path) -> Vec<String> {
        let mut v: Vec<String> = fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        v.sort();
        v
    }

    fn set_readonly(p: &Path, ro: bool) {
        let mut perms = fs::metadata(p).unwrap().permissions();
        perms.set_readonly(ro);
        fs::set_permissions(p, perms).unwrap();
    }

    fn p(dir: &Path, name: &str) -> PathBuf {
        dir.join(name)
    }

    // ---- read_file ----

    #[test]
    fn read_file_returns_bytes() {
        let d = tempdir().unwrap();
        let f = p(d.path(), "a.txt");
        fs::write(&f, b"hola\x00\xff").unwrap();
        let r = read_file(&f).unwrap();
        assert_eq!(r.bytes, b"hola\x00\xff");
        assert!(!r.read_only);
    }

    #[test]
    fn read_file_empty() {
        let d = tempdir().unwrap();
        let f = p(d.path(), "e.txt");
        fs::write(&f, b"").unwrap();
        assert_eq!(read_file(&f).unwrap().bytes, Vec::<u8>::new());
    }

    #[test]
    fn read_file_reports_read_only() {
        let d = tempdir().unwrap();
        let f = p(d.path(), "ro.txt");
        fs::write(&f, b"x").unwrap();
        set_readonly(&f, true);
        let r = read_file(&f);
        set_readonly(&f, false);
        assert!(r.unwrap().read_only);
    }

    #[test]
    fn read_file_not_found() {
        let d = tempdir().unwrap();
        let f = p(d.path(), "no.txt");
        assert_eq!(
            read_file(&f),
            Err(FileError::NotFound {
                path: f.display().to_string()
            })
        );
    }

    #[test]
    fn read_file_directory() {
        let d = tempdir().unwrap();
        assert_eq!(
            read_file(d.path()),
            Err(FileError::IsDirectory {
                path: d.path().display().to_string()
            })
        );
    }

    // ---- write_atomic ----

    #[test]
    fn write_atomic_creates_new_file_without_temporaries() {
        let d = tempdir().unwrap();
        let f = p(d.path(), "n.txt");
        write_atomic(&f, b"nuevo").unwrap();
        assert_eq!(fs::read(&f).unwrap(), b"nuevo");
        assert_eq!(names(d.path()), vec!["n.txt"]);
    }

    #[test]
    fn write_atomic_replaces_existing_and_keeps_neighbours() {
        let d = tempdir().unwrap();
        let f = p(d.path(), "n.txt");
        fs::write(&f, b"viejo y largo").unwrap();
        fs::write(p(d.path(), "otro.txt"), b"o").unwrap();
        write_atomic(&f, b"nuevo").unwrap();
        assert_eq!(fs::read(&f).unwrap(), b"nuevo");
        assert_eq!(names(d.path()), vec!["n.txt", "otro.txt"]);
    }

    #[test]
    fn write_atomic_empty_content() {
        let d = tempdir().unwrap();
        let f = p(d.path(), "v.txt");
        fs::write(&f, b"algo").unwrap();
        write_atomic(&f, b"").unwrap();
        assert_eq!(fs::read(&f).unwrap(), b"");
    }

    #[test]
    fn write_atomic_missing_folder_is_not_found_and_creates_nothing() {
        let d = tempdir().unwrap();
        let f = d.path().join("sub").join("n.txt");
        assert_eq!(
            write_atomic(&f, b"x"),
            Err(FileError::NotFound {
                path: f.display().to_string()
            })
        );
        assert!(names(d.path()).is_empty());
    }

    #[test]
    fn write_atomic_directory_target() {
        let d = tempdir().unwrap();
        let sub = p(d.path(), "carpeta");
        fs::create_dir(&sub).unwrap();
        assert_eq!(
            write_atomic(&sub, b"x"),
            Err(FileError::IsDirectory {
                path: sub.display().to_string()
            })
        );
        assert_eq!(names(d.path()), vec!["carpeta"]);
        assert!(names(&sub).is_empty());
    }

    #[test]
    fn write_atomic_read_only_target_is_untouched() {
        let d = tempdir().unwrap();
        let f = p(d.path(), "ro.txt");
        fs::write(&f, b"original").unwrap();
        set_readonly(&f, true);
        let r = write_atomic(&f, b"cambio");
        let content = fs::read(&f).unwrap();
        set_readonly(&f, false);
        assert_eq!(
            r,
            Err(FileError::ReadOnly {
                path: f.display().to_string()
            })
        );
        assert_eq!(content, b"original");
        assert_eq!(names(d.path()), vec!["ro.txt"]);
    }

    #[test]
    fn write_via_temp_cleans_up_when_rename_fails() {
        let d = tempdir().unwrap();
        let f = p(d.path(), "n.txt");
        fs::write(&f, b"intacto").unwrap();
        let r = write_via_temp(&f, b"nuevo", |_, _| {
            Err(std::io::Error::other("fallo simulado"))
        });
        assert_eq!(
            r,
            Err(FileError::Io {
                path: f.display().to_string(),
                message: "fallo simulado".into()
            })
        );
        assert_eq!(fs::read(&f).unwrap(), b"intacto");
        assert_eq!(names(d.path()), vec!["n.txt"]);
    }

    #[test]
    fn write_via_temp_failure_leaves_new_target_nonexistent() {
        let d = tempdir().unwrap();
        let f = p(d.path(), "n.txt");
        let r = write_via_temp(&f, b"nuevo", |_, _| {
            Err(std::io::Error::other("fallo simulado"))
        });
        assert!(r.is_err());
        assert!(names(d.path()).is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn write_atomic_preserves_mode_of_existing_file() {
        use std::os::unix::fs::PermissionsExt;
        let d = tempdir().unwrap();
        let f = p(d.path(), "x.sh");
        fs::write(&f, b"#!/bin/sh").unwrap();
        fs::set_permissions(&f, fs::Permissions::from_mode(0o751)).unwrap();
        write_atomic(&f, b"nuevo").unwrap();
        let mode = fs::metadata(&f).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o751);
    }

    // ---- open_text ----

    #[test]
    fn open_text_autodetects_utf8_and_crlf() {
        let d = tempdir().unwrap();
        let f = p(d.path(), "a.md");
        fs::write(&f, "uno\r\ndós\r\n").unwrap();
        let o = open_text(&f, None).unwrap();
        assert_eq!(o.file.text, "uno\ndós\n");
        assert_eq!(o.file.encoding, Encoding::Utf8);
        assert_eq!(o.file.line_ending, LineEnding::Crlf);
        assert!(!o.read_only);
    }

    #[test]
    fn open_text_with_explicit_encoding() {
        let d = tempdir().unwrap();
        let f = p(d.path(), "a.txt");
        fs::write(&f, [0x63, 0x61, 0x66, 0xE9]).unwrap();
        let o = open_text(&f, Some(Encoding::Iso8859_1)).unwrap();
        assert_eq!(o.file.text, "café");
        assert_eq!(o.file.encoding, Encoding::Iso8859_1);
    }

    #[test]
    fn open_text_read_only_flag() {
        let d = tempdir().unwrap();
        let f = p(d.path(), "ro.txt");
        fs::write(&f, b"x").unwrap();
        set_readonly(&f, true);
        let r = open_text(&f, None);
        set_readonly(&f, false);
        assert!(r.unwrap().read_only);
    }

    #[test]
    fn open_text_invalid_sequence_gives_decode_error() {
        let d = tempdir().unwrap();
        let f = p(d.path(), "a.txt");
        fs::write(&f, b"abc\xC3\x28").unwrap();
        match open_text(&f, Some(Encoding::Ascii)) {
            Err(FileError::Decode {
                path,
                encoding,
                offset,
                message,
            }) => {
                assert_eq!(path, f.display().to_string());
                assert_eq!(encoding, Encoding::Ascii);
                assert_eq!(offset, Some(3));
                assert!(message.contains('3'));
            }
            other => panic!("inesperado: {other:?}"),
        }
    }

    #[test]
    fn open_text_odd_utf16_has_no_offset() {
        let d = tempdir().unwrap();
        let f = p(d.path(), "a.txt");
        fs::write(&f, [0x61, 0x00, 0x62]).unwrap();
        match open_text(&f, Some(Encoding::Utf16Le)) {
            Err(FileError::Decode {
                encoding, offset, ..
            }) => {
                assert_eq!(encoding, Encoding::Utf16Le);
                assert_eq!(offset, None);
            }
            other => panic!("inesperado: {other:?}"),
        }
    }

    #[test]
    fn open_text_propagates_io_errors() {
        let d = tempdir().unwrap();
        let f = p(d.path(), "no.txt");
        assert!(matches!(
            open_text(&f, None),
            Err(FileError::NotFound { .. })
        ));
        assert!(matches!(
            open_text(d.path(), None),
            Err(FileError::IsDirectory { .. })
        ));
    }

    // ---- save_text ----

    #[test]
    fn save_text_lossless_writes_with_encoding_and_line_ending() {
        let d = tempdir().unwrap();
        let f = p(d.path(), "a.txt");
        let s = save_text(&f, "a\nb\n", Encoding::Utf8Bom, LineEnding::Crlf, None).unwrap();
        let expected = b"\xEF\xBB\xBFa\r\nb\r\n";
        assert_eq!(fs::read(&f).unwrap(), expected);
        assert_eq!(s.bytes_written, expected.len());
        assert_eq!(s.losses, None);
        assert_eq!(names(d.path()), vec!["a.txt"]);
    }

    #[test]
    fn save_text_unmappable_without_strategy_touches_nothing() {
        let d = tempdir().unwrap();
        let f = p(d.path(), "a.txt");
        let r = save_text(&f, "10 €\n", Encoding::Ascii, LineEnding::Lf, None);
        assert_eq!(
            r,
            Err(FileError::Unmappable {
                path: f.display().to_string(),
                report: loss_report("10 €\n", Encoding::Ascii)
            })
        );
        assert!(names(d.path()).is_empty());
    }

    #[test]
    fn save_text_unmappable_keeps_existing_content() {
        let d = tempdir().unwrap();
        let f = p(d.path(), "a.txt");
        fs::write(&f, b"previo").unwrap();
        assert!(save_text(&f, "€", Encoding::Ascii, LineEnding::Lf, None).is_err());
        assert_eq!(fs::read(&f).unwrap(), b"previo");
    }

    #[test]
    fn save_text_with_strategy_reports_losses() {
        let d = tempdir().unwrap();
        let f = p(d.path(), "a.txt");
        let s = save_text(
            &f,
            "10 €\n",
            Encoding::Ascii,
            LineEnding::Crlf,
            Some(LossStrategy::Transliterate),
        )
        .unwrap();
        assert_eq!(fs::read(&f).unwrap(), b"10 EUR\r\n");
        assert_eq!(s.bytes_written, 8);
        assert_eq!(s.losses, Some(loss_report("10 €\n", Encoding::Ascii)));
    }

    #[test]
    fn save_text_strategy_without_losses_gives_none() {
        let d = tempdir().unwrap();
        let f = p(d.path(), "a.txt");
        let s = save_text(
            &f,
            "hola\n",
            Encoding::Utf8,
            LineEnding::Lf,
            Some(LossStrategy::Transliterate),
        )
        .unwrap();
        assert_eq!(s.losses, None);
    }

    #[test]
    fn save_text_read_only_target() {
        let d = tempdir().unwrap();
        let f = p(d.path(), "ro.txt");
        fs::write(&f, b"x").unwrap();
        set_readonly(&f, true);
        let r = save_text(&f, "y", Encoding::Utf8, LineEnding::Lf, None);
        set_readonly(&f, false);
        assert!(matches!(r, Err(FileError::ReadOnly { .. })));
        assert_eq!(fs::read(&f).unwrap(), b"x");
    }

    #[test]
    fn round_trip_is_byte_identical_for_every_encoding_and_line_ending() {
        let d = tempdir().unwrap();
        for enc in Encoding::ALL {
            for le in LineEnding::ALL {
                let f = p(d.path(), "rt.txt");
                let bytes = write_text("hola\nmundo\n\nfin", enc, le).unwrap();
                fs::write(&f, &bytes).unwrap();
                let o = open_text(&f, Some(enc)).unwrap();
                save_text(&f, &o.file.text, o.file.encoding, o.file.line_ending, None).unwrap();
                assert_eq!(fs::read(&f).unwrap(), bytes, "{enc:?} {le:?}");
            }
        }
    }

    #[test]
    fn round_trip_with_autodetection() {
        let d = tempdir().unwrap();
        let f = p(d.path(), "rt.md");
        let bytes = "# Título\r\n\r\ncafé ñ\r\n".as_bytes().to_vec();
        fs::write(&f, &bytes).unwrap();
        let o = open_text(&f, None).unwrap();
        save_text(&f, &o.file.text, o.file.encoding, o.file.line_ending, None).unwrap();
        assert_eq!(fs::read(&f).unwrap(), bytes);
        assert_eq!(o.file, read_text(&bytes).unwrap());
    }

    // ---- errores: serde y Display ----

    #[test]
    fn file_error_serialization() {
        let j = |e: &FileError| serde_json::to_value(e).unwrap();
        assert_eq!(
            j(&FileError::NotFound { path: "a".into() }),
            serde_json::json!({"kind":"not-found","path":"a"})
        );
        assert_eq!(
            j(&FileError::PermissionDenied { path: "a".into() }),
            serde_json::json!({"kind":"permission-denied","path":"a"})
        );
        assert_eq!(
            j(&FileError::ReadOnly { path: "a".into() }),
            serde_json::json!({"kind":"read-only","path":"a"})
        );
        assert_eq!(
            j(&FileError::IsDirectory { path: "a".into() }),
            serde_json::json!({"kind":"is-directory","path":"a"})
        );
        assert_eq!(
            j(&FileError::Io {
                path: "a".into(),
                message: "m".into()
            }),
            serde_json::json!({"kind":"io","path":"a","message":"m"})
        );
        assert_eq!(
            j(&FileError::Decode {
                path: "a".into(),
                encoding: Encoding::Ascii,
                offset: Some(3),
                message: "m".into()
            }),
            serde_json::json!({"kind":"decode","path":"a","encoding":"ascii","offset":3,"message":"m"})
        );
        let report = loss_report("€", Encoding::Ascii);
        assert_eq!(
            j(&FileError::Unmappable {
                path: "a".into(),
                report: report.clone()
            }),
            serde_json::json!({"kind":"unmappable","path":"a","report":serde_json::to_value(&report).unwrap()})
        );
    }

    #[test]
    fn opened_and_saved_serialization() {
        let o = OpenedFile {
            file: read_text(b"hi").unwrap(),
            read_only: false,
        };
        let v = serde_json::to_value(&o).unwrap();
        assert_eq!(v["readOnly"], false);
        assert_eq!(v["file"]["text"], "hi");
        let s = SavedFile {
            bytes_written: 12,
            losses: None,
        };
        assert_eq!(
            serde_json::to_value(&s).unwrap(),
            serde_json::json!({"bytesWritten":12,"losses":null})
        );
    }

    #[test]
    fn display_is_spanish_and_includes_path() {
        let all = [
            FileError::NotFound {
                path: "/x/y".into(),
            },
            FileError::PermissionDenied {
                path: "/x/y".into(),
            },
            FileError::ReadOnly {
                path: "/x/y".into(),
            },
            FileError::IsDirectory {
                path: "/x/y".into(),
            },
            FileError::Io {
                path: "/x/y".into(),
                message: "boom".into(),
            },
            FileError::Decode {
                path: "/x/y".into(),
                encoding: Encoding::Ascii,
                offset: Some(1),
                message: "malo".into(),
            },
            FileError::Unmappable {
                path: "/x/y".into(),
                report: loss_report("€", Encoding::Ascii),
            },
        ];
        for e in all {
            let s = e.to_string();
            assert!(s.contains("/x/y"), "{s}");
        }
        let e: &dyn std::error::Error = &FileError::NotFound { path: "a".into() };
        assert!(e.source().is_none());
    }

    #[test]
    fn io_error_mapping() {
        use std::io::{Error, ErrorKind};
        let path = Path::new("r");
        assert!(matches!(
            map_io(path, &Error::from(ErrorKind::NotFound)),
            FileError::NotFound { .. }
        ));
        assert!(matches!(
            map_io(path, &Error::from(ErrorKind::PermissionDenied)),
            FileError::PermissionDenied { .. }
        ));
        assert_eq!(
            map_io(path, &Error::other("boom")),
            FileError::Io {
                path: "r".into(),
                message: "boom".into()
            }
        );
    }
}
