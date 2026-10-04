//! Pruebas unitarias de **M08 `FileService`**. Todas trabajan en carpetas
//! temporales (`tempfile::tempdir()`), nunca en el árbol del repositorio.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::json;
use tempfile::{TempDir, tempdir};

use super::*;
use crate::model::text_codec::{DetectionMethod, loss_report, read_text_as};

// ---------------------------------------------------------------- utilidades

/// Nombres de las entradas de `dir`, ordenados.
fn entries(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// Carpeta temporal con un fichero `name` que contiene `bytes`.
fn dir_with(name: &str, bytes: &[u8]) -> (TempDir, PathBuf) {
    let dir = tempdir().unwrap();
    let path = dir.path().join(name);
    fs::write(&path, bytes).unwrap();
    (dir, path)
}

fn shown(path: &Path) -> String {
    path.display().to_string()
}

fn set_readonly(path: &Path, readonly: bool) {
    let mut perms = fs::metadata(path).unwrap().permissions();
    perms.set_readonly(readonly);
    fs::set_permissions(path, perms).unwrap();
}

/// Marca un fichero como de solo lectura y lo restaura al salir de ámbito
/// (también si la prueba falla), para que la carpeta temporal se pueda borrar.
struct ReadOnlyGuard(PathBuf);

impl ReadOnlyGuard {
    fn new(path: &Path) -> Self {
        set_readonly(path, true);
        ReadOnlyGuard(path.to_path_buf())
    }
}

impl Drop for ReadOnlyGuard {
    fn drop(&mut self) {
        let readonly = false;
        set_readonly(&self.0, readonly);
    }
}

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/encodings")
}

// ---------------------------------------------------------------- read_file

#[test]
fn read_file_returns_the_bytes() {
    let (_dir, path) = dir_with("a.md", b"hola\r\n\xff\x00");
    let read = read_file(&path).unwrap();
    assert_eq!(
        read,
        FileBytes {
            bytes: b"hola\r\n\xff\x00".to_vec(),
            read_only: false
        }
    );
}

#[test]
fn read_file_reads_an_empty_file() {
    let (_dir, path) = dir_with("vacio.txt", b"");
    assert_eq!(read_file(&path).unwrap().bytes, Vec::<u8>::new());
}

#[test]
fn read_file_reports_read_only() {
    let (_dir, path) = dir_with("ro.md", b"x");
    let _guard = ReadOnlyGuard::new(&path);
    let read = read_file(&path).unwrap();
    assert!(read.read_only);
    assert_eq!(read.bytes, b"x");
}

#[test]
fn read_file_not_found() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("no-existe.md");
    assert_eq!(
        read_file(&path),
        Err(FileError::NotFound { path: shown(&path) })
    );
}

#[test]
fn read_file_not_found_when_parent_is_missing() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("falta").join("a.md");
    assert_eq!(
        read_file(&path),
        Err(FileError::NotFound { path: shown(&path) })
    );
}

#[test]
fn read_file_on_a_directory() {
    let dir = tempdir().unwrap();
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

    let (_dir, path) = dir_with("secreto.md", b"x");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
    let result = read_file(&path);
    let can_bypass = fs::read(&path).is_ok(); // root ignora los permisos
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    if !can_bypass {
        assert_eq!(
            result,
            Err(FileError::PermissionDenied { path: shown(&path) })
        );
    }
}

// ---------------------------------------------------------------- io errors

#[test]
fn io_errors_are_mapped_by_kind() {
    let path = Path::new("x/y.md");
    let p = shown(path);
    assert_eq!(
        map_io(path, &io::Error::from(io::ErrorKind::NotFound)),
        FileError::NotFound { path: p.clone() }
    );
    assert_eq!(
        map_io(path, &io::Error::from(io::ErrorKind::PermissionDenied)),
        FileError::PermissionDenied { path: p.clone() }
    );
    let other = io::Error::other("disco lleno");
    assert_eq!(
        map_io(path, &other),
        FileError::Io {
            path: p,
            message: other.to_string()
        }
    );
}

// ---------------------------------------------------------------- write_atomic

#[test]
fn write_atomic_creates_a_new_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("nuevo.md");
    write_atomic(&path, b"contenido").unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"contenido");
    assert_eq!(entries(dir.path()), vec!["nuevo.md"]);
}

