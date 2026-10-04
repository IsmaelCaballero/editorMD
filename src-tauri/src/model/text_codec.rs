//! **M09 `TextCodec`** (Modelo · Rust) — detección de la codificación y
//! decodificación de ficheros de texto.
//!
//! Convierte los bytes de un fichero en texto Unicode. Esta versión cubre solo
//! la lectura: [`detect`] decide la codificación (BOM, UTF-8 válido o
//! heurística con `chardetng`), [`decode`] decodifica de forma estricta con una
//! codificación concreta y [`decode_auto`] combina ambas.
//!
//! La decodificación nunca sustituye bytes por `U+FFFD`: o se decodifica toda
//! la entrada o se devuelve un [`DecodeError`] que indica dónde está el
//! problema.

use std::fmt;

use chardetng::{EncodingDetector, Iso2022JpDetection, Utf8Detection};
use serde::{Deserialize, Serialize};

/// BOM de UTF-8.
const BOM_UTF8: &[u8] = &[0xEF, 0xBB, 0xBF];
/// BOM de UTF-16 little-endian.
const BOM_UTF16_LE: &[u8] = &[0xFF, 0xFE];
/// BOM de UTF-16 big-endian.
const BOM_UTF16_BE: &[u8] = &[0xFE, 0xFF];

/// Codificaciones soportadas (D-15).
///
/// Se serializa y deserializa como su identificador estable ([`Encoding::id`]),
/// p. ej. `"utf-8-bom"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Encoding {
    /// UTF-8 sin BOM (id `"utf-8"`).
    #[serde(rename = "utf-8")]
    Utf8,
    /// UTF-8 con BOM `EF BB BF` (id `"utf-8-bom"`).
    #[serde(rename = "utf-8-bom")]
    Utf8Bom,
    /// UTF-16 little-endian (id `"utf-16le"`).
    #[serde(rename = "utf-16le")]
    Utf16Le,
    /// UTF-16 big-endian (id `"utf-16be"`).
    #[serde(rename = "utf-16be")]
    Utf16Be,
    /// ASCII de 7 bits (id `"ascii"`).
    #[serde(rename = "ascii")]
    Ascii,
    /// ISO-8859-1 (Latin-1) real (id `"iso-8859-1"`).
    #[serde(rename = "iso-8859-1")]
    Iso8859_1,
    /// ISO-8859-15 (Latin-9) (id `"iso-8859-15"`).
    #[serde(rename = "iso-8859-15")]
    Iso8859_15,
    /// Windows-1252 según WHATWG (id `"windows-1252"`).
    #[serde(rename = "windows-1252")]
    Windows1252,
    /// Mac OS Roman (id `"macintosh"`).
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

    /// Identificador estable (el mismo que usa el frontend en
    /// `src/model/document.ts`).
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
    ///
    /// # Examples
    ///
    /// ```
    /// use editormd_lib::model::text_codec::Encoding;
    ///
    /// assert_eq!(Encoding::from_id("macintosh"), Some(Encoding::MacRoman));
    /// assert_eq!(Encoding::from_id("UTF-8"), None);
    /// ```
    pub fn from_id(id: &str) -> Option<Encoding> {
        Encoding::ALL.into_iter().find(|e| e.id() == id)
    }

    /// BOM de la codificación: `EF BB BF` (`Utf8Bom`), `FF FE` (`Utf16Le`),
    /// `FE FF` (`Utf16Be`); vacío en las demás.
    pub fn bom(self) -> &'static [u8] {
        match self {
            Encoding::Utf8Bom => BOM_UTF8,
            Encoding::Utf16Le => BOM_UTF16_LE,
            Encoding::Utf16Be => BOM_UTF16_BE,
            Encoding::Utf8
            | Encoding::Ascii
            | Encoding::Iso8859_1
            | Encoding::Iso8859_15
            | Encoding::Windows1252
            | Encoding::MacRoman => &[],
        }
    }
}

/// Cómo se ha decidido la codificación.
///
/// Se serializa en kebab-case: `"bom"`, `"utf8-valid"`, `"heuristic"`,
/// `"fallback"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum DetectionMethod {
    /// La entrada empieza por un BOM reconocido.
    #[serde(rename = "bom")]
    Bom,
    /// La entrada es UTF-8 válido (incluidos el vacío y el ASCII puro).
    #[serde(rename = "utf8-valid")]
    Utf8Valid,
    /// La heurística de `chardetng` ha dado una codificación soportada.
    #[serde(rename = "heuristic")]
    Heuristic,
    /// La heurística ha dado una codificación no soportada; se usa
    /// windows-1252.
    #[serde(rename = "fallback")]
    Fallback,
}

/// Resultado de [`detect`]: la codificación y cómo se ha decidido.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Detection {
    /// Codificación detectada.
    pub encoding: Encoding,
    /// Método con el que se ha decidido.
    pub method: DetectionMethod,
}

/// Resultado de [`decode_auto`]: el texto junto con la detección usada.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Decoded {
    /// Texto decodificado (sin el BOM, si lo había).
    pub text: String,
    /// Codificación detectada y usada para decodificar.
    pub encoding: Encoding,
    /// Método con el que se ha decidido la codificación.
    pub method: DetectionMethod,
}

