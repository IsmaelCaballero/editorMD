//! Lectura y escritura de ficheros de texto completos (parte de **M09
//! `TextCodec`**): combina codificación y fin de línea para garantizar la
//! **ida y vuelta byte a byte** (`PLAN.md` §1 y §2.5).
//!
//! Al leer, el texto se entrega siempre con `\n` (lo que espera el editor) y se
//! recuerdan la codificación y el fin de línea originales; al escribir, se
//! restauran. Si el fichero mezclaba finales de línea se avisa con
//! [`TextFile::mixed_line_endings`]: al guardarlo quedará normalizado.

use serde::Serialize;

use super::{
    DecodeError, DetectionMethod, EncodeError, Encoding, LineEnding, LineEndingStats, LossStrategy,
    decode, decode_auto, encode, encode_lossy, normalize_line_endings,
};

/// Como [`write_text`], pero los caracteres que no caben en `encoding` se
/// tratan con `strategy` en lugar de dar error (ver [`super::loss_report`]).
///
/// # Examples
///
/// ```
/// use editormd_lib::model::text_codec::{write_text_lossy, Encoding, LineEnding, LossStrategy};
///
/// let bytes = write_text_lossy("10 €\n", Encoding::Ascii, LineEnding::Crlf, LossStrategy::Transliterate);
/// assert_eq!(bytes, b"10 EUR\r\n");
/// ```
pub fn write_text_lossy(
    text: &str,
    encoding: Encoding,
    line_ending: LineEnding,
    strategy: LossStrategy,
) -> Vec<u8> {
    encode_lossy(
        &normalize_line_endings(text, line_ending),
        encoding,
        strategy,
    )
}

/// Un fichero de texto leído, listo para el editor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextFile {
    /// Contenido, siempre con `\n` como fin de línea (sin BOM).
    pub text: String,
    /// Codificación con la que se leyó (y con la que se guardará).
    pub encoding: Encoding,
    /// Fin de línea predominante (y con el que se guardará). Si el fichero no
    /// tiene saltos de línea, [`LineEnding::Lf`].
    pub line_ending: LineEnding,
    /// `true` si el fichero mezclaba varios finales de línea.
    pub mixed_line_endings: bool,
    /// Cómo se detectó la codificación; `None` si la eligió el usuario
    /// («Reabrir con codificación…»).
    pub detection: Option<DetectionMethod>,
}

/// Lee un fichero detectando la codificación ([`decode_auto`]) y el fin de línea.
///
/// # Errors
///
/// Los de [`decode_auto`] (solo tras un BOM con contenido no válido).
///
/// # Examples
///
/// ```
/// use editormd_lib::model::text_codec::{read_text, Encoding, LineEnding};
///
/// let f = read_text(b"\xef\xbb\xbfuno\r\ndos\r\n").unwrap();
/// assert_eq!(f.text, "uno\ndos\n");
/// assert_eq!((f.encoding, f.line_ending), (Encoding::Utf8Bom, LineEnding::Crlf));
/// ```
pub fn read_text(bytes: &[u8]) -> Result<TextFile, DecodeError> {
    let decoded = decode_auto(bytes)?;
    Ok(TextFile::new(
        decoded.text,
        decoded.encoding,
        Some(decoded.method),
    ))
}

/// Lee un fichero con la codificación indicada («Reabrir con codificación…»).
///
/// # Errors
///
/// Los de [`decode`].
pub fn read_text_as(bytes: &[u8], encoding: Encoding) -> Result<TextFile, DecodeError> {
    Ok(TextFile::new(decode(bytes, encoding)?, encoding, None))
}