#[test]
fn write_atomic_replaces_an_existing_file() {
    let (dir, path) = dir_with("a.md", b"viejo contenido largo");
    fs::write(dir.path().join("otro.txt"), b"y").unwrap();
    write_atomic(&path, b"nuevo").unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"nuevo");
    assert_eq!(entries(dir.path()), vec!["a.md", "otro.txt"]);
}

#[test]
fn write_atomic_writes_empty_content() {
    let (dir, path) = dir_with("a.md", b"algo");
    write_atomic(&path, b"").unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"");
    assert_eq!(entries(dir.path()), vec!["a.md"]);
}

#[test]
fn write_atomic_twice_leaves_no_temporaries() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("a.md");
    write_atomic(&path, b"uno").unwrap();
    write_atomic(&path, b"dos").unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"dos");
    assert_eq!(entries(dir.path()), vec!["a.md"]);
}

#[test]
fn write_atomic_on_a_directory() {
    let dir = tempdir().unwrap();
    let sub = dir.path().join("carpeta");
    fs::create_dir(&sub).unwrap();
    assert_eq!(
        write_atomic(&sub, b"x"),
        Err(FileError::IsDirectory { path: shown(&sub) })
    );
    assert!(sub.is_dir());
    assert_eq!(entries(dir.path()), vec!["carpeta"]);
    assert_eq!(entries(&sub), Vec::<String>::new());
}

#[test]
fn write_atomic_refuses_read_only_files() {
    let (dir, path) = dir_with("ro.md", b"original");
    let _guard = ReadOnlyGuard::new(&path);
    assert_eq!(
        write_atomic(&path, b"cambio"),
        Err(FileError::ReadOnly { path: shown(&path) })
    );
    assert_eq!(fs::read(&path).unwrap(), b"original");
    assert_eq!(entries(dir.path()), vec!["ro.md"]);
}

#[test]
fn write_atomic_missing_parent_is_not_found() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("falta").join("a.md");
    assert_eq!(
        write_atomic(&path, b"x"),
        Err(FileError::NotFound { path: shown(&path) })
    );
    assert_eq!(entries(dir.path()), Vec::<String>::new());
}

#[test]
fn write_atomic_failed_rename_keeps_the_old_file_and_no_temporary() {
    let (dir, path) = dir_with("a.md", b"original");
    let mut seen_tmp = None;
    let result = write_atomic_with(&path, b"nuevo", |tmp, dest| {
        assert_eq!(dest, path.as_path());
        assert_eq!(tmp.parent(), Some(dir.path()));
        assert_eq!(fs::read(tmp).unwrap(), b"nuevo");
        seen_tmp = Some(tmp.to_path_buf());
        Err(io::Error::other("fallo simulado"))
    });
    assert_eq!(
        result,
        Err(FileError::Io {
            path: shown(&path),
            message: "fallo simulado".to_string()
        })
    );
    assert!(seen_tmp.is_some_and(|t| !t.exists()));
    assert_eq!(fs::read(&path).unwrap(), b"original");
    assert_eq!(entries(dir.path()), vec!["a.md"]);
}

#[test]
fn write_atomic_failed_rename_of_a_new_file_leaves_nothing() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("nuevo.md");
    let result = write_atomic_with(&path, b"x", |_, _| {
        Err(io::Error::from(io::ErrorKind::PermissionDenied))
    });
    assert_eq!(
        result,
        Err(FileError::PermissionDenied { path: shown(&path) })
    );
    assert_eq!(entries(dir.path()), Vec::<String>::new());
}

#[test]
fn write_atomic_temporary_names_do_not_clash_with_existing_files() {
    let (dir, path) = dir_with("a.md", b"original");
    // Ficheros que podrían parecer temporales de otra ejecución: no se tocan.
    let mut before = vec!["a.md".to_string()];
    for name in [".a.md.tmp", ".a.md.0.tmp", "a.md.tmp"] {
        fs::write(dir.path().join(name), b"ajeno").unwrap();
        before.push(name.to_string());
    }
    before.sort();
    write_atomic(&path, b"nuevo").unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"nuevo");
    assert_eq!(entries(dir.path()), before);
    for name in [".a.md.tmp", ".a.md.0.tmp", "a.md.tmp"] {
        assert_eq!(fs::read(dir.path().join(name)).unwrap(), b"ajeno");
    }
}

