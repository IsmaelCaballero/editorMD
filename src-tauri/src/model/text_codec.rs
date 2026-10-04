//! **M09 `TextCodec`** (Modelo · Rust) — codificaciones y finales de línea de
//! los ficheros de texto (D-15, `PLAN.md` §2.5).
//!
//! - **Lectura** (este fichero): detectar la codificación de unos bytes y
//!   convertirlos a texto Unicode, sin sustituciones silenciosas por `U+FFFD`.
//! - **Escritura** ([`encode`]): texto → bytes con BOM, estricta.
//! - **Finales de línea** ([`LineEnding`], [`LineEndingStats`],
//!   [`normalize_line_endings`]).
//! - **Ficheros completos** ([`read_text`], [`read_text_as`], [`write_text`]):
//!   ida y vuelta byte a byte.
//!
//! El informe de pérdidas y la transliteración llegarán en el paso 3 de F2.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

mod encode;
mod line_ending;
mod text_file;

pub use encode::{EncodeError, encode};
pub use line_ending::{LineEnding, LineEndingStats, normalize_line_endings};
pub use text_file::{TextFile, read_text, read_text_as, write_text};

/// Codificaciones soportadas (D-15).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Encoding {
    /// UTF-8 sin BOM (`"utf-8"`).
    Utf8,
    /// UTF-8 con BOM (`"utf-8-bom"`).
    Utf8Bom,
    /// UTF-16 little endian con BOM (`"utf-16le"`).
    Utf16Le,
    /// UTF-16 big endian con BOM (`"utf-16be"`).
    Utf16Be,
    /// ASCII de 7 bits (`"ascii"`).
    Ascii,
    /// ISO-8859-1 (Latin-1) real (`"iso-8859-1"`).
    Iso8859_1,
    /// ISO-8859-15 (Latin-9) (`"iso-8859-15"`).
    Iso8859_15,
    /// Windows-1252 (`"windows-1252"`).
    Windows1252,
    /// Mac Roman (`"macintosh"`).
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
    ///
    /// # Examples
    ///
    /// ```
    /// use editormd_lib::model::text_codec::Encoding;
    ///
    /// assert_eq!(Encoding::Utf8Bom.id(), "utf-8-bom");
    /// ```
    pub fn id(self) -> &'static str {
        match self {
            Encoding::Utf8 => "utf-8",
            Encoding::Utf8Bom => "utf-8-bom",
            Encoding::Utf16Le => "utf-16le",
            Encoding::Utf16Be => "utf-16be",
            Encoding::Ascii => "ascii",
            Encoding::Iso8859_1 => "iso-8859-1",
            Encoding::Iso8859_15 => "iso-8859-15",
            Encoding::Windows1252 => "windows-1252",
            Encoding::MacRoman => "macintosh",
        }
    }

    /// Inverso de [`Encoding::id`]; `None` si el identificador no existe
    /// (distingue mayúsculas).
    pub fn from_id(id: &str) -> Option<Encoding> {
        Encoding::ALL.into_iter().find(|e| e.id() == id)
    }

    /// BOM de la codificación: `EF BB BF` (`Utf8Bom`), `FF FE` (`Utf16Le`),
    /// `FE FF` (`Utf16Be`); vacío en las demás.
    pub fn bom(self) -> &'static [u8] {
        match self {
            Encoding::Utf8Bom => &BOM_UTF8,
            Encoding::Utf16Le => &BOM_UTF16_LE,
            Encoding::Utf16Be => &BOM_UTF16_BE,
            _ => &[],
        }
    }
}

const BOM_UTF8: [u8; 3] = [0xEF, 0xBB, 0xBF];
const BOM_UTF16_LE: [u8; 2] = [0xFF, 0xFE];
const BOM_UTF16_BE: [u8; 2] = [0xFE, 0xFF];

impl Serialize for Encoding {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.id())
    }
}

impl<'de> Deserialize<'de> for Encoding {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let id = String::deserialize(deserializer)?;
        Encoding::from_id(&id)
            .ok_or_else(|| serde::de::Error::custom(format!("codificación desconocida: «{id}»")))
    }
}

