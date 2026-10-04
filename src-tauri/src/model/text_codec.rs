//! **M09 `TextCodec`** (Modelo · Rust) — detección y decodificación de texto.
//!
//! Convierte los bytes de un fichero en texto Unicode (D-15). Esta primera
//! parte cubre solo la lectura: detectar la codificación ([`detect`]) y
//! decodificar de forma estricta, sin pérdidas silenciosas ([`decode`],
//! [`decode_auto`]). Nunca se sustituye nada por `U+FFFD`: o se decodifica
//! todo o se devuelve un [`DecodeError`].

use std::fmt;

use serde::{Deserialize, Serialize};

/// Codificaciones soportadas (D-15).
///
/// Se serializa como su identificador (véase [`Encoding::id`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Encoding {
    /// UTF-8 sin BOM (`"utf-8"`).
    #[serde(rename = "utf-8")]
    Utf8,
    /// UTF-8 con BOM (`"utf-8-bom"`).
    #[serde(rename = "utf-8-bom")]
    Utf8Bom,
    /// UTF-16 little endian (`"utf-16le"`).
    #[serde(rename = "utf-16le")]
    Utf16Le,
    /// UTF-16 big endian (`"utf-16be"`).
    #[serde(rename = "utf-16be")]
    Utf16Be,
    /// ASCII de 7 bits (`"ascii"`).
    #[serde(rename = "ascii")]
    Ascii,
    /// ISO-8859-1 (Latin-1) real (`"iso-8859-1"`).
    #[serde(rename = "iso-8859-1")]
    Iso8859_1,
    /// ISO-8859-15 (Latin-9) (`"iso-8859-15"`).
    #[serde(rename = "iso-8859-15")]
    Iso8859_15,
    /// Windows-1252 (`"windows-1252"`).
    #[serde(rename = "windows-1252")]
    Windows1252,
    /// Mac OS Roman (`"macintosh"`).
    #[serde(rename = "macintosh")]
    MacRoman,
}

impl Encoding {
    /// Las 9 codificaciones, en el orden de la declaración.
    pub const ALL: [Encoding; 9] = [
        Encoding::Utf8,
        Encoding::Utf8Bom,
        Encoding::Utf16Le,
        Encoding::Utf16Be,
        Encoding::Ascii,
        Encoding::Iso8859_1,
        Encoding::Iso8859_15,
        Encoding::Windows1252,
        Encoding::MacRoman,
    ];

    /// Identificador estable (el mismo que usa el frontend en `src/model/document.ts`).
    pub fn id(self) -> &'static str {
        todo!()
    }

    /// Inverso de [`Encoding::id`]; `None` si el identificador no existe
    /// (distingue mayúsculas).
    pub fn from_id(id: &str) -> Option<Encoding> {
        let _ = id;
        todo!()
    }

    /// BOM de la codificación: `EF BB BF` (`Utf8Bom`), `FF FE` (`Utf16Le`),
    /// `FE FF` (`Utf16Be`); vacío en las demás.
    pub fn bom(self) -> &'static [u8] {
        todo!()
    }
}

/// Cómo se ha decidido la codificación.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DetectionMethod {
    /// Marca de orden de bytes al principio.
    Bom,
    /// El contenido es UTF-8 válido.
    Utf8Valid,
    /// Heurística estadística (`chardetng`).
    Heuristic,
    /// Sin criterio fiable: se asume Windows-1252.
    Fallback,
}

/// Resultado de [`detect`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Detection {
    /// Codificación detectada.
    pub encoding: Encoding,
    /// Cómo se decidió.
    pub method: DetectionMethod,
}

/// Resultado de [`decode_auto`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Decoded {
    /// Texto decodificado.
    pub text: String,
    /// Codificación usada.
    pub encoding: Encoding,
    /// Cómo se decidió.
    pub method: DetectionMethod,
}

/// Error de decodificación.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// Secuencia no válida; `offset` es la posición (en bytes, contando el BOM
    /// si lo hay) del primer byte de la primera secuencia no válida en la
    /// entrada original.
    InvalidSequence {
        /// Codificación con la que se intentaba decodificar.
        encoding: Encoding,
        /// Posición del primer byte no válido.
        offset: usize,
    },
    /// UTF-16 con un número impar de bytes (tras quitar el BOM).
    OddLength {
        /// Codificación con la que se intentaba decodificar.
        encoding: Encoding,
    },
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = f;
        todo!()
    }
}

impl std::error::Error for DecodeError {}