#[test]
fn parent_dir_of_a_bare_file_name_is_the_current_directory() {
    assert_eq!(parent_dir(Path::new("a.md")), Path::new("."));
    assert_eq!(parent_dir(Path::new("x/a.md")), Path::new("x"));
}

#[cfg(unix)]
#[test]
fn write_atomic_keeps_the_mode_of_an_existing_file() {
    use std::os::unix::fs::PermissionsExt;

    let (_dir, path) = dir_with("script.sh", b"echo 1\n");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o750)).unwrap();
    write_atomic(&path, b"echo 2\n").unwrap();
    let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o750);
    assert_eq!(fs::read(&path).unwrap(), b"echo 2\n");
}

#[cfg(unix)]
#[test]
fn write_atomic_in_a_read_only_directory_is_permission_denied() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempdir().unwrap();
    let sub = dir.path().join("cerrada");
    fs::create_dir(&sub).unwrap();
    fs::set_permissions(&sub, fs::Permissions::from_mode(0o555)).unwrap();
    let path = sub.join("a.md");
    let result = write_atomic(&path, b"x");
    let can_bypass = fs::write(sub.join("sonda"), b"").is_ok(); // root
    fs::set_permissions(&sub, fs::Permissions::from_mode(0o755)).unwrap();
    if !can_bypass {
        assert_eq!(
            result,
            Err(FileError::PermissionDenied { path: shown(&path) })
        );
        assert_eq!(entries(&sub), Vec::<String>::new());
    }
}

// ---------------------------------------------------------------- open_text

#[test]
fn open_text_detects_encoding_and_line_ending() {
    let (_dir, path) = dir_with("a.md", b"\xef\xbb\xbfuno\r\ndos\r\n");
    let opened = open_text(&path, None).unwrap();
    assert!(!opened.read_only);
    assert_eq!(opened.file.text, "uno\ndos\n");
    assert_eq!(opened.file.encoding, Encoding::Utf8Bom);
    assert_eq!(opened.file.line_ending, LineEnding::Crlf);
    assert_eq!(opened.file.detection, Some(DetectionMethod::Bom));
}

#[test]
fn open_text_with_an_explicit_encoding() {
    let (_dir, path) = dir_with("a.txt", b"A\xf1o\n");
    let opened = open_text(&path, Some(Encoding::Windows1252)).unwrap();
    assert_eq!(opened.file.text, "Año\n");
    assert_eq!(opened.file.encoding, Encoding::Windows1252);
    assert_eq!(opened.file.detection, None);
}

#[test]
fn open_text_reports_read_only() {
    let (_dir, path) = dir_with("ro.md", b"hola");
    let _guard = ReadOnlyGuard::new(&path);
    let opened = open_text(&path, None).unwrap();
    assert!(opened.read_only);
    assert_eq!(opened.file.text, "hola");
}

#[test]
fn open_text_invalid_sequence_is_a_decode_error() {
    let bytes = b"abc\xffdef";
    let (_dir, path) = dir_with("a.txt", bytes);
    let expected = read_text_as(bytes, Encoding::Ascii).unwrap_err();
    assert_eq!(
        open_text(&path, Some(Encoding::Ascii)),
        Err(FileError::Decode {
            path: shown(&path),
            encoding: Encoding::Ascii,
            offset: Some(3),
            message: expected.to_string()
        })
    );
}

#[test]
fn open_text_odd_utf16_is_a_decode_error_without_offset() {
    let bytes = b"\xff\xfea\x00b";
    let (_dir, path) = dir_with("a.txt", bytes);
    let expected = read_text_as(bytes, Encoding::Utf16Le).unwrap_err();
    assert_eq!(
        open_text(&path, Some(Encoding::Utf16Le)),
        Err(FileError::Decode {
            path: shown(&path),
            encoding: Encoding::Utf16Le,
            offset: None,
            message: expected.to_string()
        })
    );
}

#[test]
fn open_text_auto_detection_error_after_bom() {
    let (_dir, path) = dir_with("a.txt", b"\xef\xbb\xbfab\xff");
    match open_text(&path, None) {
        Err(FileError::Decode {
            path: p,
            encoding,
            offset,
            ..
        }) => {
            assert_eq!(p, shown(&path));
            assert_eq!(encoding, Encoding::Utf8Bom);
            assert_eq!(offset, Some(5));
        }
        other => panic!("se esperaba Decode, no {other:?}"),
    }
}