/// Cómo se ha decidido la codificación.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DetectionMethod {
    /// Por la marca de orden de bytes (BOM).
    Bom,
    /// El contenido es UTF-8 válido.
    Utf8Valid,
    /// Por la heurística de `chardetng`.
    Heuristic,
    /// La heurística no dio una codificación soportada; se usa Windows-1252.
    Fallback,
}

/// Resultado de [`detect`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Detection {
    /// Codificación detectada.
    pub encoding: Encoding,
    /// Cómo se ha decidido.
    pub method: DetectionMethod,
}

/// Resultado de [`decode_auto`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Decoded {
    /// Texto decodificado (sin BOM).
    pub text: String,
    /// Codificación usada.
    pub encoding: Encoding,
    /// Cómo se decidió la codificación.
    pub method: DetectionMethod,
}

/// Error de decodificación.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// Secuencia no válida; `offset` = posición (en bytes, contando el BOM si lo hay)
    /// del primer byte de la primera secuencia no válida en la entrada original.
    InvalidSequence {
        /// Codificación con la que se intentó decodificar.
        encoding: Encoding,
        /// Posición en bytes de la secuencia no válida.
        offset: usize,
    },
    /// UTF-16 con un número impar de bytes (tras quitar el BOM).
    OddLength {
        /// Codificación con la que se intentó decodificar.
        encoding: Encoding,
    },
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::InvalidSequence { encoding, offset } => write!(
                f,
                "secuencia no válida para la codificación {} en el byte {offset}",
                encoding.id()
            ),
            DecodeError::OddLength { encoding } => write!(
                f,
                "longitud impar de bytes para la codificación {}",
                encoding.id()
            ),
        }
    }
}

impl std::error::Error for DecodeError {}

/// Detecta la codificación de `bytes`.
///
/// Orden: BOM, UTF-8 válido, heurística (`chardetng`) y, si esta da algo no
/// soportado, Windows-1252 como último recurso. Nunca devuelve `Ascii` ni
/// `Iso8859_1`.
///
/// # Examples
///
/// ```
/// use editormd_lib::model::text_codec::{detect, DetectionMethod, Encoding};
///
/// let d = detect("hola ñ".as_bytes());
/// assert_eq!(d.encoding, Encoding::Utf8);
/// assert_eq!(d.method, DetectionMethod::Utf8Valid);
/// ```
pub fn detect(bytes: &[u8]) -> Detection {
    let bom = [Encoding::Utf8Bom, Encoding::Utf16Le, Encoding::Utf16Be]
        .into_iter()
        .find(|e| bytes.starts_with(e.bom()));
    if let Some(encoding) = bom {
        return Detection {
            encoding,
            method: DetectionMethod::Bom,
        };
    }
    if std::str::from_utf8(bytes).is_ok() {
        return Detection {
            encoding: Encoding::Utf8,
            method: DetectionMethod::Utf8Valid,
        };
    }
    let mut detector = chardetng::EncodingDetector::new(chardetng::Iso2022JpDetection::Deny);
    detector.feed(bytes, true);
    let guess = detector.guess(None, chardetng::Utf8Detection::Deny);
    let supported = if guess == encoding_rs::WINDOWS_1252 {
        Some(Encoding::Windows1252)
    } else if guess == encoding_rs::ISO_8859_15 {
        Some(Encoding::Iso8859_15)
    } else if guess == encoding_rs::MACINTOSH {
        Some(Encoding::MacRoman)
    } else {
        None
    };
    match supported {
        Some(encoding) => Detection {
            encoding,
            method: DetectionMethod::Heuristic,
        },
        None => Detection {
            encoding: Encoding::Windows1252,
            method: DetectionMethod::Fallback,
        },
    }
}

