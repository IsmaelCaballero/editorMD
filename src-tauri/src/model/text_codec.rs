//! **M09 `TextCodec`** (Modelo · Rust) — detección de la codificación y
//! decodificación de ficheros de texto.
//!
//! Convierte los bytes de un fichero en texto Unicode de forma **estricta**:
//! o se decodifica todo sin pérdidas o se devuelve un [`DecodeError`] que
//! indica dónde está el problema. Nunca se sustituye nada por `U+FFFD`.
//!
//! Las codificaciones soportadas son las de la decisión D-15 (ver
//! [`Encoding`]). La escritura, los finales de línea y el informe de
//! pérdidas pertenecen a otras partes del componente.

use std::fmt;

use serde::{Deserialize, Serialize};

/// BOM de UTF-8.
const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];
/// BOM de UTF-16 little-endian.
const UTF16LE_BOM: &[u8] = &[0xFF, 0xFE];
/// BOM de UTF-16 big-endian.
const UTF16BE_BOM: &[u8] = &[0xFE, 0xFF];

/// Codificaciones soportadas (D-15).
///
/// Se serializa y deserializa con serde como su identificador estable
/// ([`Encoding::id`]), p. ej. `"utf-8-bom"`, el mismo que usa el frontend en
/// `src/model/document.ts`.
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
    /// ISO-8859-1 / Latin-1 real, byte a byte (id `"iso-8859-1"`).
    #[serde(rename = "iso-8859-1")]
    Iso8859_1,
    /// ISO-8859-15 / Latin-9 (id `"iso-8859-15"`).
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
    /// assert_eq!(Encoding::MacRoman.id(), "macintosh");
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
    /// assert_eq!(Encoding::from_id("utf-16le"), Some(Encoding::Utf16Le));
    /// assert_eq!(Encoding::from_id("UTF-8"), None);
    /// ```
    pub fn from_id(id: &str) -> Option<Encoding> {
        Encoding::ALL.into_iter().find(|e| e.id() == id)
    }

    /// BOM de la codificación: `EF BB BF` ([`Encoding::Utf8Bom`]), `FF FE`
    /// ([`Encoding::Utf16Le`]), `FE FF` ([`Encoding::Utf16Be`]); vacío en las
    /// demás.
    pub fn bom(self) -> &'static [u8] {
        match self {
            Encoding::Utf8Bom => UTF8_BOM,
            Encoding::Utf16Le => UTF16LE_BOM,
            Encoding::Utf16Be => UTF16BE_BOM,
            _ => &[],
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
    /// La heurística estadística (`chardetng`) ha dado una codificación
    /// soportada.
    #[serde(rename = "heuristic")]
    Heuristic,
    /// La heurística ha dado una codificación no soportada y se usa
    /// [`Encoding::Windows1252`] por defecto.
    #[serde(rename = "fallback")]
    Fallback,
}

/// Resultado de [`detect`]: codificación elegida y cómo se ha decidido.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Detection {
    /// Codificación detectada.
    pub encoding: Encoding,
    /// Método con el que se ha decidido.
    pub method: DetectionMethod,
}

/// Resultado de [`decode_auto`]: texto decodificado y datos de la detección.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Decoded {
    /// Texto Unicode (sin el BOM, si lo había).
    pub text: String,
    /// Codificación con la que se ha decodificado.
    pub encoding: Encoding,
    /// Método con el que se ha detectado la codificación.
    pub method: DetectionMethod,
}

/// Error de decodificación estricta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// Secuencia no válida; `offset` = posición (en bytes, contando el BOM si
    /// lo hay) del primer byte de la primera secuencia no válida en la entrada
    /// original.
    InvalidSequence {
        /// Codificación con la que se intentaba decodificar.
        encoding: Encoding,
        /// Posición en bytes del primer byte no válido en la entrada original.
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
        match self {
            DecodeError::InvalidSequence { encoding, offset } => write!(
                f,
                "secuencia no válida en {} en la posición {offset} (bytes)",
                encoding.id()
            ),
            DecodeError::OddLength { encoding } => {
                write!(f, "número impar de bytes en un texto {}", encoding.id())
            }
        }
    }
}