#[test]
fn open_text_propagates_file_errors() {
    let dir = tempdir().unwrap();
    let missing = dir.path().join("no.md");
    assert_eq!(
        open_text(&missing, None),
        Err(FileError::NotFound {
            path: shown(&missing)
        })
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
    let dir = tempdir().unwrap();
    let path = dir.path().join("a.md");
    let saved = save_text(
        &path,
        "uno\ndos\n",
        Encoding::Utf8Bom,
        LineEnding::Crlf,
        None,
    )
    .unwrap();
    assert_eq!(
        saved,
        SavedFile {
            bytes_written: 13,
            losses: None
        }
    );
    assert_eq!(fs::read(&path).unwrap(), b"\xef\xbb\xbfuno\r\ndos\r\n");
    assert_eq!(entries(dir.path()), vec!["a.md"]);
}

#[test]
fn save_text_lossless_ignores_the_strategy() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("a.txt");
    let saved = save_text(
        &path,
        "Año\n",
        Encoding::Iso8859_1,
        LineEnding::Lf,
        Some(LossStrategy::Replace),
    )
    .unwrap();
    assert_eq!(saved.losses, None);
    assert_eq!(saved.bytes_written, 4);
    assert_eq!(fs::read(&path).unwrap(), b"A\xf1o\n");
}

#[test]
fn save_text_unmappable_without_strategy_does_not_touch_the_disk() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("a.txt");
    let text = "10 € y 5 €\n";
    assert_eq!(
        save_text(&path, text, Encoding::Ascii, LineEnding::Lf, None),
        Err(FileError::Unmappable {
            path: shown(&path),
            report: loss_report(text, Encoding::Ascii)
        })
    );
    assert_eq!(entries(dir.path()), Vec::<String>::new());
}

#[test]
fn save_text_unmappable_keeps_an_existing_file() {
    let (dir, path) = dir_with("a.txt", b"original");
    let result = save_text(&path, "€", Encoding::Iso8859_1, LineEnding::Lf, None);
    assert!(matches!(result, Err(FileError::Unmappable { .. })));
    assert_eq!(fs::read(&path).unwrap(), b"original");
    assert_eq!(entries(dir.path()), vec!["a.txt"]);
}

#[test]
fn save_text_with_strategy_reports_the_losses() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("a.txt");
    let text = "10 €\n";
    let saved = save_text(
        &path,
        text,
        Encoding::Ascii,
        LineEnding::Crlf,
        Some(LossStrategy::Transliterate),
    )
    .unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"10 EUR\r\n");
    assert_eq!(
        saved,
        SavedFile {
            bytes_written: 8,
            losses: Some(loss_report(text, Encoding::Ascii))
        }
    );
}

#[test]
fn save_text_with_replace_strategy() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("a.txt");
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
    assert!(saved.losses.is_some_and(|r| r.total == 1));
}

#[test]
fn save_text_propagates_write_errors() {
    let (dir, path) = dir_with("ro.md", b"original");
    let _guard = ReadOnlyGuard::new(&path);
    assert_eq!(
        save_text(&path, "x", Encoding::Utf8, LineEnding::Lf, None),
        Err(FileError::ReadOnly { path: shown(&path) })
    );
    assert_eq!(
        save_text(dir.path(), "x", Encoding::Utf8, LineEnding::Lf, None),
        Err(FileError::IsDirectory {
            path: shown(dir.path())
        })
    );
    assert_eq!(fs::read(&path).unwrap(), b"original");
}

#[test]
fn save_text_utf16_counts_the_bom() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("a.txt");
    let saved = save_text(&path, "ab", Encoding::Utf16Be, LineEnding::Lf, None).unwrap();
    assert_eq!(saved.bytes_written, 6);
    assert_eq!(fs::read(&path).unwrap(), b"\xfe\xff\x00a\x00b");
}

