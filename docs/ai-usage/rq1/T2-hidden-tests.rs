//! Pruebas ocultas de aceptación de RQ.1 · T2 (M08 `FileService`).
//! El orquestador las copia en `src-tauri/tests/hidden.rs` de cada ejecución.
#![allow(clippy::pedantic)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use editormd_lib::model::file_service::{
    FileError, open_text, read_file, save_text, write_atomic,
};
use editormd_lib::model::text_codec::{
    DetectionMethod, Encoding, LineEnding, LossStrategy, decode, loss_report,
};

fn entries(dir: &Path) -> BTreeSet<String> {
    fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect()
}

fn set_read_only(path: &Path, ro: bool) {
    let mut p = fs::metadata(path).unwrap().permissions();
    p.set_readonly(ro);
    fs::set_permissions(path, p).unwrap();
}

fn disp(p: &Path) -> String {
    p.display().to_string()
}

fn setup(name: &str, bytes: &[u8]) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(name);
    fs::write(&path, bytes).unwrap();
    (dir, path)
}

// ---------- read_file ----------

#[test]
fn read_file_returns_bytes_and_writable() {
    let (_d, p) = setup("a.md", b"hola\r\n\xff");
    let f = read_file(&p).unwrap();
    assert_eq!(f.bytes, b"hola\r\n\xff");
    assert!(!f.read_only);
}

#[test]
fn read_file_empty() {
    let (_d, p) = setup("vacio.md", b"");
    assert!(read_file(&p).unwrap().bytes.is_empty());
}

#[test]
fn read_file_reports_read_only() {
    let (_d, p) = setup("ro.md", b"x");
    set_read_only(&p, true);
    let r = read_file(&p);
    set_read_only(&p, false);
    let f = r.unwrap();
    assert!(f.read_only);
    assert_eq!(f.bytes, b"x");
}

#[test]
fn read_file_not_found() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("no.md");
    assert_eq!(read_file(&p), Err(FileError::NotFound { path: disp(&p) }));
}

#[test]
fn read_file_directory() {
    let d = tempfile::tempdir().unwrap();
    assert_eq!(
        read_file(d.path()),
        Err(FileError::IsDirectory { path: disp(d.path()) })
    );
}

// ---------- write_atomic ----------

#[test]
fn write_atomic_creates_new_file_without_leftovers() {
    let d = tempfile::tempdir().unwrap();
    fs::write(d.path().join("otro.txt"), b"o").unwrap();
    let p = d.path().join("nuevo.md");
    write_atomic(&p, b"abc\n").unwrap();
    assert_eq!(fs::read(&p).unwrap(), b"abc\n");
    assert_eq!(
        entries(d.path()),
        BTreeSet::from(["nuevo.md".to_string(), "otro.txt".to_string()])
    );
}

#[test]
fn write_atomic_replaces_existing_content_exactly() {
    let (d, p) = setup("a.md", b"contenido largo anterior\n");
    write_atomic(&p, b"corto").unwrap();
    assert_eq!(fs::read(&p).unwrap(), b"corto");
    assert_eq!(entries(d.path()), BTreeSet::from(["a.md".to_string()]));
}

#[test]
fn write_atomic_empty_bytes() {
    let (_d, p) = setup("a.md", b"algo");
    write_atomic(&p, b"").unwrap();
    assert_eq!(fs::read(&p).unwrap(), b"");
}

#[test]
fn write_atomic_twice() {
    let (d, p) = setup("a.md", b"0");
    write_atomic(&p, b"1").unwrap();
    write_atomic(&p, b"22").unwrap();
    assert_eq!(fs::read(&p).unwrap(), b"22");
    assert_eq!(entries(d.path()).len(), 1);
}

#[test]
fn write_atomic_read_only_is_refused_and_untouched() {
    let (d, p) = setup("ro.md", b"original");
    set_read_only(&p, true);
    let r = write_atomic(&p, b"nuevo");
    set_read_only(&p, false);
    assert_eq!(r, Err(FileError::ReadOnly { path: disp(&p) }));
    assert_eq!(fs::read(&p).unwrap(), b"original");
    assert_eq!(entries(d.path()), BTreeSet::from(["ro.md".to_string()]));
}

#[test]
fn write_atomic_directory_target() {
    let d = tempfile::tempdir().unwrap();
    let sub = d.path().join("sub");
    fs::create_dir(&sub).unwrap();
    assert_eq!(
        write_atomic(&sub, b"x"),
        Err(FileError::IsDirectory { path: disp(&sub) })
    );
    assert!(sub.is_dir());
    assert!(entries(&sub).is_empty());
    assert_eq!(entries(d.path()), BTreeSet::from(["sub".to_string()]));
}

#[test]
fn write_atomic_missing_parent() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("no-existe").join("a.md");
    assert_eq!(
        write_atomic(&p, b"x"),
        Err(FileError::NotFound { path: disp(&p) })
    );
    assert!(entries(d.path()).is_empty());
}