/// Decodifica `bytes` con la codificación indicada, de forma estricta: o se
/// decodifica todo o hay error (nunca se sustituye por `U+FFFD`).
///
/// # Errors
///
/// [`DecodeError::InvalidSequence`] si hay una secuencia no válida y
/// [`DecodeError::OddLength`] si UTF-16 tiene un número impar de bytes.
///
/// # Examples
///
/// ```
/// use editormd_lib::model::text_codec::{decode, Encoding};
///
/// assert_eq!(decode(b"caf\xe9", Encoding::Iso8859_1).unwrap(), "café");
/// assert!(decode(b"caf\xe9", Encoding::Utf8).is_err());
/// ```
pub fn decode(bytes: &[u8], encoding: Encoding) -> Result<String, DecodeError> {
    match encoding {
        Encoding::Utf8 => decode_utf8(bytes, 0, encoding),
        Encoding::Utf8Bom => {
            let skip = if bytes.starts_with(&BOM_UTF8) {
                BOM_UTF8.len()
            } else {
                0
            };
            decode_utf8(&bytes[skip..], skip, encoding)
        }
        Encoding::Utf16Le | Encoding::Utf16Be => decode_utf16(bytes, encoding),
        Encoding::Ascii => match bytes.iter().position(|b| !b.is_ascii()) {
            Some(offset) => Err(DecodeError::InvalidSequence { encoding, offset }),
            None => Ok(bytes.iter().copied().map(char::from).collect()),
        },
        // `encoding_rs` trata «ISO-8859-1» como windows-1252, así que se hace a mano.
        Encoding::Iso8859_1 => Ok(bytes.iter().copied().map(char::from).collect()),
        Encoding::Iso8859_15 => Ok(decode_single_byte(bytes, encoding_rs::ISO_8859_15)),
        Encoding::Windows1252 => Ok(decode_single_byte(bytes, encoding_rs::WINDOWS_1252)),
        Encoding::MacRoman => Ok(decode_single_byte(bytes, encoding_rs::MACINTOSH)),
    }
}

/// UTF-8 estricto; `base` es la longitud del BOM ya quitado, para el `offset`.
fn decode_utf8(bytes: &[u8], base: usize, encoding: Encoding) -> Result<String, DecodeError> {
    std::str::from_utf8(bytes)
        .map(str::to_owned)
        .map_err(|e| DecodeError::InvalidSequence {
            encoding,
            offset: base + e.valid_up_to(),
        })
}

/// UTF-16 estricto (LE o BE según `encoding`).
fn decode_utf16(bytes: &[u8], encoding: Encoding) -> Result<String, DecodeError> {
    let little = encoding == Encoding::Utf16Le;
    let skip = if bytes.starts_with(encoding.bom()) {
        2
    } else {
        0
    };
    let body = &bytes[skip..];
    if !body.len().is_multiple_of(2) {
        return Err(DecodeError::OddLength { encoding });
    }
    let units = body.as_chunks::<2>().0.iter().map(|pair| {
        if little {
            u16::from_le_bytes(*pair)
        } else {
            u16::from_be_bytes(*pair)
        }
    });
    let mut text = String::with_capacity(body.len() / 2);
    let mut index = 0usize; // unidades consumidas
    for result in char::decode_utf16(units) {
        match result {
            Ok(c) => {
                index += c.len_utf16();
                text.push(c);
            }
            Err(_) => {
                return Err(DecodeError::InvalidSequence {
                    encoding,
                    offset: skip + index * 2,
                });
            }
        }
    }
    Ok(text)
}

/// Codificaciones de un byte de `encoding_rs`, que nunca fallan. Sin tratar BOM.
fn decode_single_byte(bytes: &[u8], codec: &'static encoding_rs::Encoding) -> String {
    let (text, _) = codec.decode_without_bom_handling(bytes);
    text.into_owned()
}