/// Prepara los bytes de un fichero: normaliza los finales de línea de `text` a
/// `line_ending` y lo codifica en `encoding` (con su BOM).
///
/// Para un texto devuelto por [`read_text`] o [`read_text_as`] sin editar, y con
/// su misma codificación y fin de línea, el resultado es idéntico al original
/// salvo que este mezclara finales de línea.
///
/// # Errors
///
/// [`EncodeError::Unmappable`] si algún carácter no cabe en `encoding`.
///
/// # Examples
///
/// ```
/// use editormd_lib::model::text_codec::{write_text, Encoding, LineEnding};
///
/// let bytes = write_text("uno\ndos\n", Encoding::Windows1252, LineEnding::Crlf).unwrap();
/// assert_eq!(bytes, b"uno\r\ndos\r\n");
/// ```
pub fn write_text(
    text: &str,
    encoding: Encoding,
    line_ending: LineEnding,
) -> Result<Vec<u8>, EncodeError> {
    encode(&normalize_line_endings(text, line_ending), encoding)
}

impl TextFile {
    /// Analiza los finales de línea del texto decodificado y lo normaliza a `\n`.
    fn new(raw: String, encoding: Encoding, detection: Option<DetectionMethod>) -> TextFile {
        let stats = LineEndingStats::of(&raw);
        let text = match normalize_line_endings(&raw, LineEnding::Lf) {
            std::borrow::Cow::Borrowed(_) => raw,
            std::borrow::Cow::Owned(normalized) => normalized,
        };
        TextFile {
            text,
            encoding,
            line_ending: stats.dominant().unwrap_or(LineEnding::Lf),
            mixed_line_endings: stats.is_mixed(),
            detection,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn fixtures() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/encodings")
    }

    /// Texto (con `\n`) que contiene cada fichero de `fixtures/encodings`
    /// (ver `scripts/generate-encoding-fixtures.py`).
    fn expected_text(encoding: Encoding) -> String {
        let latin = "# Título\n\nAño, pingüino, ¿qué tal? ¡Olé!\nÚltima línea con ñ y ç.\n";
        let unicode = "Emoji 😀 y € fuera de Latin-1.\n";
        match encoding {
            Encoding::Utf8 | Encoding::Utf8Bom | Encoding::Utf16Le | Encoding::Utf16Be => {
                format!("{latin}{unicode}")
            }
            Encoding::Ascii => "# Title\n\nPlain ASCII text.\nLast line.\n".to_owned(),
            Encoding::Iso8859_1 | Encoding::MacRoman => latin.to_owned(),
            Encoding::Iso8859_15 => format!("{latin}Euro: €.\n"),
            Encoding::Windows1252 => format!("{latin}Euro: € y comillas “así”.\n"),
        }
    }

    fn each_fixture() -> impl Iterator<Item = (Encoding, LineEnding, Vec<u8>)> {
        Encoding::ALL.into_iter().flat_map(|e| {
            LineEnding::ALL.into_iter().map(move |le| {
                let name = format!("{}-{}.txt", e.id(), le.id());
                let bytes = std::fs::read(fixtures().join(&name))
                    .unwrap_or_else(|err| panic!("{name}: {err}"));
                (e, le, bytes)
            })
        })
    }

    #[test]
    fn fixtures_matrix_is_complete() {
        assert_eq!(each_fixture().count(), 27);
    }

    #[test]
    fn read_as_declared_encoding_gives_text_and_line_ending() {
        for (e, le, bytes) in each_fixture() {
            let f = read_text_as(&bytes, e).unwrap();
            assert_eq!(f.text, expected_text(e), "{e:?} {le:?}");
            assert_eq!(f.encoding, e);
            assert_eq!(f.line_ending, le, "{e:?}");
            assert!(!f.mixed_line_endings);
            assert_eq!(f.detection, None);
        }
    }

    #[test]
    fn roundtrip_byte_for_byte_with_declared_encoding() {
        for (e, le, bytes) in each_fixture() {
            let f = read_text_as(&bytes, e).unwrap();
            assert_eq!(
                write_text(&f.text, f.encoding, f.line_ending).unwrap(),
                bytes,
                "{e:?} {le:?}"
            );
        }
    }

    #[test]
    fn roundtrip_byte_for_byte_with_autodetection() {
        // Aunque la detección no distinga Latin-1, Latin-9 o Mac Roman de
        // Windows-1252, guardar con lo detectado devuelve los mismos bytes.
        for (e, le, bytes) in each_fixture() {
            let f = read_text(&bytes).unwrap();
            assert!(f.detection.is_some());
            assert_eq!(
                write_text(&f.text, f.encoding, f.line_ending).unwrap(),
                bytes,
                "{e:?} {le:?} detectada como {:?}",
                f.encoding
            );
        }
    }

    #[test]
    fn autodetection_of_unicode_fixtures() {
        for (e, _, bytes) in each_fixture() {
            let f = read_text(&bytes).unwrap();
            match e {
                Encoding::Utf8Bom | Encoding::Utf16Le | Encoding::Utf16Be => {
                    assert_eq!((f.encoding, f.detection), (e, Some(DetectionMethod::Bom)));
                }
                // ASCII puro se detecta como UTF-8 (son los mismos bytes).
                Encoding::Utf8 | Encoding::Ascii => {
                    assert_eq!(
                        (f.encoding, f.detection),
                        (Encoding::Utf8, Some(DetectionMethod::Utf8Valid))
                    );
                }
                _ => assert_ne!(f.encoding, Encoding::Utf8, "{e:?}"),
            }
        }
    }

    #[test]
    fn mixed_line_endings_are_reported_and_normalized() {
        let bytes = std::fs::read(fixtures().join("mixed-eol.txt")).unwrap();
        let f = read_text(&bytes).unwrap();
        assert!(f.mixed_line_endings);
        assert_eq!(f.line_ending, LineEnding::Lf); // 2 LF, 1 CRLF, 1 CR
        assert_eq!(f.text, "uno\ndos\ntres\ncuatro\ncinco");
        assert_eq!(
            write_text(&f.text, f.encoding, f.line_ending).unwrap(),
            b"uno\ndos\ntres\ncuatro\ncinco"
        );
    }

    #[test]
    fn no_line_breaks_defaults_to_lf() {
        let f = read_text(b"una sola linea").unwrap();
        assert_eq!(f.line_ending, LineEnding::Lf);
        assert!(!f.mixed_line_endings);
        assert_eq!(read_text(b"").unwrap().text, "");
    }

    #[test]
    fn write_converts_between_line_endings_and_encodings() {
        let text = "uno\ndos\n";
        assert_eq!(
            write_text(text, Encoding::Utf16Be, LineEnding::Cr).unwrap(),
            [
                0xFE, 0xFF, 0, b'u', 0, b'n', 0, b'o', 0, b'\r', 0, b'd', 0, b'o', 0, b's', 0,
                b'\r'
            ]
        );
        // Un texto con \r\n o \r sueltos (p. ej. pegado) también se normaliza.
        assert_eq!(
            write_text("a\r\nb\rc", Encoding::Utf8, LineEnding::Lf).unwrap(),
            b"a\nb\nc"
        );
    }

    #[test]
    fn errors_are_propagated() {
        assert!(matches!(
            read_text(b"\xff\xfeA"),
            Err(DecodeError::OddLength { .. })
        ));
        assert!(matches!(
            read_text_as(b"\xf1", Encoding::Ascii),
            Err(DecodeError::InvalidSequence { offset: 0, .. })
        ));
        assert!(matches!(
            write_text("€", Encoding::Iso8859_1, LineEnding::Lf),
            Err(EncodeError::Unmappable { ch: '€', .. })
        ));
    }

    #[test]
    fn serializes_for_the_frontend() {
        let f = read_text(b"a\r\nb").unwrap();
        assert_eq!(
            serde_json::to_value(&f).unwrap(),
            serde_json::json!({
                "text": "a\nb",
                "encoding": "utf-8",
                "lineEnding": "crlf",
                "mixedLineEndings": false,
                "detection": "utf8-valid"
            })
        );
    }
}