/// Detecta la codificación de `bytes`.
pub fn detect(bytes: &[u8]) -> Detection {
    let _ = bytes;
    todo!()
}

/// Decodifica `bytes` con la codificación indicada.
///
/// # Errors
///
/// Véase [`DecodeError`].
pub fn decode(bytes: &[u8], encoding: Encoding) -> Result<String, DecodeError> {
    let _ = (bytes, encoding);
    todo!()
}

/// Detecta y decodifica.
///
/// # Errors
///
/// Véase [`DecodeError`].
pub fn decode_auto(bytes: &[u8]) -> Result<Decoded, DecodeError> {
    let _ = bytes;
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utf16le(s: &str) -> Vec<u8> {
        s.encode_utf16().flat_map(u16::to_le_bytes).collect()
    }
    fn utf16be(s: &str) -> Vec<u8> {
        s.encode_utf16().flat_map(u16::to_be_bytes).collect()
    }
    fn det(bytes: &[u8]) -> (Encoding, DetectionMethod) {
        let d = detect(bytes);
        (d.encoding, d.method)
    }

    // ---- Encoding ----
    #[test]
    fn ids_roundtrip_and_are_stable() {
        let ids = [
            "utf-8",
            "utf-8-bom",
            "utf-16le",
            "utf-16be",
            "ascii",
            "iso-8859-1",
            "iso-8859-15",
            "windows-1252",
            "macintosh",
        ];
        for (enc, id) in Encoding::ALL.iter().zip(ids) {
            assert_eq!(enc.id(), id);
            assert_eq!(Encoding::from_id(id), Some(*enc));
        }
    }

    #[test]
    fn from_id_rejects_unknown_and_wrong_case() {
        assert_eq!(Encoding::from_id("UTF-8"), None);
        assert_eq!(Encoding::from_id(""), None);
        assert_eq!(Encoding::from_id("latin1"), None);
    }

    #[test]
    fn bom_values() {
        assert_eq!(Encoding::Utf8Bom.bom(), &[0xEF, 0xBB, 0xBF]);
        assert_eq!(Encoding::Utf16Le.bom(), &[0xFF, 0xFE]);
        assert_eq!(Encoding::Utf16Be.bom(), &[0xFE, 0xFF]);
        for enc in [
            Encoding::Utf8,
            Encoding::Ascii,
            Encoding::Iso8859_1,
            Encoding::Iso8859_15,
            Encoding::Windows1252,
            Encoding::MacRoman,
        ] {
            assert!(enc.bom().is_empty());
        }
    }

    #[test]
    fn serde_encoding_uses_id() {
        for enc in Encoding::ALL {
            let json = serde_json::to_string(&enc).unwrap();
            assert_eq!(json, format!("\"{}\"", enc.id()));
            assert_eq!(serde_json::from_str::<Encoding>(&json).unwrap(), enc);
        }
        assert!(serde_json::from_str::<Encoding>("\"nope\"").is_err());
    }

    #[test]
    fn serde_method_kebab_and_structs_camel() {
        let methods = [
            (DetectionMethod::Bom, "bom"),
            (DetectionMethod::Utf8Valid, "utf8-valid"),
            (DetectionMethod::Heuristic, "heuristic"),
            (DetectionMethod::Fallback, "fallback"),
        ];
        for (m, s) in methods {
            assert_eq!(serde_json::to_string(&m).unwrap(), format!("\"{s}\""));
        }
        let d = Detection {
            encoding: Encoding::Utf8Bom,
            method: DetectionMethod::Bom,
        };
        assert_eq!(
            serde_json::to_string(&d).unwrap(),
            r#"{"encoding":"utf-8-bom","method":"bom"}"#
        );
        let dec = Decoded {
            text: "a".into(),
            encoding: Encoding::Utf8,
            method: DetectionMethod::Utf8Valid,
        };
        assert_eq!(
            serde_json::to_string(&dec).unwrap(),
            r#"{"text":"a","encoding":"utf-8","method":"utf8-valid"}"#
        );
    }

    // ---- detect ----
    #[test]
    fn detect_boms() {
        assert_eq!(
            det(b"\xEF\xBB\xBFhola"),
            (Encoding::Utf8Bom, DetectionMethod::Bom)
        );
        assert_eq!(
            det(b"\xFF\xFEh\0"),
            (Encoding::Utf16Le, DetectionMethod::Bom)
        );
        assert_eq!(
            det(b"\xFE\xFF\0h"),
            (Encoding::Utf16Be, DetectionMethod::Bom)
        );
        // BOM solo, sin contenido.
        assert_eq!(
            det(b"\xEF\xBB\xBF"),
            (Encoding::Utf8Bom, DetectionMethod::Bom)
        );
        assert_eq!(det(b"\xFF\xFE"), (Encoding::Utf16Le, DetectionMethod::Bom));
    }

    #[test]
    fn detect_bom_wins_even_if_content_invalid() {
        assert_eq!(
            det(b"\xEF\xBB\xBF\xFF"),
            (Encoding::Utf8Bom, DetectionMethod::Bom)
        );
    }

    #[test]
    fn detect_empty_and_ascii_are_utf8() {
        assert_eq!(det(b""), (Encoding::Utf8, DetectionMethod::Utf8Valid));
        assert_eq!(
            det(b"# Hola\n"),
            (Encoding::Utf8, DetectionMethod::Utf8Valid)
        );
        assert_eq!(det(b"a\0b"), (Encoding::Utf8, DetectionMethod::Utf8Valid));
    }

    #[test]
    fn detect_valid_utf8() {
        assert_eq!(
            det("canción – 日本語 🦀".as_bytes()),
            (Encoding::Utf8, DetectionMethod::Utf8Valid)
        );
    }

    #[test]
    fn detect_partial_bom_is_not_bom() {
        assert_ne!(det(b"\xEF\xBB").1, DetectionMethod::Bom);
        assert_ne!(det(b"\xFF").1, DetectionMethod::Bom);
    }

    #[test]
    fn detect_heuristic_latin_text() {
        // "El niño comió una canción con el pingüino" en Windows-1252.
        let bytes = b"El ni\xF1o comi\xF3 una canci\xF3n con el ping\xFCino en la ma\xF1ana.";
        let d = detect(bytes);
        assert_eq!(d.method, DetectionMethod::Heuristic);
        assert_eq!(d.encoding, Encoding::Windows1252);
    }

    #[test]
    fn detect_fallback_for_other_encodings() {
        // Texto ruso en windows-1251: la heurística no lo da como latino.
        let (cow, _, _) = encoding_rs::WINDOWS_1251
            .encode("Привет, мир! Это длинный текст на русском языке для проверки определения.");
        let d = detect(&cow);
        assert_eq!(d.encoding, Encoding::Windows1252);
        assert_eq!(d.method, DetectionMethod::Fallback);
    }

    #[test]
    fn detect_never_returns_ascii_or_latin1() {
        for bytes in [&b""[..], b"abc", b"\xE9\xE8\xE0", b"\x80\x81", b"\xFF"] {
            let e = detect(bytes).encoding;
            assert!(e != Encoding::Ascii && e != Encoding::Iso8859_1);
        }
    }

    // ---- decode: UTF-8 ----
    #[test]
    fn utf8_ok_and_keeps_bom_char() {
        assert_eq!(decode("añ€🦀".as_bytes(), Encoding::Utf8).unwrap(), "añ€🦀");
        assert_eq!(decode(b"", Encoding::Utf8).unwrap(), "");
        assert_eq!(
            decode(b"\xEF\xBB\xBFa", Encoding::Utf8).unwrap(),
            "\u{FEFF}a"
        );
        assert_eq!(decode(b"a\0b", Encoding::Utf8).unwrap(), "a\0b");
    }

    #[test]
    fn utf8_invalid_offset() {
        assert_eq!(
            decode(b"ab\xE9c", Encoding::Utf8),
            Err(DecodeError::InvalidSequence {
                encoding: Encoding::Utf8,
                offset: 2
            })
        );
        // Secuencia truncada al final.
        assert_eq!(
            decode(b"ab\xE2\x82", Encoding::Utf8),
            Err(DecodeError::InvalidSequence {
                encoding: Encoding::Utf8,
                offset: 2
            })
        );
        // Sobrelarga y sustituto codificado.
        assert!(decode(b"\xC0\x80", Encoding::Utf8).is_err());
        assert!(decode(b"\xED\xA0\x80", Encoding::Utf8).is_err());
    }

    #[test]
    fn utf8_bom_strips_and_counts_offset() {
        assert_eq!(
            decode(b"\xEF\xBB\xBFhola", Encoding::Utf8Bom).unwrap(),
            "hola"
        );
        assert_eq!(decode(b"hola", Encoding::Utf8Bom).unwrap(), "hola");
        assert_eq!(decode(b"\xEF\xBB\xBF", Encoding::Utf8Bom).unwrap(), "");
        assert_eq!(
            decode(b"\xEF\xBB\xBFab\xFF", Encoding::Utf8Bom),
            Err(DecodeError::InvalidSequence {
                encoding: Encoding::Utf8Bom,
                offset: 5
            })
        );
        assert_eq!(
            decode(b"ab\xFF", Encoding::Utf8Bom),
            Err(DecodeError::InvalidSequence {
                encoding: Encoding::Utf8Bom,
                offset: 2
            })
        );
    }

    // ---- decode: UTF-16 ----
    #[test]
    fn utf16_roundtrip_with_and_without_bom() {
        let s = "Hola ñ € 🦀\n";
        let mut le = vec![0xFF, 0xFE];
        le.extend(utf16le(s));
        assert_eq!(decode(&le, Encoding::Utf16Le).unwrap(), s);
        assert_eq!(decode(&utf16le(s), Encoding::Utf16Le).unwrap(), s);
        let mut be = vec![0xFE, 0xFF];
        be.extend(utf16be(s));
        assert_eq!(decode(&be, Encoding::Utf16Be).unwrap(), s);
        assert_eq!(decode(&utf16be(s), Encoding::Utf16Be).unwrap(), s);
        assert_eq!(decode(b"", Encoding::Utf16Le).unwrap(), "");
        assert_eq!(decode(b"\xFF\xFE", Encoding::Utf16Le).unwrap(), "");
    }

    #[test]
    fn utf16_other_endianness_bom_is_a_char() {
        // BOM big endian leído como little endian: U+FFFE.
        assert_eq!(
            decode(b"\xFE\xFFa\0", Encoding::Utf16Le).unwrap(),
            "\u{FFFE}a"
        );
        // BOM little endian leído como big endian: U+FFFE.
        assert_eq!(
            decode(b"\xFF\xFE\0a", Encoding::Utf16Be).unwrap(),
            "\u{FFFE}a"
        );
    }

    #[test]
    fn utf16_odd_length() {
        assert_eq!(
            decode(b"a\0b", Encoding::Utf16Le),
            Err(DecodeError::OddLength {
                encoding: Encoding::Utf16Le
            })
        );
        assert_eq!(
            decode(b"\xFE\xFF\0a\0", Encoding::Utf16Be),
            Err(DecodeError::OddLength {
                encoding: Encoding::Utf16Be
            })
        );
        assert_eq!(
            decode(b"\xFF\xFE\x00", Encoding::Utf16Le),
            Err(DecodeError::OddLength {
                encoding: Encoding::Utf16Le
            })
        );
        assert_eq!(
            decode(b"\0", Encoding::Utf16Be),
            Err(DecodeError::OddLength {
                encoding: Encoding::Utf16Be
            })
        );
    }

    #[test]
    fn utf16_lone_surrogates() {
        // Alto sin pareja al final (offset 2), con BOM (+2).
        assert_eq!(
            decode(b"a\0\x3D\xD8", Encoding::Utf16Le),
            Err(DecodeError::InvalidSequence {
                encoding: Encoding::Utf16Le,
                offset: 2
            })
        );
        assert_eq!(
            decode(b"\xFF\xFEa\0\x3D\xD8", Encoding::Utf16Le),
            Err(DecodeError::InvalidSequence {
                encoding: Encoding::Utf16Le,
                offset: 4
            })
        );
        // Alto seguido de un no-bajo: el error es el alto.
        assert_eq!(
            decode(b"\xD8\x3D\0a", Encoding::Utf16Be),
            Err(DecodeError::InvalidSequence {
                encoding: Encoding::Utf16Be,
                offset: 0
            })
        );
        // Bajo suelto.
        assert_eq!(
            decode(b"\0a\xDC\x00", Encoding::Utf16Be),
            Err(DecodeError::InvalidSequence {
                encoding: Encoding::Utf16Be,
                offset: 2
            })
        );
        // Alto, alto, bajo: el primero es el inválido.
        assert_eq!(
            decode(b"\xD8\x3D\xD8\x3D\xDE\x00", Encoding::Utf16Be),
            Err(DecodeError::InvalidSequence {
                encoding: Encoding::Utf16Be,
                offset: 0
            })
        );
    }

    // ---- decode: ASCII y de un byte ----
    #[test]
    fn ascii_ok_and_error() {
        assert_eq!(decode(b"abc\0\x7F", Encoding::Ascii).unwrap(), "abc\0\x7F");
        assert_eq!(
            decode(b"ab\x80", Encoding::Ascii),
            Err(DecodeError::InvalidSequence {
                encoding: Encoding::Ascii,
                offset: 2
            })
        );
    }

    #[test]
    fn latin1_is_real_latin1() {
        let all: Vec<u8> = (0..=255).collect();
        let text = decode(&all, Encoding::Iso8859_1).unwrap();
        let expected: String = (0..=255u8).map(char::from).collect();
        assert_eq!(text, expected);
        assert_eq!(
            decode(b"\x80\x9F", Encoding::Iso8859_1).unwrap(),
            "\u{80}\u{9F}"
        );
    }

    #[test]
    fn latin9_differences() {
        assert_eq!(
            decode(b"\xA4\xA6\xA8\xB4\xB8\xBC\xBD\xBE", Encoding::Iso8859_15).unwrap(),
            "€ŠšŽžŒœŸ"
        );
        assert_eq!(decode(b"\xE9\xA3", Encoding::Iso8859_15).unwrap(), "é£");
    }

    #[test]
    fn windows1252_whatwg() {
        assert_eq!(
            decode(b"\x80\x93a\x94", Encoding::Windows1252).unwrap(),
            "€“a”"
        );
        assert_eq!(
            decode(b"\x81\x8D\x8F\x90\x9D", Encoding::Windows1252).unwrap(),
            "\u{81}\u{8D}\u{8F}\u{90}\u{9D}"
        );
    }

    #[test]
    fn mac_roman() {
        assert_eq!(decode(b"\x8E\x80\xA5", Encoding::MacRoman).unwrap(), "éÄ•");
        // Un BOM UTF-8 no se interpreta: son tres caracteres.
        assert_eq!(
            decode(b"\xEF\xBB\xBF", Encoding::MacRoman)
                .unwrap()
                .chars()
                .count(),
            3
        );
    }

    #[test]
    fn single_byte_decoders_never_fail() {
        let all: Vec<u8> = (0..=255).collect();
        for enc in [
            Encoding::Iso8859_1,
            Encoding::Iso8859_15,
            Encoding::Windows1252,
            Encoding::MacRoman,
        ] {
            let t = decode(&all, enc).unwrap();
            assert_eq!(t.chars().count(), 256);
            assert!(!t.contains('\u{FFFD}'));
        }
    }

    // ---- decode_auto ----
    #[test]
    fn auto_utf8_and_bom() {
        let d = decode_auto("hola ñ".as_bytes()).unwrap();
        assert_eq!(
            d,
            Decoded {
                text: "hola ñ".into(),
                encoding: Encoding::Utf8,
                method: DetectionMethod::Utf8Valid
            }
        );
        let d = decode_auto(b"\xEF\xBB\xBFhola").unwrap();
        assert_eq!(
            (d.text.as_str(), d.encoding, d.method),
            ("hola", Encoding::Utf8Bom, DetectionMethod::Bom)
        );
        let mut le = vec![0xFF, 0xFE];
        le.extend(utf16le("hi"));
        let d = decode_auto(&le).unwrap();
        assert_eq!((d.text.as_str(), d.encoding), ("hi", Encoding::Utf16Le));
        let mut be = vec![0xFE, 0xFF];
        be.extend(utf16be("hi"));
        let d = decode_auto(&be).unwrap();
        assert_eq!((d.text.as_str(), d.encoding), ("hi", Encoding::Utf16Be));
    }

    #[test]
    fn auto_legacy_never_fails() {
        let d = decode_auto(b"El ni\xF1o comi\xF3 una canci\xF3n con el ping\xFCino.").unwrap();
        assert_eq!(d.encoding, Encoding::Windows1252);
        assert!(d.text.contains("niño"));
    }

    #[test]
    fn auto_errors_after_bom() {
        assert_eq!(
            decode_auto(b"\xFF\xFEa\0b"),
            Err(DecodeError::OddLength {
                encoding: Encoding::Utf16Le
            })
        );
        assert_eq!(
            decode_auto(b"\xEF\xBB\xBF\xFF"),
            Err(DecodeError::InvalidSequence {
                encoding: Encoding::Utf8Bom,
                offset: 3
            })
        );
    }

    // ---- DecodeError ----
    #[test]
    fn error_display_spanish_with_id_and_offset() {
        let e = DecodeError::InvalidSequence {
            encoding: Encoding::Utf8,
            offset: 42,
        };
        let msg = e.to_string();
        assert!(msg.contains("utf-8") && msg.contains("42"), "{msg}");
        let e = DecodeError::OddLength {
            encoding: Encoding::Utf16Be,
        };
        let msg = e.to_string();
        assert!(msg.contains("utf-16be"), "{msg}");
        let _: &dyn std::error::Error = &e;
    }
}