/// Detecta la codificación y decodifica con ella.
///
/// # Errors
///
/// Los de [`decode`]; solo puede fallar tras un BOM.
pub fn decode_auto(bytes: &[u8]) -> Result<Decoded, DecodeError> {
    let Detection { encoding, method } = detect(bytes);
    let text = decode(bytes, encoding)?;
    Ok(Decoded {
        text,
        encoding,
        method,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utf16(units: &[u16], le: bool) -> Vec<u8> {
        units
            .iter()
            .flat_map(|u| if le { u.to_le_bytes() } else { u.to_be_bytes() })
            .collect()
    }

    fn invalid(encoding: Encoding, offset: usize) -> Result<String, DecodeError> {
        Err(DecodeError::InvalidSequence { encoding, offset })
    }

    fn odd(encoding: Encoding) -> Result<String, DecodeError> {
        Err(DecodeError::OddLength { encoding })
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
        for (e, id) in Encoding::ALL.into_iter().zip(ids) {
            assert_eq!(e.id(), id);
            assert_eq!(Encoding::from_id(id), Some(e));
        }
        assert_eq!(Encoding::from_id("UTF-8"), None);
        assert_eq!(Encoding::from_id(""), None);
    }

    #[test]
    fn boms() {
        assert_eq!(Encoding::Utf8Bom.bom(), &[0xEF, 0xBB, 0xBF]);
        assert_eq!(Encoding::Utf16Le.bom(), &[0xFF, 0xFE]);
        assert_eq!(Encoding::Utf16Be.bom(), &[0xFE, 0xFF]);
        assert_eq!(Encoding::Utf8.bom(), &[] as &[u8]);
        assert_eq!(Encoding::MacRoman.bom(), &[] as &[u8]);
    }

    #[test]
    fn serde_formats() {
        assert_eq!(
            serde_json::to_string(&Encoding::Utf8Bom).unwrap(),
            "\"utf-8-bom\""
        );
        assert_eq!(
            serde_json::from_str::<Encoding>("\"macintosh\"").unwrap(),
            Encoding::MacRoman
        );
        assert!(serde_json::from_str::<Encoding>("\"nope\"").is_err());
        assert_eq!(
            serde_json::to_string(&DetectionMethod::Utf8Valid).unwrap(),
            "\"utf8-valid\""
        );
        let d = Detection {
            encoding: Encoding::Utf16Le,
            method: DetectionMethod::Bom,
        };
        assert_eq!(
            serde_json::to_string(&d).unwrap(),
            r#"{"encoding":"utf-16le","method":"bom"}"#
        );
        let dec = Decoded {
            text: "a".into(),
            encoding: Encoding::Ascii,
            method: DetectionMethod::Fallback,
        };
        assert_eq!(
            serde_json::to_string(&dec).unwrap(),
            r#"{"text":"a","encoding":"ascii","method":"fallback"}"#
        );
    }

    #[test]
    fn error_display() {
        let e = DecodeError::InvalidSequence {
            encoding: Encoding::Utf8,
            offset: 7,
        };
        let s = e.to_string();
        assert!(s.contains("utf-8") && s.contains('7'));
        let e = DecodeError::OddLength {
            encoding: Encoding::Utf16Be,
        };
        assert!(e.to_string().contains("utf-16be"));
        let _: &dyn std::error::Error = &e;
    }

    // ---- detect ----

    #[test]
    fn detect_boms() {
        let cases: [(&[u8], Encoding); 4] = [
            (&[0xEF, 0xBB, 0xBF, b'a'], Encoding::Utf8Bom),
            (&[0xFF, 0xFE, b'a', 0], Encoding::Utf16Le),
            (&[0xFE, 0xFF, 0, b'a'], Encoding::Utf16Be),
            (&[0xFF, 0xFE], Encoding::Utf16Le),
        ];
        for (bytes, encoding) in cases {
            let d = detect(bytes);
            assert_eq!((d.encoding, d.method), (encoding, DetectionMethod::Bom));
        }
    }

    #[test]
    fn detect_utf8_valid() {
        for bytes in [&b""[..], b"hello", "áéí €".as_bytes(), b"a\0b"] {
            let d = detect(bytes);
            assert_eq!(
                (d.encoding, d.method),
                (Encoding::Utf8, DetectionMethod::Utf8Valid)
            );
        }
    }

    #[test]
    fn detect_heuristic_windows_1252() {
        let bytes =
            b"El ni\xf1o comi\xf3 una pi\xf1a en la ma\xf1ana y despu\xe9s se fue a la cama.";
        let d = detect(bytes);
        assert_eq!(d.encoding, Encoding::Windows1252);
        assert_eq!(d.method, DetectionMethod::Heuristic);
    }

    #[test]
    fn detect_fallback_for_unsupported() {
        // Cirílico en windows-1251: la heurística no da una codificación soportada.
        let bytes = b"\xcf\xf0\xe8\xe2\xe5\xf2, \xec\xe8\xf0! \xca\xe0\xea \xe4\xe5\xeb\xe0?";
        let d = detect(bytes);
        assert_eq!(d.encoding, Encoding::Windows1252);
        assert_eq!(d.method, DetectionMethod::Fallback);
    }

    #[test]
    fn detect_never_ascii_or_latin1() {
        for bytes in [&b"abc"[..], b"\xe9", b"\x80\x81", b"\xff"] {
            let e = detect(bytes).encoding;
            assert!(e != Encoding::Ascii && e != Encoding::Iso8859_1);
        }
    }

    // ---- decode: UTF-8 ----

    #[test]
    fn utf8_strict_keeps_bom_char() {
        assert_eq!(
            decode(b"\xEF\xBB\xBFa", Encoding::Utf8).unwrap(),
            "\u{FEFF}a"
        );
        assert_eq!(decode(b"", Encoding::Utf8).unwrap(), "");
        assert_eq!(decode(b"a\0b", Encoding::Utf8).unwrap(), "a\0b");
    }

    #[test]
    fn utf8_invalid_offset() {
        assert_eq!(
            decode(b"ab\xffc", Encoding::Utf8),
            invalid(Encoding::Utf8, 2)
        );
        // Secuencia truncada al final.
        assert_eq!(
            decode(b"a\xE2\x82", Encoding::Utf8),
            invalid(Encoding::Utf8, 1)
        );
    }

    #[test]
    fn utf8_bom_strip_and_offset() {
        assert_eq!(
            decode(b"\xEF\xBB\xBFhola", Encoding::Utf8Bom).unwrap(),
            "hola"
        );
        assert_eq!(decode(b"hola", Encoding::Utf8Bom).unwrap(), "hola");
        assert_eq!(decode(b"\xEF\xBB\xBF", Encoding::Utf8Bom).unwrap(), "");
        assert_eq!(
            decode(b"\xEF\xBB\xBFab\xff", Encoding::Utf8Bom),
            invalid(Encoding::Utf8Bom, 5)
        );
        assert_eq!(
            decode(b"ab\xff", Encoding::Utf8Bom),
            invalid(Encoding::Utf8Bom, 2)
        );
    }

    // ---- decode: UTF-16 ----

    #[test]
    fn utf16_with_and_without_bom() {
        let mut le = vec![0xFF, 0xFE];
        le.extend(utf16(&[0x68, 0xE9, 0x20AC], true));
        assert_eq!(decode(&le, Encoding::Utf16Le).unwrap(), "hé€");
        assert_eq!(decode(&le[2..], Encoding::Utf16Le).unwrap(), "hé€");
        let mut be = vec![0xFE, 0xFF];
        be.extend(utf16(&[0x68, 0xE9, 0x20AC], false));
        assert_eq!(decode(&be, Encoding::Utf16Be).unwrap(), "hé€");
        assert_eq!(decode(&be[2..], Encoding::Utf16Be).unwrap(), "hé€");
        assert_eq!(decode(&[], Encoding::Utf16Le).unwrap(), "");
        assert_eq!(decode(&[0xFF, 0xFE], Encoding::Utf16Le).unwrap(), "");
    }

    #[test]
    fn utf16_surrogate_pair() {
        let bytes = utf16(&[0xD83D, 0xDE00], true);
        assert_eq!(decode(&bytes, Encoding::Utf16Le).unwrap(), "😀");
        let bytes = utf16(&[0xD83D, 0xDE00], false);
        assert_eq!(decode(&bytes, Encoding::Utf16Be).unwrap(), "😀");
    }

    #[test]
    fn utf16_other_endian_bom_is_a_char() {
        let text = decode(&[0xFE, 0xFF, 0x61, 0x00], Encoding::Utf16Le).unwrap();
        assert_eq!(text, "\u{FFFE}a");
        let text = decode(&[0xFF, 0xFE, 0x00, 0x61], Encoding::Utf16Be).unwrap();
        assert_eq!(text, "\u{FFFE}a");
    }

    #[test]
    fn utf16_odd_length() {
        assert_eq!(
            decode(&[0x61, 0x00, 0x62], Encoding::Utf16Le),
            odd(Encoding::Utf16Le)
        );
        assert_eq!(
            decode(&[0xFE, 0xFF, 0x00], Encoding::Utf16Be),
            odd(Encoding::Utf16Be)
        );
        assert_eq!(
            decode(&[0xFF, 0xFE, 0x61], Encoding::Utf16Le),
            odd(Encoding::Utf16Le)
        );
    }

    #[test]
    fn utf16_lone_surrogates() {
        // Alto sin pareja en medio.
        let bytes = utf16(&[0x61, 0xD83D, 0x62], true);
        assert_eq!(
            decode(&bytes, Encoding::Utf16Le),
            invalid(Encoding::Utf16Le, 2)
        );
        // Bajo sin alto, con BOM (el offset cuenta el BOM).
        let mut bytes = vec![0xFE, 0xFF];
        bytes.extend(utf16(&[0x61, 0x62, 0xDE00], false));
        assert_eq!(
            decode(&bytes, Encoding::Utf16Be),
            invalid(Encoding::Utf16Be, 6)
        );
        // Alto al final.
        let bytes = utf16(&[0xD83D], true);
        assert_eq!(
            decode(&bytes, Encoding::Utf16Le),
            invalid(Encoding::Utf16Le, 0)
        );
    }

    // ---- decode: ASCII y de un byte ----

    #[test]
    fn ascii() {
        assert_eq!(decode(b"abc\0", Encoding::Ascii).unwrap(), "abc\0");
        assert_eq!(
            decode(b"ab\x80", Encoding::Ascii),
            invalid(Encoding::Ascii, 2)
        );
    }

    #[test]
    fn latin1_is_real_latin1() {
        let all: Vec<u8> = (0..=255).collect();
        let text = decode(&all, Encoding::Iso8859_1).unwrap();
        assert_eq!(text.chars().count(), 256);
        for (i, c) in text.chars().enumerate() {
            assert_eq!(c as usize, i);
        }
    }

    #[test]
    fn latin9_differences() {
        let bytes = [0xA4, 0xA6, 0xA8, 0xB4, 0xB8, 0xBC, 0xBD, 0xBE, 0xE9];
        assert_eq!(decode(&bytes, Encoding::Iso8859_15).unwrap(), "€ŠšŽžŒœŸé");
    }

    #[test]
    fn windows1252_whatwg() {
        assert_eq!(
            decode(b"\x80\x93", Encoding::Windows1252).unwrap(),
            "€\u{201C}"
        );
        assert_eq!(
            decode(b"\x81\x8D\x8F\x90\x9D", Encoding::Windows1252).unwrap(),
            "\u{81}\u{8D}\u{8F}\u{90}\u{9D}"
        );
        assert_eq!(decode(b"a\0", Encoding::Windows1252).unwrap(), "a\0");
    }

    #[test]
    fn mac_roman() {
        assert_eq!(decode(b"\x8E\xA5", Encoding::MacRoman).unwrap(), "é•");
    }

    #[test]
    fn single_byte_encodings_never_fail() {
        let all: Vec<u8> = (0..=255).collect();
        for e in [
            Encoding::Iso8859_1,
            Encoding::Iso8859_15,
            Encoding::Windows1252,
            Encoding::MacRoman,
        ] {
            let text = decode(&all, e).unwrap();
            assert_eq!(text.chars().count(), 256);
            assert!(!text.contains('\u{FFFD}'));
        }
    }

    // ---- decode_auto ----

    #[test]
    fn auto_ok() {
        let r = decode_auto("hola €".as_bytes()).unwrap();
        assert_eq!(r.text, "hola €");
        assert_eq!(
            (r.encoding, r.method),
            (Encoding::Utf8, DetectionMethod::Utf8Valid)
        );
        let r = decode_auto(b"\xEF\xBB\xBFhola").unwrap();
        assert_eq!(r.text, "hola");
        assert_eq!(
            (r.encoding, r.method),
            (Encoding::Utf8Bom, DetectionMethod::Bom)
        );
        let mut bytes = vec![0xFF, 0xFE];
        bytes.extend(utf16(&[0x68, 0x69], true));
        let r = decode_auto(&bytes).unwrap();
        assert_eq!(r.text, "hi");
        assert_eq!(r.encoding, Encoding::Utf16Le);
        let r = decode_auto(b"El ni\xf1o comi\xf3 una pi\xf1a en la ma\xf1ana.").unwrap();
        assert!(r.text.contains("niño"));
        assert_eq!(r.encoding, Encoding::Windows1252);
    }

    #[test]
    fn auto_errors_after_bom() {
        assert_eq!(
            decode_auto(&[0xFF, 0xFE, 0x61]).map(|d| d.text),
            odd(Encoding::Utf16Le)
        );
        assert_eq!(
            decode_auto(b"\xEF\xBB\xBFa\xff").map(|d| d.text),
            invalid(Encoding::Utf8Bom, 4)
        );
    }
}
