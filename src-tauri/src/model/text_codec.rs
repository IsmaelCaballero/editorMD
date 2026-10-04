//! **M09 `TextCodec`** (Modelo · Rust) — detección de la codificación y
//! decodificación de ficheros de texto.
//!
//! Convierte los bytes de un fichero en texto Unicode. Esta versión cubre solo
//! la **lectura**: [`detect`] decide la codificación (BOM, UTF-8 válido o
//! heurística con `chardetng`) y [`decode`] decodifica de forma estricta, sin
//! sustituir nada por `U+FFFD`: o se decodifica todo o se devuelve un
//! [`DecodeError`]. [`decode_auto`] combina ambos pasos.
//!
//! Las codificaciones soportadas son las de la decisión D-15 (ver
//! [`Encoding`]); sus identificadores coinciden con los del frontend
//! (`src/model/document.ts`).

use std::fmt;

use chardetng::{EncodingDetector, Iso2022JpDetection, Utf8Detection};
use serde::{Deserialize, Serialize};

/// Codificaciones soportadas (D-15).
///
/// Se serializa y deserializa como su identificador estable ([`Encoding::id`]),
/// p. ej. `"utf-8-bom"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Encoding {
    /// UTF-8 sin BOM (`"utf-8"`).
    #[serde(rename = "utf-8")]
    Utf8,
    /// UTF-8 con BOM `EF BB BF` (`"utf-8-bom"`).
    #[serde(rename = "utf-8-bom")]
    Utf8Bom,
    /// UTF-16 little-endian (`"utf-16le"`).
    #[serde(rename = "utf-16le")]
    Utf16Le,
    /// UTF-16 big-endian (`"utf-16be"`).
    #[serde(rename = "utf-16be")]
    Utf16Be,
    /// ASCII de 7 bits (`"ascii"`).
    #[serde(rename = "ascii")]
    Ascii,
    /// ISO-8859-1 / Latin-1 real (`"iso-8859-1"`).
    #[serde(rename = "iso-8859-1")]
    Iso8859_1,
    /// ISO-8859-15 / Latin-9 (`"iso-8859-15"`).
    #[serde(rename = "iso-8859-15")]
    Iso8859_15,
    /// Windows-1252 según WHATWG (`"windows-1252"`).
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
            Encoding::Utf8Bom => &[0xEF, 0xBB, 0xBF],
            Encoding::Utf16Le => &[0xFF, 0xFE],
            Encoding::Utf16Be => &[0xFE, 0xFF],
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
#[serde(rename_all = "kebab-case")]
pub enum DetectionMethod {
    /// La entrada empieza por una marca de orden de bytes (BOM).
    Bom,
    /// La entrada es UTF-8 válido (incluidos el vacío y el ASCII puro).
    Utf8Valid,
    /// La heurística de `chardetng` ha elegido una codificación soportada.
    Heuristic,
    /// La heurística ha propuesto una codificación no soportada y se usa
    /// Windows-1252 por defecto.
    Fallback,
}

/// Resultado de [`detect`]: codificación elegida y método usado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Detection {
    /// Codificación detectada.
    pub encoding: Encoding,
    /// Cómo se ha decidido.
    pub method: DetectionMethod,
}

/// Resultado de [`decode_auto`]: texto decodificado y detección usada.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Decoded {
    /// Texto Unicode (sin BOM si la codificación lo lleva).
    pub text: String,
    /// Codificación detectada.
    pub encoding: Encoding,
    /// Cómo se ha decidido la codificación.
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
        /// Posición en bytes del primer byte no válido.
        offset: usize,
    },
    /// UTF-16 con un número impar de bytes (tras quitar el BOM).
    OddLength {
        /// Codificación UTF-16 con la que se intentaba decodificar.
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
            DecodeError::OddLength { encoding } => write!(
                f,
                "número impar de bytes para {}: falta un byte de la última unidad",
                encoding.id()
            ),
        }
    }
}

impl std::error::Error for DecodeError {}