impl std::error::Error for DecodeError {}

/// Detecta la codificación de `bytes`.
///
/// Orden: BOM → UTF-8 válido → heurística (`chardetng`) → `windows-1252`
/// por defecto. Nunca devuelve [`Encoding::Ascii`] ni
/// [`Encoding::Iso8859_1`].
///
/// # Examples
///
/// ```
/// use editormd_lib::model::text_codec::{detect, DetectionMethod, Encoding};
///
/// let d = detect("¡Hola!".as_bytes());
/// assert_eq!(d.encoding, Encoding::Utf8);
/// assert_eq!(d.method, DetectionMethod::Utf8Valid);
///
/// let d = detect(&[0xFF, 0xFE, b'a', 0x00]);
/// assert_eq!(d.encoding, Encoding::Utf16Le);
/// assert_eq!(d.method, DetectionMethod::Bom);
/// ```
pub fn detect(bytes: &[u8]) -> Detection {
    // 1. BOM.
    for encoding in [Encoding::Utf8Bom, Encoding::Utf16Le, Encoding::Utf16Be] {
        if bytes.starts_with(encoding.bom()) {
            return Detection {
                encoding,
                method: DetectionMethod::Bom,
            };
        }
    }
    // 2. UTF-8 válido (incluye el vacío y el ASCII puro).
    if std::str::from_utf8(bytes).is_ok() {
        return Detection {
            encoding: Encoding::Utf8,
            method: DetectionMethod::Utf8Valid,
        };
    }
    // 3-4. Heurística sobre toda la entrada, sin TLD y sin permitir UTF-8.
    let mut detector = chardetng::EncodingDetector::new(chardetng::Iso2022JpDetection::Deny);
    detector.feed(bytes, true);
    map_guess(detector.guess(None, chardetng::Utf8Detection::Deny))
}

