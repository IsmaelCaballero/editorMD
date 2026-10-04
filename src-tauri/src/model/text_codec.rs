//! **M09 `TextCodec`** (Modelo · Rust) — detección y decodificación de texto.
//!
//! Convierte los bytes de un fichero de texto en una cadena Unicode. Esta
//! primera parte cubre solo la **lectura**: detectar la codificación
//! ([`detect`]) y decodificar de forma estricta ([`decode`], [`decode_auto`]).
//! Nunca se sustituye nada por `U+FFFD`: o se decodifica todo o se devuelve un
//! [`DecodeError`] con la posición del primer problema.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
    /// ISO-8859-1 (Latin-1 real) (`"iso-8859-1"`).
    Iso8859_1,
    /// ISO-8859-15 (Latin-9) (`"iso-8859-15"`).
    Iso8859_15,
    /// Windows-1252 (`"windows-1252"`).
    Windows1252,
    /// Mac OS Roman (`"macintosh"`).
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
            Encoding::Utf16Le => &BOM_UTF16LE,
            Encoding::Utf16Be => &BOM_UTF16BE,
            _ => &[],
        }
    }
}

const BOM_UTF8: [u8; 3] = [0xEF, 0xBB, 0xBF];
const BOM_UTF16LE: [u8; 2] = [0xFF, 0xFE];
const BOM_UTF16BE: [u8; 2] = [0xFE, 0xFF];

impl Serialize for Encoding {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.id())
    }
}

impl<'de> Deserialize<'de> for Encoding {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let id = String::deserialize(deserializer)?;
        Encoding::from_id(&id)
            .ok_or_else(|| serde::de::Error::custom(format!("codificación desconocida: {id:?}")))
    }
}

/// Cómo se ha decidido la codificación.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DetectionMethod {
    /// La entrada empieza por un BOM.
    Bom,
    /// La entrada es UTF-8 válido.
    Utf8Valid,
    /// Decidido por la heurística de `chardetng`.
    Heuristic,
    /// La heurística dio un resultado no soportado; se usa Windows-1252.
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
    /// Texto decodificado (sin BOM si la codificación lo lleva).
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
        /// Codificación con la que se intentó decodificar.
        encoding: Encoding,
        /// Posición del primer byte no válido.
        offset: usize,
    },
    /// UTF-16 con un número impar de bytes (tras quitar el BOM).
    OddLength {
        /// Codificación UTF-16 con la que se intentó decodificar.
        encoding: Encoding,
    },
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::InvalidSequence { encoding, offset } => write!(
                f,
                "secuencia no válida en {} en el byte {offset}",
                encoding.id()
            ),
            DecodeError::OddLength { encoding } => write!(
                f,
                "longitud impar no válida para {}: falta un byte en la última unidad",
                encoding.id()
            ),
        }
    }
}

impl std::error::Error for DecodeError {}