/// Detecta la codificación de `bytes`.
///
/// Orden: BOM (`Utf8Bom`, `Utf16Le`, `Utf16Be`); UTF-8 válido (`Utf8`,
/// incluidos el vacío y el ASCII puro); heurística de `chardetng` sobre toda
/// la entrada (`Windows1252`, `Iso8859_15` o `MacRoman`); y, si la heurística
/// propone otra codificación, `Windows1252` por defecto. Nunca devuelve
/// `Ascii` ni `Iso8859_1`.
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
    // 2. UTF-8 válido.
    if std::str::from_utf8(bytes).is_ok() {
        return Detection {
            encoding: Encoding::Utf8,
            method: DetectionMethod::Utf8Valid,
        };
    }
    // 3-4. Heurística sobre toda la entrada, sin TLD y sin permitir UTF-8.
    let mut detector = EncodingDetector::new(Iso2022JpDetection::Deny);
    detector.feed(bytes, true);
    map_guess(detector.guess(None, Utf8Detection::Deny))
}

/// Traduce la propuesta de `chardetng` a una [`Detection`].
///
/// Solo se aceptan las codificaciones monobyte soportadas; cualquier otra
/// propuesta se resuelve como Windows-1252 con método `Fallback`.
fn map_guess(guess: &'static encoding_rs::Encoding) -> Detection {
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

/// Decodifica `bytes` con `encoding` de forma estricta.
///
/// No sustituye nada por `U+FFFD`: o se decodifica todo o se devuelve un
/// error. `Utf8Bom`, `Utf16Le` y `Utf16Be` quitan su propio BOM si está al
/// principio; `Utf8` no lo quita. `Iso8859_1` es Latin-1 real (cada byte `b`
/// es `U+00bb`). Las codificaciones monobyte nunca fallan.
///
/// # Errors
///
/// - [`DecodeError::InvalidSequence`] con el offset (contando el BOM) del
///   primer byte no válido: UTF-8 mal formado, sustituto UTF-16 sin pareja o
///   byte `>= 0x80` en ASCII.
/// - [`DecodeError::OddLength`] si en UTF-16 queda un número impar de bytes
///   tras quitar el BOM.
///
/// # Examples
///
/// ```
/// use editormd_lib::model::text_codec::{decode, DecodeError, Encoding};
///
/// assert_eq!(decode(b"caf\xe9", Encoding::Iso8859_1).unwrap(), "café");
/// assert_eq!(decode(b"\xEF\xBB\xBFhi", Encoding::Utf8Bom).unwrap(), "hi");
/// assert_eq!(
///     decode(b"ab\xff", Encoding::Utf8),
///     Err(DecodeError::InvalidSequence { encoding: Encoding::Utf8, offset: 2 })
/// );
/// ```
pub fn decode(bytes: &[u8], encoding: Encoding) -> Result<String, DecodeError> {
    match encoding {
        Encoding::Utf8 => decode_utf8(bytes, 0, encoding),
        Encoding::Utf8Bom => {
            let skip = bom_len(bytes, encoding);
            decode_utf8(&bytes[skip..], skip, encoding)
        }
        Encoding::Utf16Le => decode_utf16(bytes, encoding, u16::from_le_bytes),
        Encoding::Utf16Be => decode_utf16(bytes, encoding, u16::from_be_bytes),
        Encoding::Ascii => match bytes.iter().position(|b| !b.is_ascii()) {
            Some(offset) => Err(DecodeError::InvalidSequence { encoding, offset }),
            None => Ok(latin1(bytes)),
        },
        Encoding::Iso8859_1 => Ok(latin1(bytes)),
        Encoding::Iso8859_15 => Ok(single_byte(bytes, encoding_rs::ISO_8859_15)),
        Encoding::Windows1252 => Ok(single_byte(bytes, encoding_rs::WINDOWS_1252)),
        Encoding::MacRoman => Ok(single_byte(bytes, encoding_rs::MACINTOSH)),
    }
}

/// Longitud del BOM de `encoding` si `bytes` empieza por él; 0 si no.
fn bom_len(bytes: &[u8], encoding: Encoding) -> usize {
    let bom = encoding.bom();
    if bytes.starts_with(bom) { bom.len() } else { 0 }
}

/// UTF-8 estricto; `base` es lo que se ha quitado delante (el BOM) y se suma
/// al offset del error.
fn decode_utf8(bytes: &[u8], base: usize, encoding: Encoding) -> Result<String, DecodeError> {
    match std::str::from_utf8(bytes) {
        Ok(text) => Ok(text.to_owned()),
        Err(err) => Err(DecodeError::InvalidSequence {
            encoding,
            offset: base + err.valid_up_to(),
        }),
    }
}

/// UTF-16 estricto con la endianness que da `unit`; quita el BOM propio.
fn decode_utf16(
    bytes: &[u8],
    encoding: Encoding,
    unit: fn([u8; 2]) -> u16,
) -> Result<String, DecodeError> {
    let base = bom_len(bytes, encoding);
    let body = &bytes[base..];
    let (pairs, rest) = body.as_chunks::<2>();
    if !rest.is_empty() {
        return Err(DecodeError::OddLength { encoding });
    }
    let units = pairs.iter().copied().map(unit);
    let mut text = String::with_capacity(body.len() / 2);
    // Índice (en unidades de 16 bits) del siguiente carácter.
    let mut index = 0;
    for decoded in char::decode_utf16(units) {
        let Ok(c) = decoded else {
            // El sustituto sin pareja es justo la unidad en `index`.
            return Err(DecodeError::InvalidSequence {
                encoding,
                offset: base + 2 * index,
            });
        };
        text.push(c);
        index += c.len_utf16();
    }
    Ok(text)
}

/// Latin-1 real: cada byte `b` es el carácter `U+00bb`.
fn latin1(bytes: &[u8]) -> String {
    bytes.iter().copied().map(char::from).collect()
}

/// Codificación monobyte de `encoding_rs`. Las que se usan aquí asignan un
/// carácter a cada byte, así que nunca hay errores ni se trata el BOM.
fn single_byte(bytes: &[u8], encoding: &'static encoding_rs::Encoding) -> String {
    let (text, _had_errors) = encoding.decode_without_bom_handling(bytes);
    text.into_owned()
}

/// Detecta la codificación con [`detect`] y decodifica con [`decode`].
///
/// # Errors
///
/// Devuelve el [`DecodeError`] de [`decode`]. Solo puede ocurrir tras un BOM
/// (p. ej. `FF FE` seguido de un número impar de bytes, o `EF BB BF` seguido
/// de UTF-8 no válido).
///
/// # Examples
///
/// ```
/// use editormd_lib::model::text_codec::{decode_auto, DetectionMethod, Encoding};
///
/// let d = decode_auto(b"caf\xe9 cr\xe8me br\xfbl\xe9e").unwrap();
/// assert_eq!(d.text, "café crème brûlée");
/// assert_eq!(d.encoding, Encoding::Windows1252);
/// assert_eq!(d.method, DetectionMethod::Heuristic);
/// ```
pub fn decode_auto(bytes: &[u8]) -> Result<Decoded, DecodeError> {
    let Detection { encoding, method } = detect(bytes);
    Ok(Decoded {
        text: decode(bytes, encoding)?,
        encoding,
        method,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn det(encoding: Encoding, method: DetectionMethod) -> Detection {
        Detection { encoding, method }
    }

    fn invalid(encoding: Encoding, offset: usize) -> DecodeError {
        DecodeError::InvalidSequence { encoding, offset }
    }

    // --- Encoding -------------------------------------------------------

    #[test]
    fn ids_match_frontend() {
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
    fn from_id_roundtrips_and_rejects_unknown() {
        for e in Encoding::ALL {
            assert_eq!(Encoding::from_id(e.id()), Some(e));
        }
        assert_eq!(Encoding::from_id("UTF-8"), None);
        assert_eq!(Encoding::from_id(""), None);
        assert_eq!(Encoding::from_id("latin1"), None);
    }

    #[test]
    fn boms() {
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

    #[test]
    fn encoding_serde_uses_id() {
        for e in Encoding::ALL {
            let json = serde_json::to_string(&e).unwrap();
            assert_eq!(json, format!("\"{}\"", e.id()));
            let back: Encoding = serde_json::from_str(&json).unwrap();
            assert_eq!(back, e);
        }
        assert!(serde_json::from_str::<Encoding>("\"Utf8\"").is_err());
    }

    #[test]
    fn method_and_structs_serde() {
        let methods: Vec<String> = [
            DetectionMethod::Bom,
            DetectionMethod::Utf8Valid,
            DetectionMethod::Heuristic,
            DetectionMethod::Fallback,
        ]
        .iter()
        .map(|m| serde_json::to_string(m).unwrap())
        .collect();
        assert_eq!(
            methods,
            ["\"bom\"", "\"utf8-valid\"", "\"heuristic\"", "\"fallback\""]
        );
        let d = det(Encoding::Utf8Bom, DetectionMethod::Bom);
        assert_eq!(
            serde_json::to_string(&d).unwrap(),
            r#"{"encoding":"utf-8-bom","method":"bom"}"#
        );
        let dec = Decoded {
            text: "a".into(),
            encoding: Encoding::Windows1252,
            method: DetectionMethod::Fallback,
        };
        assert_eq!(
            serde_json::to_string(&dec).unwrap(),
            r#"{"text":"a","encoding":"windows-1252","method":"fallback"}"#
        );
    }

    #[test]
    fn error_display_mentions_id_and_offset() {
        let e = invalid(Encoding::Utf16Le, 42);
        let msg = e.to_string();
        assert!(msg.contains("utf-16le"), "{msg}");
        assert!(msg.contains("42"), "{msg}");
        let msg = DecodeError::OddLength {
            encoding: Encoding::Utf16Be,
        }
        .to_string();
        assert!(msg.contains("utf-16be"), "{msg}");
        let _: &dyn std::error::Error = &e;
    }

    // --- detect -----------------------------------------------------------

    #[test]
    fn detect_boms() {
        assert_eq!(
            detect(&[0xEF, 0xBB, 0xBF, b'a']),
            det(Encoding::Utf8Bom, DetectionMethod::Bom)
        );
        assert_eq!(
            detect(&[0xEF, 0xBB, 0xBF]),
            det(Encoding::Utf8Bom, DetectionMethod::Bom)
        );
        assert_eq!(
            detect(&[0xFF, 0xFE, b'a', 0]),
            det(Encoding::Utf16Le, DetectionMethod::Bom)
        );
        assert_eq!(
            detect(&[0xFE, 0xFF, 0, b'a']),
            det(Encoding::Utf16Be, DetectionMethod::Bom)
        );
        // BOM seguido de bytes no válidos: sigue siendo BOM.
        assert_eq!(
            detect(&[0xFF, 0xFE, 0x41]),
            det(Encoding::Utf16Le, DetectionMethod::Bom)
        );
        assert_eq!(
            detect(&[0xEF, 0xBB, 0xBF, 0xFF]),
            det(Encoding::Utf8Bom, DetectionMethod::Bom)
        );
    }

    #[test]
    fn detect_partial_bom_is_not_bom() {
        // `EF BB` solo no es BOM ni UTF-8 válido.
        assert_ne!(detect(&[0xEF, 0xBB]).method, DetectionMethod::Bom);
        assert_ne!(detect(&[0xFF]).method, DetectionMethod::Bom);
    }

    #[test]
    fn detect_utf8_valid() {
        let utf8 = det(Encoding::Utf8, DetectionMethod::Utf8Valid);
        assert_eq!(detect(b""), utf8);
        assert_eq!(detect(b"hello world\n"), utf8);
        assert_eq!(detect("año, €, 日本語".as_bytes()), utf8);
        assert_eq!(detect(&[0, 0, 0]), utf8);
    }

    #[test]
    fn detect_windows1252_heuristic() {
        // «Él dijo “adiós” — sí» en windows-1252.
        let text = "Él dijo “adiós” al señor — sí, mañana será otro día.";
        let (bytes, _, _) = encoding_rs::WINDOWS_1252.encode(text);
        assert_eq!(
            detect(&bytes),
            det(Encoding::Windows1252, DetectionMethod::Heuristic)
        );
    }

    #[test]
    fn detect_latin1_text_is_windows1252() {
        assert_eq!(
            detect(b"caf\xe9 cr\xe8me br\xfbl\xe9e"),
            det(Encoding::Windows1252, DetectionMethod::Heuristic)
        );
    }

    #[test]
    fn detect_unsupported_guess_falls_back() {
        // Ruso en windows-1251.
        let text = "Привет, как дела? Это проверка кодировки текста на русском языке.";
        let (bytes, _, _) = encoding_rs::WINDOWS_1251.encode(text);
        assert_eq!(
            detect(&bytes),
            det(Encoding::Windows1252, DetectionMethod::Fallback)
        );
        // Japonés en Shift_JIS.
        let (bytes, _, _) = encoding_rs::SHIFT_JIS.encode("これは日本語のテキストです。");
        assert_eq!(
            detect(&bytes),
            det(Encoding::Windows1252, DetectionMethod::Fallback)
        );
    }

    #[test]
    fn map_guess_covers_all_branches() {
        assert_eq!(
            map_guess(encoding_rs::WINDOWS_1252),
            det(Encoding::Windows1252, DetectionMethod::Heuristic)
        );
        assert_eq!(
            map_guess(encoding_rs::ISO_8859_15),
            det(Encoding::Iso8859_15, DetectionMethod::Heuristic)
        );
        assert_eq!(
            map_guess(encoding_rs::MACINTOSH),
            det(Encoding::MacRoman, DetectionMethod::Heuristic)
        );
        assert_eq!(
            map_guess(encoding_rs::UTF_8),
            det(Encoding::Windows1252, DetectionMethod::Fallback)
        );
    }

    #[test]
    fn detect_never_returns_ascii_or_latin1() {
        let samples: [&[u8]; 4] = [b"abc", b"\x80\x81", b"\xe9", b"\xff\xff\xff"];
        for s in samples {
            let d = detect(s);
            assert_ne!(d.encoding, Encoding::Ascii);
            assert_ne!(d.encoding, Encoding::Iso8859_1);
        }
    }

    // --- decode: UTF-8 ----------------------------------------------------

    #[test]
    fn decode_utf8() {
        assert_eq!(decode(b"", Encoding::Utf8).unwrap(), "");
        assert_eq!(decode("año €".as_bytes(), Encoding::Utf8).unwrap(), "año €");
        assert_eq!(decode(b"a\0b", Encoding::Utf8).unwrap(), "a\0b");
    }

    #[test]
    fn decode_utf8_keeps_bom() {
        assert_eq!(
            decode(&[0xEF, 0xBB, 0xBF, b'a'], Encoding::Utf8).unwrap(),
            "\u{FEFF}a"
        );
    }

    #[test]
    fn decode_utf8_invalid_offset() {
        assert_eq!(
            decode(b"ab\xffcd", Encoding::Utf8),
            Err(invalid(Encoding::Utf8, 2))
        );
        // Secuencia truncada al final.
        assert_eq!(
            decode(b"abc\xe2\x82", Encoding::Utf8),
            Err(invalid(Encoding::Utf8, 3))
        );
        // Sustituto codificado en UTF-8 (CESU) no es válido.
        assert_eq!(
            decode(b"\xed\xa0\x80", Encoding::Utf8),
            Err(invalid(Encoding::Utf8, 0))
        );
    }

    #[test]
    fn decode_utf8_bom() {
        assert_eq!(
            decode(&[0xEF, 0xBB, 0xBF, b'h', b'i'], Encoding::Utf8Bom).unwrap(),
            "hi"
        );
        assert_eq!(decode(&[0xEF, 0xBB, 0xBF], Encoding::Utf8Bom).unwrap(), "");
        assert_eq!(decode(b"hi", Encoding::Utf8Bom).unwrap(), "hi");
        assert_eq!(decode(b"", Encoding::Utf8Bom).unwrap(), "");
        // Solo se quita un BOM.
        assert_eq!(
            decode(&[0xEF, 0xBB, 0xBF, 0xEF, 0xBB, 0xBF], Encoding::Utf8Bom).unwrap(),
            "\u{FEFF}"
        );
    }

    #[test]
    fn decode_utf8_bom_offset_counts_bom() {
        assert_eq!(
            decode(&[0xEF, 0xBB, 0xBF, b'a', 0xC3], Encoding::Utf8Bom),
            Err(invalid(Encoding::Utf8Bom, 4))
        );
        assert_eq!(
            decode(&[b'a', 0xC3], Encoding::Utf8Bom),
            Err(invalid(Encoding::Utf8Bom, 1))
        );
    }

    // --- decode: UTF-16 ---------------------------------------------------

    #[test]
    fn decode_utf16le() {
        assert_eq!(
            decode(&[0xFF, 0xFE, b'h', 0, b'i', 0], Encoding::Utf16Le).unwrap(),
            "hi"
        );
        assert_eq!(
            decode(&[b'h', 0, 0xAC, 0x20], Encoding::Utf16Le).unwrap(),
            "h€"
        );
        assert_eq!(decode(&[], Encoding::Utf16Le).unwrap(), "");
        assert_eq!(decode(&[0xFF, 0xFE], Encoding::Utf16Le).unwrap(), "");
        // Par sustituto: U+1F600.
        assert_eq!(
            decode(&[0x3D, 0xD8, 0x00, 0xDE], Encoding::Utf16Le).unwrap(),
            "\u{1F600}"
        );
        assert_eq!(decode(&[0, 0], Encoding::Utf16Le).unwrap(), "\0");
    }

    #[test]
    fn decode_utf16be() {
        assert_eq!(
            decode(&[0xFE, 0xFF, 0, b'h', 0, b'i'], Encoding::Utf16Be).unwrap(),
            "hi"
        );
        assert_eq!(
            decode(&[0xD8, 0x3D, 0xDE, 0x00], Encoding::Utf16Be).unwrap(),
            "\u{1F600}"
        );
    }

    #[test]
    fn decode_utf16_other_endian_bom_is_kept() {
        // FE FF leído como LE es U+FFFE (no carácter, pero válido en UTF-16).
        assert_eq!(
            decode(&[0xFE, 0xFF, b'a', 0], Encoding::Utf16Le).unwrap(),
            "\u{FFFE}a"
        );
        assert_eq!(
            decode(&[0xFF, 0xFE, 0, b'a'], Encoding::Utf16Be).unwrap(),
            "\u{FFFE}a"
        );
    }

    #[test]
    fn decode_utf16_odd_length() {
        assert_eq!(
            decode(&[b'a', 0, b'b'], Encoding::Utf16Le),
            Err(DecodeError::OddLength {
                encoding: Encoding::Utf16Le
            })
        );
        assert_eq!(
            decode(&[0xFE, 0xFF, 0], Encoding::Utf16Be),
            Err(DecodeError::OddLength {
                encoding: Encoding::Utf16Be
            })
        );
    }

    #[test]
    fn decode_utf16_unpaired_surrogates() {
        // Sustituto bajo aislado.
        assert_eq!(
            decode(&[b'a', 0, 0x00, 0xDC], Encoding::Utf16Le),
            Err(invalid(Encoding::Utf16Le, 2))
        );
        // Sustituto alto seguido de un carácter normal.
        assert_eq!(
            decode(&[0x3D, 0xD8, b'a', 0], Encoding::Utf16Le),
            Err(invalid(Encoding::Utf16Le, 0))
        );
        // Sustituto alto al final, con BOM: el offset cuenta el BOM.
        assert_eq!(
            decode(&[0xFE, 0xFF, 0, b'a', 0xD8, 0x3D], Encoding::Utf16Be),
            Err(invalid(Encoding::Utf16Be, 4))
        );
        // Dos sustitutos altos seguidos: falla el primero.
        assert_eq!(
            decode(&[0xD8, 0x3D, 0xD8, 0x3D, 0xDE, 0x00], Encoding::Utf16Be),
            Err(invalid(Encoding::Utf16Be, 0))
        );
    }

    // --- decode: monobyte -------------------------------------------------

    #[test]
    fn decode_ascii() {
        assert_eq!(decode(b"abc\0\x7f", Encoding::Ascii).unwrap(), "abc\0\x7f");
        assert_eq!(
            decode(b"abc\x80", Encoding::Ascii),
            Err(invalid(Encoding::Ascii, 3))
        );
        assert_eq!(
            decode("é".as_bytes(), Encoding::Ascii),
            Err(invalid(Encoding::Ascii, 0))
        );
    }

    #[test]
    fn decode_latin1_is_real_latin1() {
        let all: Vec<u8> = (0..=255u8).collect();
        let text = decode(&all, Encoding::Iso8859_1).unwrap();
        let expected: String = (0..=255u8).map(char::from).collect();
        assert_eq!(text, expected);
        assert_eq!(
            decode(&[0x80, 0x9F, 0xA4], Encoding::Iso8859_1).unwrap(),
            "\u{80}\u{9F}\u{A4}"
        );
    }

    #[test]
    fn decode_latin9() {
        assert_eq!(
            decode(
                &[0xA4, 0xA6, 0xA8, 0xB4, 0xB8, 0xBC, 0xBD, 0xBE, 0xE9],
                Encoding::Iso8859_15
            )
            .unwrap(),
            "€ŠšŽžŒœŸé"
        );
        let all: Vec<u8> = (0..=255u8).collect();
        assert_eq!(
            decode(&all, Encoding::Iso8859_15).unwrap().chars().count(),
            256
        );
    }

    #[test]
    fn decode_windows1252() {
        assert_eq!(
            decode(&[0x80, 0x93, 0x94, 0xE9], Encoding::Windows1252).unwrap(),
            "€“”é"
        );
        assert_eq!(
            decode(&[0x81, 0x8D, 0x8F, 0x90, 0x9D], Encoding::Windows1252).unwrap(),
            "\u{81}\u{8D}\u{8F}\u{90}\u{9D}"
        );
        // No quita un BOM UTF-8.
        assert_eq!(
            decode(&[0xEF, 0xBB, 0xBF], Encoding::Windows1252).unwrap(),
            "ï»¿"
        );
        let all: Vec<u8> = (0..=255u8).collect();
        assert_eq!(
            decode(&all, Encoding::Windows1252).unwrap().chars().count(),
            256
        );
    }

    #[test]
    fn decode_macroman() {
        assert_eq!(
            decode(&[0x8E, 0x87, 0xDB, 0xA5], Encoding::MacRoman).unwrap(),
            "éá€•"
        );
        let all: Vec<u8> = (0..=255u8).collect();
        assert_eq!(
            decode(&all, Encoding::MacRoman).unwrap().chars().count(),
            256
        );
    }

    // --- decode_auto ------------------------------------------------------

    #[test]
    fn decode_auto_utf8() {
        assert_eq!(
            decode_auto("hola ñ".as_bytes()).unwrap(),
            Decoded {
                text: "hola ñ".into(),
                encoding: Encoding::Utf8,
                method: DetectionMethod::Utf8Valid,
            }
        );
        assert_eq!(decode_auto(b"").unwrap().text, "");
    }

    #[test]
    fn decode_auto_bom() {
        assert_eq!(
            decode_auto(&[0xEF, 0xBB, 0xBF, b'x']).unwrap(),
            Decoded {
                text: "x".into(),
                encoding: Encoding::Utf8Bom,
                method: DetectionMethod::Bom,
            }
        );
        assert_eq!(
            decode_auto(&[0xFE, 0xFF, 0, b'x']).unwrap(),
            Decoded {
                text: "x".into(),
                encoding: Encoding::Utf16Be,
                method: DetectionMethod::Bom,
            }
        );
    }

    #[test]
    fn decode_auto_heuristic_and_fallback() {
        let d = decode_auto(b"caf\xe9 cr\xe8me br\xfbl\xe9e").unwrap();
        assert_eq!(d.text, "café crème brûlée");
        assert_eq!(d.encoding, Encoding::Windows1252);
        assert_eq!(d.method, DetectionMethod::Heuristic);

        let (bytes, _, _) = encoding_rs::WINDOWS_1251
            .encode("Привет, как дела? Это проверка кодировки текста на русском языке.");
        let d = decode_auto(&bytes).unwrap();
        assert_eq!(d.method, DetectionMethod::Fallback);
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
            decode_auto(&[0xEF, 0xBB, 0xBF, 0xFF]),
            Err(invalid(Encoding::Utf8Bom, 3))
        );
    }
}