#[cfg(unix)]
#[test]
fn write_atomic_preserves_unix_mode() {
    use std::os::unix::fs::PermissionsExt;
    let (_d, p) = setup("m.md", b"x");
    fs::set_permissions(&p, fs::Permissions::from_mode(0o640)).unwrap();
    write_atomic(&p, b"y").unwrap();
    assert_eq!(fs::metadata(&p).unwrap().permissions().mode() & 0o777, 0o640);
}

// ---------- open_text ----------

#[test]
fn open_text_auto_detects() {
    let (_d, p) = setup("a.md", b"\xef\xbb\xbfuno\r\ndos\r\n");
    let o = open_text(&p, None).unwrap();
    assert_eq!(o.file.text, "uno\ndos\n");
    assert_eq!(o.file.encoding, Encoding::Utf8Bom);
    assert_eq!(o.file.line_ending, LineEnding::Crlf);
    assert_eq!(o.file.detection, Some(DetectionMethod::Bom));
    assert!(!o.read_only);
}

#[test]
fn open_text_with_explicit_encoding() {
    let (_d, p) = setup("a.txt", b"caf\xe9\r");
    let o = open_text(&p, Some(Encoding::Windows1252)).unwrap();
    assert_eq!(o.file.text, "café\n");
    assert_eq!(o.file.encoding, Encoding::Windows1252);
    assert_eq!(o.file.line_ending, LineEnding::Cr);
    assert_eq!(o.file.detection, None);
}

#[test]
fn open_text_read_only_flag() {
    let (_d, p) = setup("ro.md", b"x");
    set_read_only(&p, true);
    let r = open_text(&p, None);
    set_read_only(&p, false);
    assert!(r.unwrap().read_only);
}

#[test]
fn open_text_decode_error_invalid_sequence() {
    let (_d, p) = setup("a.txt", b"ab\xe9");
    let message = decode(b"ab\xe9", Encoding::Ascii).unwrap_err().to_string();
    assert_eq!(
        open_text(&p, Some(Encoding::Ascii)),
        Err(FileError::Decode {
            path: disp(&p),
            encoding: Encoding::Ascii,
            offset: Some(2),
            message,
        })
    );
}

#[test]
fn open_text_decode_error_odd_length_auto() {
    let (_d, p) = setup("a.txt", b"\xff\xfeA");
    match open_text(&p, None) {
        Err(FileError::Decode { path, encoding, offset, message }) => {
            assert_eq!(path, disp(&p));
            assert_eq!(encoding, Encoding::Utf16Le);
            assert_eq!(offset, None);
            assert!(!message.is_empty());
        }
        other => panic!("se esperaba Decode, no {other:?}"),
    }
}

#[test]
fn open_text_not_found_and_directory() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("no.md");
    assert_eq!(open_text(&p, None), Err(FileError::NotFound { path: disp(&p) }));
    assert_eq!(
        open_text(d.path(), None),
        Err(FileError::IsDirectory { path: disp(d.path()) })
    );
}

// ---------- save_text ----------

#[test]
fn save_text_round_trip_byte_for_byte() {
    let cases: [&[u8]; 6] = [
        b"# T\xc3\xadtulo\n\ntexto\n",
        b"\xef\xbb\xbfuno\r\ndos\r\n",
        b"\xff\xfeh\x00o\x00\r\x00\n\x00l\x00a\x00",
        b"\xfe\xff\x00h\x00\n",
        b"caf\xe9\rcr\xe8me\r",
        b"sin salto final",
    ];
    for (i, bytes) in cases.iter().enumerate() {
        let (d, p) = setup(&format!("f{i}.md"), bytes);
        let o = open_text(&p, None).unwrap();
        let s = save_text(&p, &o.file.text, o.file.encoding, o.file.line_ending, None).unwrap();
        assert_eq!(fs::read(&p).unwrap(), *bytes, "caso {i}");
        assert_eq!(s.bytes_written, bytes.len(), "caso {i}");
        assert_eq!(s.losses, None, "caso {i}");
        assert_eq!(entries(d.path()).len(), 1, "caso {i}");
    }
}

#[test]
fn save_text_normalizes_line_endings_and_bom() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("a.md");
    let s = save_text(&p, "a\nb", Encoding::Utf8Bom, LineEnding::Crlf, None).unwrap();
    assert_eq!(fs::read(&p).unwrap(), b"\xef\xbb\xbfa\r\nb");
    assert_eq!(s.bytes_written, 7);
}

#[test]
fn save_text_unmappable_without_strategy_creates_nothing() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("a.txt");
    let text = "precio: 10 €\nfin ✓\n";
    assert_eq!(
        save_text(&p, text, Encoding::Iso8859_1, LineEnding::Lf, None),
        Err(FileError::Unmappable {
            path: disp(&p),
            report: loss_report(text, Encoding::Iso8859_1),
        })
    );
    assert!(entries(d.path()).is_empty());
}