/// Decodifica `bytes` con `encoding` de forma estricta.
///
/// # Errors
///
/// - [`DecodeError::InvalidSequence`] si hay una secuencia no válida en la
///   codificación (UTF-8 mal formado, sustituto UTF-16 sin pareja o byte
///   `>= 0x80` en ASCII).
/// - [`DecodeError::OddLength`] si una entrada UTF-16 tiene un número impar
///   de bytes tras quitar el BOM.
///
/// # Examples
///
/// ```
/// use editormd_lib::model::text_codec::{decode, DecodeError, Encoding};
///
/// assert_eq!(decode(&[0x63, 0xE9], Encoding::Iso8859_1).unwrap(), "cé");
/// assert_eq!(
///     decode(&[b'a', 0xE9], Encoding::Utf8),
///     Err(DecodeError::InvalidSequence { encoding: Encoding::Utf8, offset: 1 })
/// );
/// ```
pub fn decode(bytes: &[u8], encoding: Encoding) -> Result<String, DecodeError> {
    match encoding {
        Encoding::Utf8 => decode_utf8(bytes, encoding, 0),
        Encoding::Utf8Bom => match bytes.strip_prefix(UTF8_BOM) {
            Some(rest) => decode_utf8(rest, encoding, UTF8_BOM.len()),
            None => decode_utf8(bytes, encoding, 0),
        },
        Encoding::Utf16Le | Encoding::Utf16Be => decode_utf16(bytes, encoding),
        Encoding::Ascii => match bytes.iter().position(|b| !b.is_ascii()) {
            Some(offset) => Err(DecodeError::InvalidSequence { encoding, offset }),
            None => Ok(bytes.iter().copied().map(char::from).collect()),
        },
        // Latin-1 real: cada byte es el punto de código del mismo valor
        // (`encoding_rs` lo trataría como windows-1252).
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
/// Devuelve el error de [`decode`]; solo puede ocurrir tras un BOM (p. ej.
/// `FF FE` seguido de un número impar de bytes, o `EF BB BF` seguido de
/// UTF-8 no válido).
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

/// Traduce el resultado de `chardetng` a una [`Detection`].
fn map_guess(guess: &'static encoding_rs::Encoding) -> Detection {
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

/// Decodifica UTF-8 estricto; `base` se suma al offset (longitud del BOM
/// quitado).
fn decode_utf8(bytes: &[u8], encoding: Encoding, base: usize) -> Result<String, DecodeError> {
    std::str::from_utf8(bytes)
        .map(str::to_owned)
        .map_err(|e| DecodeError::InvalidSequence {
            encoding,
            offset: base + e.valid_up_to(),
        })
}

/// Decodifica UTF-16 estricto con la endianness de `encoding`, quitando su
/// propio BOM si está al principio.
fn decode_utf16(bytes: &[u8], encoding: Encoding) -> Result<String, DecodeError> {
    let bom = encoding.bom();
    let (base, body) = match bytes.strip_prefix(bom) {
        Some(rest) => (bom.len(), rest),
        None => (0, bytes),
    };
    if body.len() % 2 != 0 {
        return Err(DecodeError::OddLength { encoding });
    }
    let (pairs, _rest) = body.as_chunks::<2>();
    let units: Vec<u16> = pairs
        .iter()
        .map(|&pair| {
            if encoding == Encoding::Utf16Be {
                u16::from_be_bytes(pair)
            } else {
                u16::from_le_bytes(pair)
            }
        })
        .collect();

    let mut text = String::with_capacity(units.len());
    // `index` cuenta las unidades de 16 bits ya decodificadas: al primer
    // sustituto sin pareja, señala justo esa unidad.
    let mut index = 0;
    for result in char::decode_utf16(units) {
        match result {
            Ok(c) => {
                text.push(c);
                index += c.len_utf16();
            }
            Err(_) => {
                return Err(DecodeError::InvalidSequence {
                    encoding,
                    offset: base + 2 * index,
                });
            }
        }
    }
    Ok(text)
}

/// Decodifica con una codificación de un byte de `encoding_rs` que asigna
/// un carácter a cada byte (no puede fallar).
fn decode_single_byte(bytes: &[u8], encoding: &'static encoding_rs::Encoding) -> String {
    let (text, _had_errors) = encoding.decode_without_bom_handling(bytes);
    text.into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn invalid(encoding: Encoding, offset: usize) -> Result<String, DecodeError> {
        Err(DecodeError::InvalidSequence { encoding, offset })
    }

    fn utf16le(s: &str) -> Vec<u8> {
        s.encode_utf16().flat_map(u16::to_le_bytes).collect()
    }

    fn utf16be(s: &str) -> Vec<u8> {
        s.encode_utf16().flat_map(u16::to_be_bytes).collect()
    }

    // ---------------------------------------------------------------- Encoding

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
    fn ids_match_the_frontend() {
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
    fn from_id_rejects_unknown_and_case_variants() {
        for id in [
            "",
            "UTF-8",
            "Utf-8",
            "utf8",
            "latin1",
            "windows-1251",
            " utf-8",
        ] {
            assert_eq!(Encoding::from_id(id), None, "id {id:?}");
        }
    }

    #[test]
    fn bom_bytes() {
        assert_eq!(Encoding::Utf8Bom.bom(), &[0xEF, 0xBB, 0xBF]);
        assert_eq!(Encoding::Utf16Le.bom(), &[0xFF, 0xFE]);
        assert_eq!(Encoding::Utf16Be.bom(), &[0xFE, 0xFF]);
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

    // ------------------------------------------------------------------- serde

    #[test]
    fn encoding_serializes_as_id() {
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
    fn detection_and_decoded_serialize_in_camel_case() {
        let d = Detection {
            encoding: Encoding::Utf8Bom,
            method: DetectionMethod::Bom,
        };
        assert_eq!(
            serde_json::to_string(&d).unwrap(),
            r#"{"encoding":"utf-8-bom","method":"bom"}"#
        );
        let d = Decoded {
            text: "x".into(),
            encoding: Encoding::Windows1252,
            method: DetectionMethod::Utf8Valid,
        };
        assert_eq!(
            serde_json::to_string(&d).unwrap(),
            r#"{"text":"x","encoding":"windows-1252","method":"utf8-valid"}"#
        );
    }

    // ------------------------------------------------------------------ errors

    #[test]
    fn error_display_mentions_encoding_and_offset() {
        let e = DecodeError::InvalidSequence {
            encoding: Encoding::Utf16Be,
            offset: 42,
        };
        let msg = e.to_string();
        assert!(msg.contains("utf-16be"), "{msg}");
        assert!(msg.contains("42"), "{msg}");

        let e = DecodeError::OddLength {
            encoding: Encoding::Utf16Le,
        };
        let msg = e.to_string();
        assert!(msg.contains("utf-16le"), "{msg}");
    }

    #[test]
    fn decode_error_is_a_std_error() {
        let e: Box<dyn std::error::Error> = Box::new(DecodeError::OddLength {
            encoding: Encoding::Utf16Le,
        });
        assert!(e.source().is_none());
    }

    // ------------------------------------------------------------------ detect

    fn detected(bytes: &[u8]) -> (Encoding, DetectionMethod) {
        let d = detect(bytes);
        (d.encoding, d.method)
    }

    #[test]
    fn detect_boms() {
        use DetectionMethod::Bom;
        assert_eq!(detected(&[0xEF, 0xBB, 0xBF]), (Encoding::Utf8Bom, Bom));
        assert_eq!(detected(b"\xEF\xBB\xBFhola"), (Encoding::Utf8Bom, Bom));
        assert_eq!(detected(&[0xFF, 0xFE]), (Encoding::Utf16Le, Bom));
        assert_eq!(
            detected(&[0xFE, 0xFF, 0x00, b'a']),
            (Encoding::Utf16Be, Bom)
        );
        // El BOM manda aunque lo que sigue no sea válido.
        assert_eq!(detected(&[0xFF, 0xFE, 0x41]), (Encoding::Utf16Le, Bom));
        assert_eq!(
            detected(&[0xEF, 0xBB, 0xBF, 0xFF]),
            (Encoding::Utf8Bom, Bom)
        );
    }

    #[test]
    fn detect_utf8_valid_including_empty_and_ascii() {
        use DetectionMethod::Utf8Valid;
        assert_eq!(detected(b""), (Encoding::Utf8, Utf8Valid));
        assert_eq!(detected(b"hello\r\nworld\0"), (Encoding::Utf8, Utf8Valid));
        assert_eq!(
            detected("Canción ñandú — €".as_bytes()),
            (Encoding::Utf8, Utf8Valid)
        );
        // Un BOM truncado no es BOM.
        let m = detect(&[0xEF, 0xBB]).method;
        assert!(m == DetectionMethod::Heuristic || m == DetectionMethod::Fallback);
    }

    #[test]
    fn detect_heuristic_windows_1252_for_western_text() {
        let text = "El pingüino comió una piña en la montaña; ¿acaso está aquí \
                    el niño? Canción, corazón, ilusión, después, también.";
        let (bytes, _, _) = encoding_rs::WINDOWS_1252.encode(text);
        assert!(std::str::from_utf8(&bytes).is_err());
        assert_eq!(
            detected(&bytes),
            (Encoding::Windows1252, DetectionMethod::Heuristic)
        );
    }

    #[test]
    fn detect_fallback_for_unsupported_guess() {
        let text = "Съешь же ещё этих мягких французских булок, да выпей чаю. \
                    Широкая электрификация южных губерний даст мощный толчок.";
        let (bytes, _, _) = encoding_rs::WINDOWS_1251.encode(text);
        assert_eq!(
            detected(&bytes),
            (Encoding::Windows1252, DetectionMethod::Fallback)
        );

        let text = "いろはにほへと ちりぬるを わかよたれそ つねならむ。日本語の文章です。";
        let (bytes, _, _) = encoding_rs::SHIFT_JIS.encode(text);
        assert_eq!(
            detected(&bytes),
            (Encoding::Windows1252, DetectionMethod::Fallback)
        );
    }

    #[test]
    fn detect_never_returns_ascii_or_latin1() {
        let samples: [&[u8]; 4] = [b"", b"abc", &[0xE9], &[0x80, 0x81, 0xFF]];
        for s in samples {
            let e = detect(s).encoding;
            assert_ne!(e, Encoding::Ascii);
            assert_ne!(e, Encoding::Iso8859_1);
        }
    }

    #[test]
    fn map_guess_covers_supported_and_unsupported_encodings() {
        use DetectionMethod::{Fallback, Heuristic};
        let cases = [
            (encoding_rs::WINDOWS_1252, Encoding::Windows1252, Heuristic),
            (encoding_rs::ISO_8859_15, Encoding::Iso8859_15, Heuristic),
            (encoding_rs::MACINTOSH, Encoding::MacRoman, Heuristic),
            (encoding_rs::WINDOWS_1251, Encoding::Windows1252, Fallback),
            (encoding_rs::SHIFT_JIS, Encoding::Windows1252, Fallback),
            (encoding_rs::UTF_8, Encoding::Windows1252, Fallback),
        ];
        for (guess, encoding, method) in cases {
            assert_eq!(
                map_guess(guess),
                Detection { encoding, method },
                "{guess:?}"
            );
        }
    }

    // ------------------------------------------------------------ decode UTF-8

    #[test]
    fn decode_utf8_valid() {
        assert_eq!(decode(b"", Encoding::Utf8).unwrap(), "");
        assert_eq!(
            decode("añ€😀\0".as_bytes(), Encoding::Utf8).unwrap(),
            "añ€😀\0"
        );
    }

    #[test]
    fn decode_utf8_keeps_a_leading_bom() {
        assert_eq!(
            decode(b"\xEF\xBB\xBFhi", Encoding::Utf8).unwrap(),
            "\u{FEFF}hi"
        );
    }

    #[test]
    fn decode_utf8_reports_first_invalid_offset() {
        assert_eq!(
            decode(b"ab\xFFcd", Encoding::Utf8),
            invalid(Encoding::Utf8, 2)
        );
        // Secuencia truncada al final.
        assert_eq!(
            decode(b"abc\xE2\x82", Encoding::Utf8),
            invalid(Encoding::Utf8, 3)
        );
        // Sustituto codificado en UTF-8 (CESU) no es válido.
        assert_eq!(
            decode(b"\xED\xA0\x80", Encoding::Utf8),
            invalid(Encoding::Utf8, 0)
        );
        // Forma demasiado larga.
        assert_eq!(
            decode(b"x\xC0\xAF", Encoding::Utf8),
            invalid(Encoding::Utf8, 1)
        );
    }

    #[test]
    fn decode_utf8_bom_strips_bom() {
        assert_eq!(
            decode(b"\xEF\xBB\xBFhola", Encoding::Utf8Bom).unwrap(),
            "hola"
        );
        assert_eq!(decode(b"\xEF\xBB\xBF", Encoding::Utf8Bom).unwrap(), "");
        // Solo se quita un BOM.
        assert_eq!(
            decode(b"\xEF\xBB\xBF\xEF\xBB\xBFx", Encoding::Utf8Bom).unwrap(),
            "\u{FEFF}x"
        );
    }

    #[test]
    fn decode_utf8_bom_without_bom_still_decodes() {
        assert_eq!(decode("ñ".as_bytes(), Encoding::Utf8Bom).unwrap(), "ñ");
        assert_eq!(decode(b"", Encoding::Utf8Bom).unwrap(), "");
    }

    #[test]
    fn decode_utf8_bom_offset_counts_the_bom() {
        assert_eq!(
            decode(b"\xEF\xBB\xBFab\x80", Encoding::Utf8Bom),
            invalid(Encoding::Utf8Bom, 5)
        );
        assert_eq!(
            decode(b"ab\x80", Encoding::Utf8Bom),
            invalid(Encoding::Utf8Bom, 2)
        );
    }

    // ----------------------------------------------------------- decode UTF-16

    #[test]
    fn decode_utf16_with_and_without_bom() {
        let s = "Añ€😀\0z";
        let mut le = vec![0xFF, 0xFE];
        le.extend(utf16le(s));
        let mut be = vec![0xFE, 0xFF];
        be.extend(utf16be(s));
        assert_eq!(decode(&le, Encoding::Utf16Le).unwrap(), s);
        assert_eq!(decode(&be, Encoding::Utf16Be).unwrap(), s);
        assert_eq!(decode(&utf16le(s), Encoding::Utf16Le).unwrap(), s);
        assert_eq!(decode(&utf16be(s), Encoding::Utf16Be).unwrap(), s);
        assert_eq!(decode(&[], Encoding::Utf16Le).unwrap(), "");
        assert_eq!(decode(&[0xFE, 0xFF], Encoding::Utf16Be).unwrap(), "");
    }

    #[test]
    fn decode_utf16_does_not_strip_the_other_endianness_bom() {
        // FE FF leído como LE es la unidad U+FFFE.
        assert_eq!(
            decode(&[0xFE, 0xFF, b'a', 0x00], Encoding::Utf16Le).unwrap(),
            "\u{FFFE}a"
        );
        assert_eq!(
            decode(&[0xFF, 0xFE, 0x00, b'a'], Encoding::Utf16Be).unwrap(),
            "\u{FFFE}a"
        );
    }

    #[test]
    fn decode_utf16_odd_length() {
        let odd = DecodeError::OddLength {
            encoding: Encoding::Utf16Le,
        };
        assert_eq!(decode(b"a", Encoding::Utf16Le), Err(odd.clone()));
        assert_eq!(decode(&[0xFF, 0xFE, b'a'], Encoding::Utf16Le), Err(odd));
        assert_eq!(
            decode(&[0x00, b'a', 0x00], Encoding::Utf16Be),
            Err(DecodeError::OddLength {
                encoding: Encoding::Utf16Be
            })
        );
    }

    #[test]
    fn decode_utf16_unpaired_surrogates() {
        // Sustituto alto al final (sin BOM).
        assert_eq!(
            decode(&[b'a', 0x00, 0x3D, 0xD8], Encoding::Utf16Le),
            invalid(Encoding::Utf16Le, 2)
        );
        // Sustituto alto seguido de un carácter normal, con BOM.
        assert_eq!(
            decode(&[0xFF, 0xFE, 0x3D, 0xD8, b'a', 0x00], Encoding::Utf16Le),
            invalid(Encoding::Utf16Le, 2)
        );
        // Sustituto bajo suelto tras un par válido (BE con BOM).
        let mut be = vec![0xFE, 0xFF];
        be.extend(utf16be("😀"));
        be.extend([0xDC, 0x00]);
        assert_eq!(
            decode(&be, Encoding::Utf16Be),
            invalid(Encoding::Utf16Be, 6)
        );
        // Dos sustitutos altos seguidos: falla el primero.
        assert_eq!(
            decode(&[0xD8, 0x3D, 0xD8, 0x3D, 0xDE, 0x00], Encoding::Utf16Be),
            invalid(Encoding::Utf16Be, 0)
        );
    }

    // ------------------------------------------------------------ decode ASCII

    #[test]
    fn decode_ascii() {
        assert_eq!(decode(b"abc\0\x7F", Encoding::Ascii).unwrap(), "abc\0\x7F");
        assert_eq!(
            decode(b"ab\x80c\xFF", Encoding::Ascii),
            invalid(Encoding::Ascii, 2)
        );
        assert_eq!(
            decode(b"\xFF", Encoding::Ascii),
            invalid(Encoding::Ascii, 0)
        );
    }

    // ------------------------------------------------------ decode single-byte

    #[test]
    fn decode_iso_8859_1_maps_every_byte_to_the_same_code_point() {
        let bytes: Vec<u8> = (0..=255).collect();
        let text = decode(&bytes, Encoding::Iso8859_1).unwrap();
        let expected: String = (0..=255u8).map(char::from).collect();
        assert_eq!(text, expected);
        assert_eq!(
            decode(&[0x80, 0x9F], Encoding::Iso8859_1).unwrap(),
            "\u{80}\u{9F}"
        );
    }

    #[test]
    fn decode_iso_8859_15_differences() {
        let bytes = [0xA4, 0xA6, 0xA8, 0xB4, 0xB8, 0xBC, 0xBD, 0xBE, 0xE9, 0x80];
        assert_eq!(
            decode(&bytes, Encoding::Iso8859_15).unwrap(),
            "€ŠšŽžŒœŸé\u{80}"
        );
    }

    #[test]
    fn decode_windows_1252() {
        assert_eq!(
            decode(&[0x80, 0x93, 0x94, 0xE9], Encoding::Windows1252).unwrap(),
            "€“”é"
        );
        assert_eq!(
            decode(&[0x81, 0x8D, 0x8F, 0x90, 0x9D], Encoding::Windows1252).unwrap(),
            "\u{81}\u{8D}\u{8F}\u{90}\u{9D}"
        );
    }

    #[test]
    fn decode_mac_roman() {
        assert_eq!(
            decode(&[0x80, 0x8E, 0x96, 0xA5, b'x'], Encoding::MacRoman).unwrap(),
            "Äéñ•x"
        );
    }

    #[test]
    fn single_byte_encodings_never_fail() {
        let bytes: Vec<u8> = (0..=255).collect();
        for e in [
            Encoding::Iso8859_1,
            Encoding::Iso8859_15,
            Encoding::Windows1252,
            Encoding::MacRoman,
        ] {
            let text = decode(&bytes, e).unwrap();
            assert_eq!(text.chars().count(), 256, "{e:?}");
            assert!(!text.contains('\u{FFFD}'), "{e:?}");
        }
    }

    #[test]
    fn nul_bytes_are_valid_everywhere() {
        for e in Encoding::ALL {
            let bytes: &[u8] = match e {
                Encoding::Utf16Le | Encoding::Utf16Be => &[0x00, 0x00],
                _ => &[0x00],
            };
            assert_eq!(decode(bytes, e).unwrap(), "\0", "{e:?}");
        }
    }

    // ------------------------------------------------------------- decode_auto

    #[test]
    fn decode_auto_with_bom() {
        let mut bytes = vec![0xFF, 0xFE];
        bytes.extend(utf16le("hola"));
        assert_eq!(
            decode_auto(&bytes).unwrap(),
            Decoded {
                text: "hola".into(),
                encoding: Encoding::Utf16Le,
                method: DetectionMethod::Bom,
            }
        );
    }

    #[test]
    fn decode_auto_utf8_and_heuristic() {
        assert_eq!(
            decode_auto("ñ".as_bytes()).unwrap(),
            Decoded {
                text: "ñ".into(),
                encoding: Encoding::Utf8,
                method: DetectionMethod::Utf8Valid,
            }
        );
        let text = "El pingüino comió una piña en la montaña; ¿acaso está aquí \
                    el niño? Canción, corazón, ilusión, después, también.";
        let (bytes, _, _) = encoding_rs::WINDOWS_1252.encode(text);
        let d = decode_auto(&bytes).unwrap();
        assert_eq!(d.text, text);
        assert_eq!(d.encoding, Encoding::Windows1252);
        assert_eq!(d.method, DetectionMethod::Heuristic);
    }

    #[test]
    fn decode_auto_fallback_never_fails() {
        let text = "Съешь же ещё этих мягких французских булок, да выпей чаю.";
        let (bytes, _, _) = encoding_rs::WINDOWS_1251.encode(text);
        let d = decode_auto(&bytes).unwrap();
        assert_eq!(d.encoding, Encoding::Windows1252);
        assert_eq!(d.text.chars().count(), bytes.len());
    }

    #[test]
    fn decode_auto_errors_after_bom() {
        assert_eq!(
            decode_auto(&[0xFF, 0xFE, b'a']),
            Err(DecodeError::OddLength {
                encoding: Encoding::Utf16Le
            })
        );
        assert_eq!(
            decode_auto(b"\xEF\xBB\xBFa\xC3"),
            Err(DecodeError::InvalidSequence {
                encoding: Encoding::Utf8Bom,
                offset: 4
            })
        );
    }
}