#[test]
fn round_trip_of_every_fixture_is_byte_identical() {
    let dir = tempdir().unwrap();
    let mut checked = 0;
    for entry in fs::read_dir(fixtures()).unwrap() {
        let source = entry.unwrap().path();
        let original = fs::read(&source).unwrap();
        let path = dir.path().join(source.file_name().unwrap());
        fs::write(&path, &original).unwrap();

        let opened = open_text(&path, None).unwrap();
        if opened.file.mixed_line_endings {
            continue;
        }
        let f = opened.file;
        let saved = save_text(&path, &f.text, f.encoding, f.line_ending, None).unwrap();
        assert_eq!(saved.bytes_written, original.len(), "{}", source.display());
        assert_eq!(fs::read(&path).unwrap(), original, "{}", source.display());
        checked += 1;
    }
    assert!(checked > 10);
}

#[test]
fn round_trip_with_explicit_encoding() {
    let original = b"# T\xedtulo\r\rcaf\xe9\r";
    let (_dir, path) = dir_with("mac.txt", original);
    let f = open_text(&path, Some(Encoding::Iso8859_1)).unwrap().file;
    assert_eq!(f.line_ending, LineEnding::Cr);
    save_text(&path, &f.text, f.encoding, f.line_ending, None).unwrap();
    assert_eq!(fs::read(&path).unwrap(), original);
}

#[test]
fn round_trip_normalizes_mixed_line_endings() {
    let (_dir, path) = dir_with("mixto.md", b"a\r\nb\r\nc\n");
    let f = open_text(&path, None).unwrap().file;
    assert!(f.mixed_line_endings);
    save_text(&path, &f.text, f.encoding, f.line_ending, None).unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"a\r\nb\r\nc\r\n");
}

// ---------------------------------------------------------------- serde y Display

#[test]
fn file_error_serializes_with_a_kind_tag() {
    let p = "c:/x/a.md".to_string();
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
                message: "m".into(),
            },
            json!({"kind": "io", "path": p, "message": "m"}),
        ),
        (
            FileError::Decode {
                path: p.clone(),
                encoding: Encoding::Ascii,
                offset: Some(3),
                message: "m".into(),
            },
            json!({"kind": "decode", "path": p, "encoding": "ascii", "offset": 3, "message": "m"}),
        ),
        (
            FileError::Decode {
                path: p.clone(),
                encoding: Encoding::Utf16Le,
                offset: None,
                message: "m".into(),
            },
            json!({"kind": "decode", "path": p, "encoding": "utf-16le", "offset": null, "message": "m"}),
        ),
    ];
    for (error, expected) in cases {
        assert_eq!(serde_json::to_value(&error).unwrap(), expected);
    }
}

#[test]
fn unmappable_serializes_its_report() {
    let report = loss_report("€", Encoding::Ascii);
    let error = FileError::Unmappable {
        path: "a".into(),
        report: report.clone(),
    };
    assert_eq!(
        serde_json::to_value(&error).unwrap(),
        json!({"kind": "unmappable", "path": "a", "report": serde_json::to_value(&report).unwrap()})
    );
}

#[test]
fn opened_and_saved_files_serialize_in_camel_case() {
    let (_dir, path) = dir_with("a.md", b"hola\n");
    let opened = open_text(&path, None).unwrap();
    assert_eq!(
        serde_json::to_value(&opened).unwrap(),
        json!({"file": serde_json::to_value(&opened.file).unwrap(), "readOnly": false})
    );
    let saved = SavedFile {
        bytes_written: 12,
        losses: None,
    };
    assert_eq!(
        serde_json::to_value(&saved).unwrap(),
        json!({"bytesWritten": 12, "losses": null})
    );
}

#[test]
fn display_mentions_the_path_in_every_variant() {
    let p = "/tmp/ruta-única.md".to_string();
    let errors = [
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
            offset: Some(3),
            message: "secuencia no válida".into(),
        },
        FileError::Unmappable {
            path: p.clone(),
            report: loss_report("€ €", Encoding::Ascii),
        },
    ];
    let mut messages = Vec::new();
    for error in &errors {
        let message = error.to_string();
        assert!(message.contains(&p), "{message}");
        messages.push(message);
    }
    assert!(messages[4].contains("disco lleno"));
    assert!(messages[5].contains("secuencia no válida"));
    assert!(messages[6].contains("ascii"));
    messages.dedup();
    assert_eq!(
        messages.len(),
        errors.len(),
        "mensajes distintos por variante"
    );
}

#[test]
fn file_error_is_a_std_error() {
    let error: Box<dyn std::error::Error> = Box::new(FileError::NotFound { path: "a".into() });
    assert!(error.to_string().contains('a'));
}