#[test]
fn save_text_unmappable_without_strategy_keeps_existing() {
    let (d, p) = setup("a.txt", b"antes");
    let r = save_text(&p, "€", Encoding::Ascii, LineEnding::Lf, None);
    assert!(matches!(r, Err(FileError::Unmappable { .. })));
    assert_eq!(fs::read(&p).unwrap(), b"antes");
    assert_eq!(entries(d.path()).len(), 1);
}

#[test]
fn save_text_with_strategy_reports_losses() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("a.txt");
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
fn save_text_with_strategy_but_lossless() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("a.txt");
    let s = save_text(
        &p,
        "café\n",
        Encoding::Windows1252,
        LineEnding::Lf,
        Some(LossStrategy::Transliterate),
    )
    .unwrap();
    assert_eq!(fs::read(&p).unwrap(), b"caf\xe9\n");
    assert_eq!(s.losses, None);
}

#[test]
fn save_text_read_only_directory_and_missing_parent() {
    let (d, p) = setup("ro.md", b"original");
    set_read_only(&p, true);
    let r = save_text(&p, "nuevo", Encoding::Utf8, LineEnding::Lf, None);
    set_read_only(&p, false);
    assert_eq!(r, Err(FileError::ReadOnly { path: disp(&p) }));
    assert_eq!(fs::read(&p).unwrap(), b"original");

    assert_eq!(
        save_text(d.path(), "x", Encoding::Utf8, LineEnding::Lf, None),
        Err(FileError::IsDirectory { path: disp(d.path()) })
    );
    let q = d.path().join("no").join("a.md");
    assert_eq!(
        save_text(&q, "x", Encoding::Utf8, LineEnding::Lf, None),
        Err(FileError::NotFound { path: disp(&q) })
    );
    assert_eq!(entries(d.path()), BTreeSet::from(["ro.md".to_string()]));
}

// ---------- errores y serialización ----------

#[test]
fn file_error_display_and_error_trait() {
    let errs = [
        FileError::NotFound { path: "C:/x/a.md".into() },
        FileError::PermissionDenied { path: "C:/x/a.md".into() },
        FileError::ReadOnly { path: "C:/x/a.md".into() },
        FileError::IsDirectory { path: "C:/x/a.md".into() },
        FileError::Io { path: "C:/x/a.md".into(), message: "disco lleno".into() },
        FileError::Decode {
            path: "C:/x/a.md".into(),
            encoding: Encoding::Ascii,
            offset: Some(2),
            message: "m".into(),
        },
        FileError::Unmappable {
            path: "C:/x/a.md".into(),
            report: loss_report("€", Encoding::Ascii),
        },
    ];
    for e in &errs {
        let s = e.to_string();
        assert!(s.contains("C:/x/a.md"), "{s}");
        let _: &dyn std::error::Error = e;
    }
}

#[test]
fn file_error_serialization() {
    use serde_json::{json, to_value};
    assert_eq!(
        to_value(FileError::NotFound { path: "p".into() }).unwrap(),
        json!({"kind": "not-found", "path": "p"})
    );
    assert_eq!(
        to_value(FileError::PermissionDenied { path: "p".into() }).unwrap(),
        json!({"kind": "permission-denied", "path": "p"})
    );
    assert_eq!(
        to_value(FileError::ReadOnly { path: "p".into() }).unwrap(),
        json!({"kind": "read-only", "path": "p"})
    );
    assert_eq!(
        to_value(FileError::IsDirectory { path: "p".into() }).unwrap(),
        json!({"kind": "is-directory", "path": "p"})
    );
    assert_eq!(
        to_value(FileError::Io { path: "p".into(), message: "m".into() }).unwrap(),
        json!({"kind": "io", "path": "p", "message": "m"})
    );
    assert_eq!(
        to_value(FileError::Decode {
            path: "p".into(),
            encoding: Encoding::Utf16Le,
            offset: None,
            message: "m".into()
        })
        .unwrap(),
        json!({"kind": "decode", "path": "p", "encoding": "utf-16le", "offset": null, "message": "m"})
    );
    let report = loss_report("€", Encoding::Ascii);
    assert_eq!(
        to_value(FileError::Unmappable { path: "p".into(), report: report.clone() }).unwrap(),
        json!({"kind": "unmappable", "path": "p", "report": to_value(&report).unwrap()})
    );
}

#[test]
fn opened_and_saved_serialization() {
    use serde_json::{json, to_value};
    let (_d, p) = setup("a.md", b"hola\n");
    let o = open_text(&p, None).unwrap();
    let v = to_value(&o).unwrap();
    assert_eq!(v["readOnly"], json!(false));
    assert_eq!(v["file"], to_value(&o.file).unwrap());
    assert_eq!(v.as_object().unwrap().len(), 2);

    let s = save_text(&p, "hola\n", Encoding::Utf8, LineEnding::Lf, None).unwrap();
    assert_eq!(to_value(&s).unwrap(), json!({"bytesWritten": 5, "losses": null}));
}