/// Detecta la codificación de `bytes`.
///
/// Orden: BOM, UTF-8 válido (incluye vacío y ASCII puro), heurística de
/// `chardetng` y, si da algo no soportado, Windows-1252 como respaldo.
///
/// # Examples
///
/// ```
/// use editormd_lib::model::text_codec::{detect, DetectionMethod, Encoding};
///
/// let d = detect("hola".as_bytes());
/// assert_eq!(d.encoding, Encoding::Utf8);
/// assert_eq!(d.method, DetectionMethod::Utf8Valid);
///
/// let d = detect(&[0xFF, 0xFE, b'a', 0]);
/// assert_eq!(d.encoding, Encoding::Utf16Le);
/// assert_eq!(d.method, DetectionMethod::Bom);
/// ```
pub fn detect(bytes: &[u8]) -> Detection {
    let bom = |encoding| Detection {
        encoding,
        method: DetectionMethod::Bom,
    };
    if bytes.starts_with(&BOM_UTF8) {
        return bom(Encoding::Utf8Bom);
    }
    if bytes.starts_with(&BOM_UTF16LE) {
        return bom(Encoding::Utf16Le);
    }
    if bytes.starts_with(&BOM_UTF16BE) {
        return bom(Encoding::Utf16Be);
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

/// Decodifica `bytes` con `encoding` de forma estricta: sin sustituciones
/// por `U+FFFD`.
///
/// `Utf8` no quita un BOM inicial; `Utf8Bom` y UTF-16 sí quitan el suyo si
/// está. `Iso8859_1` es Latin-1 real (cada byte es el punto de código igual).
///
/// # Errors
///
/// [`DecodeError::InvalidSequence`] si hay una secuencia no válida (UTF-8,
/// sustituto UTF-16 sin pareja o byte `>= 0x80` en ASCII) y
/// [`DecodeError::OddLength`] si UTF-16 tiene un número impar de bytes.
///
/// # Examples
///
/// ```
/// use editormd_lib::model::text_codec::{decode, DecodeError, Encoding};
///
/// assert_eq!(decode(&[0xE9], Encoding::Iso8859_1).as_deref(), Ok("é"));
/// assert_eq!(
///     decode(b"a\xFF", Encoding::Utf8),
///     Err(DecodeError::InvalidSequence { encoding: Encoding::Utf8, offset: 1 })
/// );
/// ```
pub fn decode(bytes: &[u8], encoding: Encoding) -> Result<String, DecodeError> {
    match encoding {
        Encoding::Utf8 => decode_utf8(bytes, 0, encoding),
        Encoding::Utf8Bom => {
            let skip = if bytes.starts_with(&BOM_UTF8) { 3 } else { 0 };
            decode_utf8(&bytes[skip..], skip, encoding)
        }
        Encoding::Utf16Le => decode_utf16(bytes, encoding, &BOM_UTF16LE, u16::from_le_bytes),
        Encoding::Utf16Be => decode_utf16(bytes, encoding, &BOM_UTF16BE, u16::from_be_bytes),
        Encoding::Ascii => match bytes.iter().position(|b| !b.is_ascii()) {
            Some(offset) => Err(DecodeError::InvalidSequence { encoding, offset }),
            None => Ok(bytes.iter().map(|&b| char::from(b)).collect()),
        },
        Encoding::Iso8859_1 => Ok(bytes.iter().map(|&b| char::from(b)).collect()),
        Encoding::Iso8859_15 => Ok(decode_single_byte(bytes, encoding_rs::ISO_8859_15)),
        Encoding::Windows1252 => Ok(decode_single_byte(bytes, encoding_rs::WINDOWS_1252)),
        Encoding::MacRoman => Ok(decode_single_byte(bytes, encoding_rs::MACINTOSH)),
    }
}

/// UTF-8 estricto; `base` es la longitud del BOM ya quitado.
fn decode_utf8(bytes: &[u8], base: usize, encoding: Encoding) -> Result<String, DecodeError> {
    std::str::from_utf8(bytes)
        .map(str::to_owned)
        .map_err(|e| DecodeError::InvalidSequence {
            encoding,
            offset: base + e.valid_up_to(),
        })
}

/// UTF-16 estricto; quita `bom` si está al principio.
fn decode_utf16(
    bytes: &[u8],
    encoding: Encoding,
    bom: &[u8],
    to_unit: fn([u8; 2]) -> u16,
) -> Result<String, DecodeError> {
    let skip = if bytes.starts_with(bom) { bom.len() } else { 0 };
    let body = &bytes[skip..];
    if !body.len().is_multiple_of(2) {
        return Err(DecodeError::OddLength { encoding });
    }
    let units = body.as_chunks::<2>().0.iter().map(|&c| to_unit(c));
    let mut text = String::with_capacity(body.len() / 2);
    let mut offset = skip;
    for item in char::decode_utf16(units) {
        let Ok(c) = item else {
            return Err(DecodeError::InvalidSequence { encoding, offset });
        };
        text.push(c);
        offset += 2 * c.len_utf16();
    }
    Ok(text)
}

/// Decodifica con una codificación de un solo byte de `encoding_rs`: cada
/// byte tiene carácter asignado, así que nunca hay sustituciones.
fn decode_single_byte(bytes: &[u8], codec: &'static encoding_rs::Encoding) -> String {
    codec.decode_without_bom_handling(bytes).0.into_owned()
}

/// Detecta la codificación y decodifica con ella.
///
/// # Errors
///
/// Los de [`decode`]; solo puede fallar tras un BOM (p. ej. `FF FE` seguido
/// de un número impar de bytes).
///
/// # Examples
///
/// ```
/// use editormd_lib::model::text_codec::{decode_auto, DetectionMethod, Encoding};
///
/// let d = decode_auto(&[0xEF, 0xBB, 0xBF, b'h', b'i']).unwrap();
/// assert_eq!(d.text, "hi");
/// assert_eq!(d.encoding, Encoding::Utf8Bom);
/// assert_eq!(d.method, DetectionMethod::Bom);
/// ```
pub fn decode_auto(bytes: &[u8]) -> Result<Decoded, DecodeError> {
    let Detection { encoding, method } = detect(bytes);
    decode(bytes, encoding).map(|text| Decoded {
        text,
        encoding,
        method,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn invalid(encoding: Encoding, offset: usize) -> DecodeError {
        DecodeError::InvalidSequence { encoding, offset }
    }

    #[test]
    fn ids_roundtrip_and_are_case_sensitive() {
        for e in Encoding::ALL {
            assert_eq!(Encoding::from_id(e.id()), Some(e));
        }
        assert_eq!(Encoding::from_id("UTF-8"), None);
        assert_eq!(Encoding::from_id(""), None);
        assert_eq!(Encoding::MacRoman.id(), "macintosh");
        assert_eq!(Encoding::ALL.len(), 9);
        assert_eq!(Encoding::ALL[0], Encoding::Utf8);
        assert_eq!(Encoding::ALL[8], Encoding::MacRoman);
    }

    #[test]
    fn boms() {
        assert_eq!(Encoding::Utf8Bom.bom(), &[0xEF, 0xBB, 0xBF]);
        assert_eq!(Encoding::Utf16Le.bom(), &[0xFF, 0xFE]);
        assert_eq!(Encoding::Utf16Be.bom(), &[0xFE, 0xFF]);
        assert_eq!(Encoding::Utf8.bom(), &[] as &[u8]);
        assert_eq!(Encoding::Windows1252.bom(), &[] as &[u8]);
    }

    #[test]
    fn serde_encoding_as_id() {
        assert_eq!(
            serde_json::to_string(&Encoding::Utf8Bom).unwrap(),
            "\"utf-8-bom\""
        );
        let e: Encoding = serde_json::from_str("\"iso-8859-15\"").unwrap();
        assert_eq!(e, Encoding::Iso8859_15);
        assert!(serde_json::from_str::<Encoding>("\"nope\"").is_err());
        assert!(serde_json::from_str::<Encoding>("3").is_err());
    }

    #[test]
    fn serde_detection_and_decoded() {
        let d = Detection {
            encoding: Encoding::Utf8,
            method: DetectionMethod::Utf8Valid,
        };
        assert_eq!(
            serde_json::to_string(&d).unwrap(),
            r#"{"encoding":"utf-8","method":"utf8-valid"}"#
        );
        let d = Decoded {
            text: "x".into(),
            encoding: Encoding::Ascii,
            method: DetectionMethod::Fallback,
        };
        assert_eq!(
            serde_json::to_string(&d).unwrap(),
            r#"{"text":"x","encoding":"ascii","method":"fallback"}"#
        );
        assert_eq!(
            serde_json::to_string(&DetectionMethod::Bom).unwrap(),
            "\"bom\""
        );
        assert_eq!(
            serde_json::to_string(&DetectionMethod::Heuristic).unwrap(),
            "\"heuristic\""
        );
    }

    #[test]
    fn error_display() {
        let m = invalid(Encoding::Utf8, 7).to_string();
        assert!(m.contains("utf-8") && m.contains('7'));
        let m = DecodeError::OddLength {
            encoding: Encoding::Utf16Be,
        }
        .to_string();
        assert!(m.contains("utf-16be"));
        let e: &dyn std::error::Error = &invalid(Encoding::Ascii, 0);
        assert_ne!(e.to_string(), "");
    }

    #[test]
    fn detect_bom() {
        let d = detect(&[0xEF, 0xBB, 0xBF]);
        assert_eq!(
            (d.encoding, d.method),
            (Encoding::Utf8Bom, DetectionMethod::Bom)
        );
        let d = detect(&[0xFF, 0xFE, 0x41]);
        assert_eq!(
            (d.encoding, d.method),
            (Encoding::Utf16Le, DetectionMethod::Bom)
        );
        let d = detect(&[0xFE, 0xFF]);
        assert_eq!(
            (d.encoding, d.method),
            (Encoding::Utf16Be, DetectionMethod::Bom)
        );
    }

    #[test]
    fn detect_utf8() {
        for input in [
            &b""[..],
            b"plain ascii",
            "añadir € 日本".as_bytes(),
            b"a\0b",
        ] {
            let d = detect(input);
            assert_eq!(
                (d.encoding, d.method),
                (Encoding::Utf8, DetectionMethod::Utf8Valid)
            );
        }
    }

    #[test]
    fn detect_heuristic_windows1252() {
        let text = "El niño comió ñoquis, camión y canción. Más información: ¿qué tal?";
        let (bytes, _, _) = encoding_rs::WINDOWS_1252.encode(text);
        let d = detect(&bytes);
        assert_eq!(d.encoding, Encoding::Windows1252);
        assert_eq!(d.method, DetectionMethod::Heuristic);
    }

    #[test]
    fn detect_fallback_for_unsupported() {
        let (bytes, _, _) = encoding_rs::WINDOWS_1251
            .encode("Привет, мир! Это обычный русский текст для проверки кодировки.");
        let d = detect(&bytes);
        assert_eq!(
            (d.encoding, d.method),
            (Encoding::Windows1252, DetectionMethod::Fallback)
        );
    }

    #[test]
    fn detect_never_ascii_or_latin1() {
        for input in [&[0xE9u8][..], &[0x80, 0x81], &[0xFF]] {
            let e = detect(input).encoding;
            assert!(e != Encoding::Ascii && e != Encoding::Iso8859_1);
        }
    }

    #[test]
    fn utf8_strict_keeps_bom() {
        assert_eq!(
            decode(&[0xEF, 0xBB, 0xBF, b'a'], Encoding::Utf8).unwrap(),
            "\u{FEFF}a"
        );
        assert_eq!(decode(b"", Encoding::Utf8).unwrap(), "");
        assert_eq!(decode(b"a\0b", Encoding::Utf8).unwrap(), "a\0b");
    }

    #[test]
    fn utf8_invalid_offsets() {
        assert_eq!(
            decode(b"ab\xFFc", Encoding::Utf8),
            Err(invalid(Encoding::Utf8, 2))
        );
        // Secuencia truncada al final.
        assert_eq!(
            decode(b"ab\xE2\x82", Encoding::Utf8),
            Err(invalid(Encoding::Utf8, 2))
        );
    }

    #[test]
    fn utf8_bom_variants() {
        assert_eq!(
            decode(&[0xEF, 0xBB, 0xBF, b'h'], Encoding::Utf8Bom).unwrap(),
            "h"
        );
        assert_eq!(decode(b"h", Encoding::Utf8Bom).unwrap(), "h");
        assert_eq!(decode(&[0xEF, 0xBB, 0xBF], Encoding::Utf8Bom).unwrap(), "");
        assert_eq!(
            decode(&[0xEF, 0xBB, 0xBF, b'a', 0xFF], Encoding::Utf8Bom),
            Err(invalid(Encoding::Utf8Bom, 4))
        );
        assert_eq!(
            decode(b"a\xFF", Encoding::Utf8Bom),
            Err(invalid(Encoding::Utf8Bom, 1))
        );
    }

    #[test]
    fn utf16_basic() {
        assert_eq!(
            decode(&[0xFF, 0xFE, b'h', 0, 0xE9, 0], Encoding::Utf16Le).unwrap(),
            "hé"
        );
        assert_eq!(
            decode(&[0xFE, 0xFF, 0, b'h', 0, 0xE9], Encoding::Utf16Be).unwrap(),
            "hé"
        );
        // Sin BOM también se decodifica.
        assert_eq!(decode(&[b'h', 0], Encoding::Utf16Le).unwrap(), "h");
        assert_eq!(decode(&[0, b'h'], Encoding::Utf16Be).unwrap(), "h");
        assert_eq!(decode(&[], Encoding::Utf16Le).unwrap(), "");
        assert_eq!(decode(&[0xFF, 0xFE], Encoding::Utf16Le).unwrap(), "");
        // Par sustituto válido (U+1F600).
        assert_eq!(
            decode(&[0x3D, 0xD8, 0x00, 0xDE], Encoding::Utf16Le).unwrap(),
            "\u{1F600}"
        );
    }

    #[test]
    fn utf16_other_endian_bom_is_a_char() {
        assert_eq!(
            decode(&[0xFF, 0xFE], Encoding::Utf16Be).unwrap(),
            "\u{FFFE}"
        );
        assert_eq!(
            decode(&[0xFE, 0xFF], Encoding::Utf16Le).unwrap(),
            "\u{FFFE}"
        );
    }

    #[test]
    fn utf16_odd_length() {
        let e = Encoding::Utf16Le;
        assert_eq!(
            decode(&[0xFF, 0xFE, 0x41], e),
            Err(DecodeError::OddLength { encoding: e })
        );
        assert_eq!(
            decode(&[0x41], e),
            Err(DecodeError::OddLength { encoding: e })
        );
    }

    #[test]
    fn utf16_lone_surrogates() {
        let e = Encoding::Utf16Le;
        // Alto sin pareja al final, tras BOM y un carácter: offset 4.
        assert_eq!(
            decode(&[0xFF, 0xFE, b'a', 0, 0x3D, 0xD8], e),
            Err(invalid(e, 4))
        );
        // Alto seguido de un no-bajo.
        assert_eq!(decode(&[0x3D, 0xD8, b'a', 0], e), Err(invalid(e, 0)));
        // Bajo suelto tras un par válido.
        assert_eq!(
            decode(&[0x3D, 0xD8, 0x00, 0xDE, 0x00, 0xDE], e),
            Err(invalid(e, 4))
        );
        let b = Encoding::Utf16Be;
        assert_eq!(decode(&[0xFE, 0xFF, 0xDC, 0x00], b), Err(invalid(b, 2)));
    }

    #[test]
    fn ascii() {
        assert_eq!(decode(b"abc\0", Encoding::Ascii).unwrap(), "abc\0");
        assert_eq!(
            decode(b"ab\x80", Encoding::Ascii),
            Err(invalid(Encoding::Ascii, 2))
        );
        assert_eq!(decode(b"", Encoding::Ascii).unwrap(), "");
    }

    #[test]
    fn latin1_is_real() {
        assert_eq!(
            decode(&[0x80, 0x9F, 0xE9, 0xA4], Encoding::Iso8859_1).unwrap(),
            "\u{80}\u{9F}é¤"
        );
        let all: Vec<u8> = (0..=255).collect();
        let s = decode(&all, Encoding::Iso8859_1).unwrap();
        assert_eq!(s.chars().count(), 256);
        assert!(s.chars().zip(0u32..).all(|(c, i)| c as u32 == i));
    }

    #[test]
    fn latin9_differences() {
        let s = decode(
            &[0xA4, 0xA6, 0xA8, 0xB4, 0xB8, 0xBC, 0xBD, 0xBE, 0xE9],
            Encoding::Iso8859_15,
        )
        .unwrap();
        assert_eq!(s, "€ŠšŽžŒœŸé");
    }

    #[test]
    fn windows1252_whatwg() {
        assert_eq!(decode(&[0x80, 0xE9], Encoding::Windows1252).unwrap(), "€é");
        assert_eq!(
            decode(&[0x81, 0x8D, 0x8F, 0x90, 0x9D], Encoding::Windows1252).unwrap(),
            "\u{81}\u{8D}\u{8F}\u{90}\u{9D}"
        );
        // El BOM UTF-8 no se interpreta, se decodifica como caracteres.
        assert_eq!(
            decode(&[0xEF, 0xBB, 0xBF], Encoding::Windows1252).unwrap(),
            "ï»¿"
        );
    }

    #[test]
    fn macroman() {
        assert_eq!(
            decode(&[0x8E, 0xA5, 0xDB], Encoding::MacRoman).unwrap(),
            "é•€"
        );
    }

    #[test]
    fn single_byte_never_fails() {
        let all: Vec<u8> = (0..=255).collect();
        for e in [
            Encoding::Iso8859_1,
            Encoding::Iso8859_15,
            Encoding::Windows1252,
            Encoding::MacRoman,
        ] {
            let s = decode(&all, e).unwrap();
            assert_eq!(s.chars().count(), 256);
            assert!(!s.contains('\u{FFFD}'));
        }
    }

    #[test]
    fn decode_auto_ok() {
        let d = decode_auto("héllo".as_bytes()).unwrap();
        assert_eq!(d.text, "héllo");
        assert_eq!(
            (d.encoding, d.method),
            (Encoding::Utf8, DetectionMethod::Utf8Valid)
        );
        let d = decode_auto(&[0xFF, 0xFE, b'h', 0]).unwrap();
        assert_eq!((d.text.as_str(), d.encoding), ("h", Encoding::Utf16Le));
        let d = decode_auto(&[0xE9]).unwrap();
        assert!(matches!(
            d.method,
            DetectionMethod::Heuristic | DetectionMethod::Fallback
        ));
        assert_eq!(d.text.chars().count(), 1);
    }

    #[test]
    fn decode_auto_errors_after_bom() {
        assert_eq!(
            decode_auto(&[0xFF, 0xFE, 0x41]),
            Err(DecodeError::OddLength {
                encoding: Encoding::Utf16Le
            })
        );
        assert_eq!(
            decode_auto(&[0xEF, 0xBB, 0xBF, 0xFF]),
            Err(invalid(Encoding::Utf8Bom, 3))
        );
        assert_eq!(decode_auto(b"").unwrap().text, "");
    }
}