/// Error de decodificación estricta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// Secuencia no válida; `offset` = posición (en bytes, contando el BOM si
    /// lo hay) del primer byte de la primera secuencia no válida en la entrada
    /// original.
    InvalidSequence {
        /// Codificación con la que se decodificaba.
        encoding: Encoding,
        /// Posición en bytes del primer byte no válido.
        offset: usize,
    },
    /// UTF-16 con un número impar de bytes (tras quitar el BOM).
    OddLength {
        /// Codificación con la que se decodificaba.
        encoding: Encoding,
    },
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::InvalidSequence { encoding, offset } => write!(
                f,
                "secuencia de bytes no válida en {} en la posición {offset}",
                encoding.id()
            ),
            DecodeError::OddLength { encoding } => write!(
                f,
                "número impar de bytes para {}: falta un byte al final",
                encoding.id()
            ),
        }
    }
}

impl std::error::Error for DecodeError {}

/// Detecta la codificación de `bytes`.
///
/// Orden de decisión:
/// 1. **BOM**: `EF BB BF` → `Utf8Bom`, `FF FE` → `Utf16Le`, `FE FF` →
///    `Utf16Be` (método [`DetectionMethod::Bom`]).
/// 2. **UTF-8 válido** (incluidos el vacío y el ASCII puro) → `Utf8`
///    ([`DetectionMethod::Utf8Valid`]). Nunca devuelve `Ascii` ni `Iso8859_1`.
/// 3. **Heurística** con `chardetng` sobre toda la entrada: windows-1252,
///    ISO-8859-15 o macintosh ([`DetectionMethod::Heuristic`]).
/// 4. Cualquier otro resultado → `Windows1252`
///    ([`DetectionMethod::Fallback`]).
///
/// # Examples
///
/// ```
/// use editormd_lib::model::text_codec::{detect, DetectionMethod, Encoding};
///
/// let d = detect("¡Hola, señor!".as_bytes());
/// assert_eq!(d.encoding, Encoding::Utf8);
/// assert_eq!(d.method, DetectionMethod::Utf8Valid);
///
/// let d = detect(&[0xFF, 0xFE, b'h', 0x00]);
/// assert_eq!(d.encoding, Encoding::Utf16Le);
/// assert_eq!(d.method, DetectionMethod::Bom);
/// ```
pub fn detect(bytes: &[u8]) -> Detection {
    let by_bom = [Encoding::Utf8Bom, Encoding::Utf16Le, Encoding::Utf16Be]
        .into_iter()
        .find(|e| bytes.starts_with(e.bom()));
    if let Some(encoding) = by_bom {
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
    let mut detector = EncodingDetector::new(Iso2022JpDetection::Deny);
    detector.feed(bytes, true);
    from_heuristic_guess(detector.guess(None, Utf8Detection::Deny))
}

/// Traduce el resultado de la heurística a una [`Detection`].
///
/// `chardetng` no llega a proponer ISO-8859-15 ni macintosh en la práctica,
/// pero se traducen igualmente por si una versión futura lo hace.
fn from_heuristic_guess(guess: &'static encoding_rs::Encoding) -> Detection {
    let heuristic = |encoding| Detection {
        encoding,
        method: DetectionMethod::Heuristic,
    };
    if guess == encoding_rs::WINDOWS_1252 {
        heuristic(Encoding::Windows1252)
    } else if guess == encoding_rs::ISO_8859_15 {
        heuristic(Encoding::Iso8859_15)
    } else if guess == encoding_rs::MACINTOSH {
        heuristic(Encoding::MacRoman)
    } else {
        Detection {
            encoding: Encoding::Windows1252,
            method: DetectionMethod::Fallback,
        }
    }
}

/// Decodifica `bytes` con `encoding` de forma estricta, sin sustituir nada
/// por `U+FFFD`.
///
/// - `Utf8`: UTF-8 estricto; **no** quita un BOM inicial (queda `U+FEFF`).
/// - `Utf8Bom`: quita `EF BB BF` si está al principio; UTF-8 estricto.
/// - `Utf16Le` / `Utf16Be`: quita su propio BOM si está; un número impar de
///   bytes da [`DecodeError::OddLength`] y un sustituto sin pareja,
///   [`DecodeError::InvalidSequence`].
/// - `Ascii`: cualquier byte ≥ `0x80` es [`DecodeError::InvalidSequence`].
/// - `Iso8859_1` (Latin-1 real), `Iso8859_15`, `Windows1252` y `MacRoman`:
///   nunca fallan.
///
/// # Errors
///
/// Devuelve [`DecodeError::InvalidSequence`] con la posición (contando el
/// BOM) del primer byte no válido, o [`DecodeError::OddLength`] si una
/// entrada UTF-16 tiene un número impar de bytes tras quitar el BOM.
///
/// # Examples
///
/// ```
/// use editormd_lib::model::text_codec::{decode, DecodeError, Encoding};
///
/// assert_eq!(decode(&[0x63, 0x61, 0xF1, 0x61], Encoding::Windows1252).unwrap(), "caña");
/// assert_eq!(decode(&[0xEF, 0xBB, 0xBF, b'a'], Encoding::Utf8Bom).unwrap(), "a");
/// assert_eq!(
///     decode(&[b'a', 0xC3], Encoding::Utf8),
///     Err(DecodeError::InvalidSequence { encoding: Encoding::Utf8, offset: 1 })
/// );
/// ```
pub fn decode(bytes: &[u8], encoding: Encoding) -> Result<String, DecodeError> {
    match encoding {
        Encoding::Utf8 | Encoding::Utf8Bom => decode_utf8(bytes, encoding),
        Encoding::Utf16Le => decode_utf16(bytes, encoding, u16::from_le_bytes),
        Encoding::Utf16Be => decode_utf16(bytes, encoding, u16::from_be_bytes),
        Encoding::Ascii => decode_ascii(bytes),
        Encoding::Iso8859_1 => Ok(bytes.iter().copied().map(char::from).collect()),
        Encoding::Iso8859_15 => Ok(decode_single_byte(bytes, encoding_rs::ISO_8859_15)),
        Encoding::Windows1252 => Ok(decode_single_byte(bytes, encoding_rs::WINDOWS_1252)),
        Encoding::MacRoman => Ok(decode_single_byte(bytes, encoding_rs::MACINTOSH)),
    }
}

/// Detecta la codificación con [`detect`] y decodifica con [`decode`].
///
/// # Errors
///
/// Devuelve el error de [`decode`]. Solo puede ocurrir tras un BOM (p. ej.
/// `FF FE` seguido de un número impar de bytes, o `EF BB BF` seguido de UTF-8
/// no válido).
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

/// Quita el BOM de `encoding` del principio de `bytes`, si está.
///
/// Devuelve la longitud del BOM quitado (0 si no estaba) y el resto.
fn strip_bom(bytes: &[u8], encoding: Encoding) -> (usize, &[u8]) {
    let bom = encoding.bom();
    match bytes.strip_prefix(bom) {
        Some(rest) => (bom.len(), rest),
        None => (0, bytes),
    }
}

/// UTF-8 estricto; en `Utf8Bom` quita antes el BOM.
fn decode_utf8(bytes: &[u8], encoding: Encoding) -> Result<String, DecodeError> {
    let (skipped, body) = strip_bom(bytes, encoding);
    std::str::from_utf8(body)
        .map(str::to_owned)
        .map_err(|e| DecodeError::InvalidSequence {
            encoding,
            offset: skipped + e.valid_up_to(),
        })
}

/// UTF-16 estricto con la endianness que indica `to_unit`.
fn decode_utf16(
    bytes: &[u8],
    encoding: Encoding,
    to_unit: fn([u8; 2]) -> u16,
) -> Result<String, DecodeError> {
    let (skipped, body) = strip_bom(bytes, encoding);
    if body.len() % 2 != 0 {
        return Err(DecodeError::OddLength { encoding });
    }
    let (pairs, _) = body.as_chunks::<2>();
    let units = pairs.iter().map(|&pair| to_unit(pair));
    let mut text = String::with_capacity(body.len());
    // Posición, en unidades de 16 bits, del siguiente carácter por decodificar.
    let mut unit_index = 0;
    for decoded in char::decode_utf16(units) {
        match decoded {
            Ok(ch) => {
                text.push(ch);
                unit_index += ch.len_utf16();
            }
            Err(_) => {
                return Err(DecodeError::InvalidSequence {
                    encoding,
                    offset: skipped + unit_index * 2,
                });
            }
        }
    }
    Ok(text)
}

/// ASCII de 7 bits: falla en el primer byte ≥ `0x80`.
fn decode_ascii(bytes: &[u8]) -> Result<String, DecodeError> {
    match bytes.iter().position(|b| !b.is_ascii()) {
        Some(offset) => Err(DecodeError::InvalidSequence {
            encoding: Encoding::Ascii,
            offset,
        }),
        None => Ok(bytes.iter().copied().map(char::from).collect()),
    }
}

/// Codificaciones de un byte de `encoding_rs` que asignan carácter a los 256
/// bytes, así que nunca hay sustituciones.
fn decode_single_byte(bytes: &[u8], encoding: &'static encoding_rs::Encoding) -> String {
    encoding.decode_without_bom_handling(bytes).0.into_owned()
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

    fn with_prefix(prefix: &[u8], rest: &[u8]) -> Vec<u8> {
        let mut v = prefix.to_vec();
        v.extend_from_slice(rest);
        v
    }

    fn encode_with(encoding: &'static encoding_rs::Encoding, s: &str) -> Vec<u8> {
        let (bytes, _, had_errors) = encoding.encode(s);
        assert!(!had_errors, "no se puede codificar en {}", encoding.name());
        bytes.into_owned()
    }

    fn invalid(encoding: Encoding, offset: usize) -> DecodeError {
        DecodeError::InvalidSequence { encoding, offset }
    }

    // --- Encoding -------------------------------------------------------

    #[test]
    fn all_lists_the_nine_encodings_in_declaration_order() {
        assert_eq!(
            Encoding::ALL,
            [
                Encoding::Utf8,
                Encoding::Utf8Bom,
                Encoding::Utf16Le,
                Encoding::Utf16Be,
                Encoding::Ascii,
                Encoding::Iso8859_1,
                Encoding::Iso8859_15,
                Encoding::Windows1252,
                Encoding::MacRoman,
            ]
        );
    }

    #[test]
    fn ids_are_the_frontend_ones() {
        let ids: Vec<&str> = Encoding::ALL.iter().map(|e| e.id()).collect();
        assert_eq!(
            ids,
            [
                "utf-8",
                "utf-8-bom",
                "utf-16le",
                "utf-16be",
                "ascii",
                "iso-8859-1",
                "iso-8859-15",
                "windows-1252",
                "macintosh",
            ]
        );
    }

    #[test]
    fn from_id_is_the_inverse_of_id() {
        for e in Encoding::ALL {
            assert_eq!(Encoding::from_id(e.id()), Some(e));
        }
    }

    #[test]
    fn from_id_rejects_unknown_and_wrong_case() {
        for id in [
            "",
            "UTF-8",
            "utf8",
            "latin1",
            "Macintosh",
            " utf-8",
            "utf-32",
        ] {
            assert_eq!(Encoding::from_id(id), None, "{id:?}");
        }
    }

    #[test]
    fn bom_bytes() {
        assert_eq!(Encoding::Utf8Bom.bom(), [0xEF, 0xBB, 0xBF]);
        assert_eq!(Encoding::Utf16Le.bom(), [0xFF, 0xFE]);
        assert_eq!(Encoding::Utf16Be.bom(), [0xFE, 0xFF]);
        for e in [
            Encoding::Utf8,
            Encoding::Ascii,
            Encoding::Iso8859_1,
            Encoding::Iso8859_15,
            Encoding::Windows1252,
            Encoding::MacRoman,
        ] {
            assert!(e.bom().is_empty(), "{e:?}");
        }
    }

    #[test]
    fn encoding_serializes_as_its_id_and_back() {
        for e in Encoding::ALL {
            let json = serde_json::to_string(&e).unwrap();
            assert_eq!(json, format!("\"{}\"", e.id()));
            let back: Encoding = serde_json::from_str(&json).unwrap();
            assert_eq!(back, e);
        }
    }

    #[test]
    fn encoding_deserialization_rejects_unknown_ids() {
        assert!(serde_json::from_str::<Encoding>("\"Utf8\"").is_err());
        assert!(serde_json::from_str::<Encoding>("\"UTF-8\"").is_err());
    }

    #[test]
    fn detection_method_serializes_in_kebab_case() {
        let cases = [
            (DetectionMethod::Bom, "\"bom\""),
            (DetectionMethod::Utf8Valid, "\"utf8-valid\""),
            (DetectionMethod::Heuristic, "\"heuristic\""),
            (DetectionMethod::Fallback, "\"fallback\""),
        ];
        for (m, json) in cases {
            assert_eq!(serde_json::to_string(&m).unwrap(), json);
        }
    }

    #[test]
    fn detection_and_decoded_serialize_as_camel_case_json() {
        let d = Detection {
            encoding: Encoding::Utf16Be,
            method: DetectionMethod::Bom,
        };
        assert_eq!(
            serde_json::to_string(&d).unwrap(),
            r#"{"encoding":"utf-16be","method":"bom"}"#
        );
        let d = Decoded {
            text: "hola".into(),
            encoding: Encoding::Windows1252,
            method: DetectionMethod::Heuristic,
        };
        assert_eq!(
            serde_json::to_string(&d).unwrap(),
            r#"{"text":"hola","encoding":"windows-1252","method":"heuristic"}"#
        );
    }

    // --- DecodeError ----------------------------------------------------

    #[test]
    fn decode_error_display_mentions_id_and_offset() {
        let msg = invalid(Encoding::Utf16Be, 42).to_string();
        assert!(msg.contains("utf-16be"), "{msg}");
        assert!(msg.contains("42"), "{msg}");
        let msg = DecodeError::OddLength {
            encoding: Encoding::Utf16Le,
        }
        .to_string();
        assert!(msg.contains("utf-16le"), "{msg}");
    }

    #[test]
    fn decode_error_is_a_std_error() {
        let e: Box<dyn std::error::Error> = Box::new(invalid(Encoding::Ascii, 0));
        assert!(e.source().is_none());
    }

    // --- detect ---------------------------------------------------------

    fn assert_detects(bytes: &[u8], encoding: Encoding, method: DetectionMethod) {
        assert_eq!(
            detect(bytes),
            Detection { encoding, method },
            "{bytes:02X?}"
        );
    }

    #[test]
    fn detect_boms() {
        use DetectionMethod::Bom;
        assert_detects(&[0xEF, 0xBB, 0xBF], Encoding::Utf8Bom, Bom);
        assert_detects(&[0xEF, 0xBB, 0xBF, b'a'], Encoding::Utf8Bom, Bom);
        assert_detects(&[0xFF, 0xFE], Encoding::Utf16Le, Bom);
        assert_detects(&[0xFF, 0xFE, b'a', 0], Encoding::Utf16Le, Bom);
        assert_detects(&[0xFE, 0xFF], Encoding::Utf16Be, Bom);
        assert_detects(&[0xFE, 0xFF, 0, b'a'], Encoding::Utf16Be, Bom);
    }

    #[test]
    fn detect_bom_wins_even_if_the_rest_is_invalid() {
        use DetectionMethod::Bom;
        assert_detects(&[0xEF, 0xBB, 0xBF, 0xFF], Encoding::Utf8Bom, Bom);
        assert_detects(&[0xFF, 0xFE, 0x41], Encoding::Utf16Le, Bom);
        // UTF-32LE se detecta como UTF-16LE (no se distingue UTF-32).
        assert_detects(&[0xFF, 0xFE, 0, 0], Encoding::Utf16Le, Bom);
    }

    #[test]
    fn detect_partial_boms_are_not_boms() {
        // EF BB sin BF no es UTF-8 válido: decide la heurística.
        assert_ne!(detect(&[0xEF, 0xBB]).method, DetectionMethod::Bom);
        assert_ne!(detect(&[0xFF]).method, DetectionMethod::Bom);
        assert_ne!(detect(&[0xFE]).method, DetectionMethod::Bom);
    }

    #[test]
    fn detect_empty_ascii_and_utf8_as_utf8() {
        use DetectionMethod::Utf8Valid;
        assert_detects(b"", Encoding::Utf8, Utf8Valid);
        assert_detects(b"# Hola\n\nmundo", Encoding::Utf8, Utf8Valid);
        assert_detects(&[0, 0, 0], Encoding::Utf8, Utf8Valid);
        assert_detects("Ñandú, €, 日本, 😀".as_bytes(), Encoding::Utf8, Utf8Valid);
    }

    #[test]
    fn detect_windows_1252_spanish_text() {
        let bytes = encode_with(
            encoding_rs::WINDOWS_1252,
            "El niño comió una piña en la montaña. ¿Qué tal estás? «Así» es la canción del año.",
        );
        assert_detects(&bytes, Encoding::Windows1252, DetectionMethod::Heuristic);
    }

    #[test]
    fn detect_single_high_byte_is_heuristic_windows_1252() {
        assert_detects(&[0xE9], Encoding::Windows1252, DetectionMethod::Heuristic);
    }

    #[test]
    fn detect_cyrillic_falls_back_to_windows_1252() {
        let bytes = encode_with(
            encoding_rs::WINDOWS_1251,
            "Привет! Это проверка определения кодировки текста на русском языке.",
        );
        assert_detects(&bytes, Encoding::Windows1252, DetectionMethod::Fallback);
    }

    #[test]
    fn detect_shift_jis_falls_back_to_windows_1252() {
        let bytes = encode_with(
            encoding_rs::SHIFT_JIS,
            "これは日本語のテキストです。文字コードの判定を確認します。",
        );
        assert_detects(&bytes, Encoding::Windows1252, DetectionMethod::Fallback);
    }

    #[test]
    fn heuristic_guess_mapping() {
        let h = DetectionMethod::Heuristic;
        let cases = [
            (encoding_rs::WINDOWS_1252, Encoding::Windows1252, h),
            (encoding_rs::ISO_8859_15, Encoding::Iso8859_15, h),
            (encoding_rs::MACINTOSH, Encoding::MacRoman, h),
            (
                encoding_rs::WINDOWS_1251,
                Encoding::Windows1252,
                DetectionMethod::Fallback,
            ),
            (
                encoding_rs::SHIFT_JIS,
                Encoding::Windows1252,
                DetectionMethod::Fallback,
            ),
            (
                encoding_rs::UTF_8,
                Encoding::Windows1252,
                DetectionMethod::Fallback,
            ),
        ];
        for (guess, encoding, method) in cases {
            assert_eq!(
                from_heuristic_guess(guess),
                Detection { encoding, method },
                "{}",
                guess.name()
            );
        }
    }

    #[test]
    fn detect_never_returns_ascii_or_latin1() {
        let samples: [&[u8]; 5] = [b"", b"abc", &[0x80], &[0xA0, 0xFF], &[0xC3]];
        for s in samples {
            let e = detect(s).encoding;
            assert!(e != Encoding::Ascii && e != Encoding::Iso8859_1, "{s:02X?}");
        }
    }

    // --- decode: UTF-8 --------------------------------------------------

    #[test]
    fn decode_utf8_valid() {
        assert_eq!(decode(b"", Encoding::Utf8).unwrap(), "");
        assert_eq!(decode("añ€😀".as_bytes(), Encoding::Utf8).unwrap(), "añ€😀");
        assert_eq!(decode(&[b'a', 0, b'b'], Encoding::Utf8).unwrap(), "a\0b");
    }

    #[test]
    fn decode_utf8_keeps_a_leading_bom() {
        assert_eq!(
            decode(&[0xEF, 0xBB, 0xBF, b'x'], Encoding::Utf8).unwrap(),
            "\u{FEFF}x"
        );
    }

    #[test]
    fn decode_utf8_reports_offset_of_first_invalid_sequence() {
        assert_eq!(
            decode(&[0xFF], Encoding::Utf8),
            Err(invalid(Encoding::Utf8, 0))
        );
        assert_eq!(
            decode(&[b'a', b'b', 0xC3, b'(', 0xFF], Encoding::Utf8),
            Err(invalid(Encoding::Utf8, 2))
        );
        // Secuencia truncada al final.
        assert_eq!(
            decode(&[b'a', 0xE2, 0x82], Encoding::Utf8),
            Err(invalid(Encoding::Utf8, 1))
        );
        // Sustituto codificado en UTF-8 (CESU): no válido.
        assert_eq!(
            decode(&[0xED, 0xA0, 0x80], Encoding::Utf8),
            Err(invalid(Encoding::Utf8, 0))
        );
        // Forma demasiado larga.
        assert_eq!(
            decode(&[b'x', 0xC0, 0xAF], Encoding::Utf8),
            Err(invalid(Encoding::Utf8, 1))
        );
    }

    #[test]
    fn decode_utf8_bom_strips_the_bom() {
        assert_eq!(
            decode(&[0xEF, 0xBB, 0xBF, b'h', b'i'], Encoding::Utf8Bom).unwrap(),
            "hi"
        );
        assert_eq!(decode(&[0xEF, 0xBB, 0xBF], Encoding::Utf8Bom).unwrap(), "");
        // Solo se quita un BOM.
        assert_eq!(
            decode(&[0xEF, 0xBB, 0xBF, 0xEF, 0xBB, 0xBF], Encoding::Utf8Bom).unwrap(),
            "\u{FEFF}"
        );
    }

    #[test]
    fn decode_utf8_bom_without_bom_decodes_anyway() {
        assert_eq!(decode(b"hola", Encoding::Utf8Bom).unwrap(), "hola");
        assert_eq!(decode(b"", Encoding::Utf8Bom).unwrap(), "");
        assert_eq!(
            decode(&[b'a', 0xFF], Encoding::Utf8Bom),
            Err(invalid(Encoding::Utf8Bom, 1))
        );
    }

    #[test]
    fn decode_utf8_bom_offset_counts_the_bom() {
        assert_eq!(
            decode(&[0xEF, 0xBB, 0xBF, b'a', 0x80], Encoding::Utf8Bom),
            Err(invalid(Encoding::Utf8Bom, 4))
        );
        // BOM incompleto: no se quita y falla en su primer byte.
        assert_eq!(
            decode(&[0xEF, 0xBB, b'a'], Encoding::Utf8Bom),
            Err(invalid(Encoding::Utf8Bom, 0))
        );
    }

    // --- decode: UTF-16 -------------------------------------------------

    #[test]
    fn decode_utf16_without_bom() {
        let s = "Hola, señor € 😀\0";
        assert_eq!(decode(&utf16le(s), Encoding::Utf16Le).unwrap(), s);
        assert_eq!(decode(&utf16be(s), Encoding::Utf16Be).unwrap(), s);
        assert_eq!(decode(b"", Encoding::Utf16Le).unwrap(), "");
        assert_eq!(decode(b"", Encoding::Utf16Be).unwrap(), "");
    }

    #[test]
    fn decode_utf16_strips_its_own_bom() {
        let s = "añ😀";
        let le = with_prefix(&[0xFF, 0xFE], &utf16le(s));
        let be = with_prefix(&[0xFE, 0xFF], &utf16be(s));
        assert_eq!(decode(&le, Encoding::Utf16Le).unwrap(), s);
        assert_eq!(decode(&be, Encoding::Utf16Be).unwrap(), s);
        assert_eq!(decode(&[0xFF, 0xFE], Encoding::Utf16Le).unwrap(), "");
        assert_eq!(decode(&[0xFE, 0xFF], Encoding::Utf16Be).unwrap(), "");
    }

    #[test]
    fn decode_utf16_keeps_the_other_endianness_bom() {
        // FE FF leído como LE es U+FFFE (no carácter, pero escalar válido).
        let le = with_prefix(&[0xFE, 0xFF], &utf16le("a"));
        assert_eq!(decode(&le, Encoding::Utf16Le).unwrap(), "\u{FFFE}a");
        let be = with_prefix(&[0xFF, 0xFE], &utf16be("a"));
        assert_eq!(decode(&be, Encoding::Utf16Be).unwrap(), "\u{FFFE}a");
    }

    #[test]
    fn decode_utf16_odd_length() {
        let odd = DecodeError::OddLength {
            encoding: Encoding::Utf16Le,
        };
        assert_eq!(decode(&[0x41], Encoding::Utf16Le), Err(odd.clone()));
        assert_eq!(
            decode(&[0x41, 0, 0x42], Encoding::Utf16Le),
            Err(odd.clone())
        );
        assert_eq!(decode(&[0xFF, 0xFE, 0x41], Encoding::Utf16Le), Err(odd));
        assert_eq!(
            decode(&[0xFE, 0xFF, 0, 0x41, 0], Encoding::Utf16Be),
            Err(DecodeError::OddLength {
                encoding: Encoding::Utf16Be
            })
        );
    }

    #[test]
    fn decode_utf16_odd_length_takes_precedence_over_invalid_units() {
        // Sustituto suelto y además longitud impar: se informa OddLength.
        assert_eq!(
            decode(&[0x00, 0xD8, 0x41], Encoding::Utf16Le),
            Err(DecodeError::OddLength {
                encoding: Encoding::Utf16Le
            })
        );
    }

    #[test]
    fn decode_utf16_lone_high_surrogate() {
        // 'a', D800, 'b' en LE: el sustituto alto está en el byte 2.
        let le = [0x61, 0x00, 0x00, 0xD8, 0x62, 0x00];
        assert_eq!(
            decode(&le, Encoding::Utf16Le),
            Err(invalid(Encoding::Utf16Le, 2))
        );
        // Sustituto alto al final.
        let be = [0x00, 0x61, 0xDB, 0xFF];
        assert_eq!(
            decode(&be, Encoding::Utf16Be),
            Err(invalid(Encoding::Utf16Be, 2))
        );
        // Dos sustitutos altos seguidos: falla el primero.
        let le = [0x00, 0xD8, 0x00, 0xD8, 0x00, 0xDC];
        assert_eq!(
            decode(&le, Encoding::Utf16Le),
            Err(invalid(Encoding::Utf16Le, 0))
        );
    }

    #[test]
    fn decode_utf16_lone_low_surrogate() {
        let be = [0x00, 0x61, 0x00, 0x62, 0xDC, 0x00];
        assert_eq!(
            decode(&be, Encoding::Utf16Be),
            Err(invalid(Encoding::Utf16Be, 4))
        );
    }

    #[test]
    fn decode_utf16_offset_counts_the_bom_and_pairs() {
        // BOM + 😀 (pareja válida, 4 bytes) + DC00 suelto → offset 2 + 4 = 6.
        let mut le = vec![0xFF, 0xFE];
        le.extend(utf16le("😀"));
        le.extend([0x00, 0xDC]);
        assert_eq!(
            decode(&le, Encoding::Utf16Le),
            Err(invalid(Encoding::Utf16Le, 6))
        );
        let mut be = vec![0xFE, 0xFF];
        be.extend(utf16be("x😀"));
        be.extend([0xD8, 0x3D]);
        assert_eq!(
            decode(&be, Encoding::Utf16Be),
            Err(invalid(Encoding::Utf16Be, 8))
        );
    }

    // --- decode: ASCII --------------------------------------------------

    #[test]
    fn decode_ascii() {
        assert_eq!(decode(b"", Encoding::Ascii).unwrap(), "");
        assert_eq!(
            decode(b"plain \x00\x7F text", Encoding::Ascii).unwrap(),
            "plain \0\u{7F} text"
        );
        assert_eq!(
            decode(&[b'a', b'b', 0x80, 0xFF], Encoding::Ascii),
            Err(invalid(Encoding::Ascii, 2))
        );
        assert_eq!(
            decode(&[0xEF, 0xBB, 0xBF], Encoding::Ascii),
            Err(invalid(Encoding::Ascii, 0))
        );
    }

    // --- decode: codificaciones de un byte --------------------------------

    fn all_bytes() -> Vec<u8> {
        (0..=255u8).collect()
    }

    #[test]
    fn decode_iso_8859_1_is_real_latin1() {
        let text = decode(&all_bytes(), Encoding::Iso8859_1).unwrap();
        let expected: String = (0..=255u8).map(char::from).collect();
        assert_eq!(text, expected);
        // Los controles C1 se conservan (no son los de windows-1252).
        assert_eq!(
            decode(&[0x80, 0x9F, 0xA4], Encoding::Iso8859_1).unwrap(),
            "\u{80}\u{9F}\u{A4}"
        );
    }

    #[test]
    fn decode_iso_8859_15_differs_from_latin1_in_eight_positions() {
        let pairs = [
            (0xA4, '€'),
            (0xA6, 'Š'),
            (0xA8, 'š'),
            (0xB4, 'Ž'),
            (0xB8, 'ž'),
            (0xBC, 'Œ'),
            (0xBD, 'œ'),
            (0xBE, 'Ÿ'),
        ];
        let text: Vec<char> = decode(&all_bytes(), Encoding::Iso8859_15)
            .unwrap()
            .chars()
            .collect();
        assert_eq!(text.len(), 256);
        for (b, &ch) in (0..=255u8).zip(&text) {
            let expected = pairs
                .iter()
                .find(|(pb, _)| *pb == b)
                .map_or(char::from(b), |&(_, c)| c);
            assert_eq!(ch, expected, "byte {b:02X}");
        }
    }

    #[test]
    fn decode_windows_1252() {
        assert_eq!(
            decode(&[0x80, 0x93, 0x94, 0xF1], Encoding::Windows1252).unwrap(),
            "€“”ñ"
        );
        // Bytes sin asignar en WHATWG: se convierten en los controles C1.
        assert_eq!(
            decode(&[0x81, 0x8D, 0x8F, 0x90, 0x9D], Encoding::Windows1252).unwrap(),
            "\u{81}\u{8D}\u{8F}\u{90}\u{9D}"
        );
        assert_eq!(
            decode(&all_bytes(), Encoding::Windows1252)
                .unwrap()
                .chars()
                .count(),
            256
        );
    }

    #[test]
    fn decode_mac_roman() {
        assert_eq!(
            decode(&[0x8E, 0x96, 0xA5, 0xDB, 0xF0], Encoding::MacRoman).unwrap(),
            "éñ•€\u{F8FF}"
        );
        assert_eq!(
            decode(&all_bytes(), Encoding::MacRoman)
                .unwrap()
                .chars()
                .count(),
            256
        );
    }

    #[test]
    fn decode_single_byte_encodings_never_fail_and_keep_bom_bytes() {
        let bytes = all_bytes();
        for e in [
            Encoding::Iso8859_1,
            Encoding::Iso8859_15,
            Encoding::Windows1252,
            Encoding::MacRoman,
        ] {
            let text = decode(&bytes, e).unwrap();
            assert!(!text.contains('\u{FFFD}'), "{e:?}");
            assert!(text.starts_with('\0'), "{e:?}");
        }
        // Un BOM UTF-8 delante no se interpreta: son tres caracteres.
        assert_eq!(
            decode(&[0xEF, 0xBB, 0xBF], Encoding::Windows1252).unwrap(),
            "ï»¿"
        );
        assert_eq!(decode(&[], Encoding::MacRoman).unwrap(), "");
    }

    #[test]
    fn decode_never_produces_replacement_characters() {
        let bytes = [0xC3, 0x28, 0xFF, 0x00, 0xD8];
        for e in Encoding::ALL {
            if let Ok(text) = decode(&bytes, e) {
                assert!(!text.contains('\u{FFFD}'), "{e:?}");
            }
        }
    }

    // --- decode_auto ----------------------------------------------------

    #[test]
    fn decode_auto_utf8_and_empty() {
        assert_eq!(
            decode_auto(b"").unwrap(),
            Decoded {
                text: String::new(),
                encoding: Encoding::Utf8,
                method: DetectionMethod::Utf8Valid,
            }
        );
        assert_eq!(
            decode_auto("cañón".as_bytes()).unwrap(),
            Decoded {
                text: "cañón".into(),
                encoding: Encoding::Utf8,
                method: DetectionMethod::Utf8Valid,
            }
        );
    }

    #[test]
    fn decode_auto_with_boms() {
        let d = decode_auto(&with_prefix(&[0xEF, 0xBB, 0xBF], "ñ".as_bytes())).unwrap();
        assert_eq!(
            d,
            Decoded {
                text: "ñ".into(),
                encoding: Encoding::Utf8Bom,
                method: DetectionMethod::Bom,
            }
        );
        let d = decode_auto(&with_prefix(&[0xFF, 0xFE], &utf16le("hé😀"))).unwrap();
        assert_eq!((d.text.as_str(), d.encoding), ("hé😀", Encoding::Utf16Le));
        let d = decode_auto(&with_prefix(&[0xFE, 0xFF], &utf16be("hé😀"))).unwrap();
        assert_eq!((d.text.as_str(), d.encoding), ("hé😀", Encoding::Utf16Be));
        assert_eq!(d.method, DetectionMethod::Bom);
    }

    #[test]
    fn decode_auto_heuristic_and_fallback() {
        let s =
            "El niño comió una piña en la montaña. ¿Qué tal estás? «Así» es la canción del año.";
        let d = decode_auto(&encode_with(encoding_rs::WINDOWS_1252, s)).unwrap();
        assert_eq!(
            d,
            Decoded {
                text: s.into(),
                encoding: Encoding::Windows1252,
                method: DetectionMethod::Heuristic,
            }
        );
        let ru = encode_with(
            encoding_rs::WINDOWS_1251,
            "Привет! Это проверка определения кодировки текста на русском языке.",
        );
        let d = decode_auto(&ru).unwrap();
        assert_eq!(d.encoding, Encoding::Windows1252);
        assert_eq!(d.method, DetectionMethod::Fallback);
        assert_eq!(d.text, decode(&ru, Encoding::Windows1252).unwrap());
    }

    #[test]
    fn decode_auto_errors_after_a_bom() {
        assert_eq!(
            decode_auto(&[0xFF, 0xFE, 0x41]),
            Err(DecodeError::OddLength {
                encoding: Encoding::Utf16Le
            })
        );
        assert_eq!(
            decode_auto(&[0xEF, 0xBB, 0xBF, b'a', 0xFF]),
            Err(invalid(Encoding::Utf8Bom, 4))
        );
        assert_eq!(
            decode_auto(&[0xFE, 0xFF, 0xDC, 0x00]),
            Err(invalid(Encoding::Utf16Be, 2))
        );
    }
}
