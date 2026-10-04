//! Codificación de salida (parte de **M09 `TextCodec`**): texto Unicode → bytes
//! en cualquiera de las 9 codificaciones, con su BOM (D-15).
//!
//! Es estricta: si un carácter no cabe en la codificación destino se devuelve
//! un error con el carácter y su posición; nunca se sustituye en silencio. El
//! informe completo de pérdidas, la transliteración y la sustitución son del
//! paso 3 de F2.

use std::fmt;

use super::Encoding;

/// Error de codificación.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodeError {
    /// Un carácter no se puede representar en la codificación destino.
    Unmappable {
        /// Codificación destino.
        encoding: Encoding,
        /// Primer carácter que no se puede representar.
        ch: char,
        /// Posición del carácter en el texto, en bytes UTF-8.
        offset: usize,
    },
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncodeError::Unmappable {
                encoding,
                ch,
                offset,
            } => write!(
                f,
                "el carácter «{ch}» (U+{:04X}) no existe en la codificación {} (byte {offset} del texto)",
                u32::from(*ch),
                encoding.id()
            ),
        }
    }
}

impl std::error::Error for EncodeError {}

/// Codifica `text` en `encoding`, añadiendo su BOM si lo tiene
/// ([`Encoding::bom`]). No toca los finales de línea.
///
/// # Errors
///
/// [`EncodeError::Unmappable`] con el primer carácter que no cabe en la
/// codificación (solo en ASCII y en las codificaciones de 8 bits).
///
/// # Examples
///
/// ```
/// use editormd_lib::model::text_codec::{encode, Encoding};
///
/// assert_eq!(encode("año", Encoding::Windows1252).unwrap(), b"a\xf1o");
/// assert_eq!(encode("a", Encoding::Utf16Le).unwrap(), b"\xff\xfea\x00");
/// assert!(encode("año", Encoding::Ascii).is_err());
/// ```
pub fn encode(text: &str, encoding: Encoding) -> Result<Vec<u8>, EncodeError> {
    todo!("{text} {encoding:?}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::text_codec::decode;

    #[test]
    fn utf8_with_and_without_bom() {
        assert_eq!(encode("ñ😀", Encoding::Utf8).unwrap(), "ñ😀".as_bytes());
        assert_eq!(
            encode("ñ", Encoding::Utf8Bom).unwrap(),
            [0xEF, 0xBB, 0xBF, 0xC3, 0xB1]
        );
        assert_eq!(encode("", Encoding::Utf8).unwrap(), b"");
        assert_eq!(encode("", Encoding::Utf8Bom).unwrap(), [0xEF, 0xBB, 0xBF]);
    }

    #[test]
    fn utf16_both_endians_with_bom_and_surrogates() {
        assert_eq!(
            encode("Añ😀", Encoding::Utf16Le).unwrap(),
            [0xFF, 0xFE, 0x41, 0x00, 0xF1, 0x00, 0x3D, 0xD8, 0x00, 0xDE]
        );
        assert_eq!(
            encode("Añ😀", Encoding::Utf16Be).unwrap(),
            [0xFE, 0xFF, 0x00, 0x41, 0x00, 0xF1, 0xD8, 0x3D, 0xDE, 0x00]
        );
        assert_eq!(encode("", Encoding::Utf16Be).unwrap(), [0xFE, 0xFF]);
    }

    #[test]
    fn ascii_rejects_non_ascii_with_offset() {
        assert_eq!(encode("abc~\t\r\n", Encoding::Ascii).unwrap(), b"abc~\t\r\n");
        assert_eq!(
            encode("Año", Encoding::Ascii),
            Err(EncodeError::Unmappable {
                encoding: Encoding::Ascii,
                ch: 'ñ',
                offset: 1
            })
        );
    }

    #[test]
    fn latin1_is_real_latin1() {
        let all: String = (0u32..=255).filter_map(char::from_u32).collect();
        let bytes = encode(&all, Encoding::Iso8859_1).unwrap();
        assert_eq!(bytes, (0u8..=255).collect::<Vec<_>>());
        assert_eq!(
            encode("10 €", Encoding::Iso8859_1),
            Err(EncodeError::Unmappable {
                encoding: Encoding::Iso8859_1,
                ch: '€',
                offset: 3
            })
        );
    }

    #[test]
    fn single_byte_codecs() {
        assert_eq!(encode("€Š", Encoding::Iso8859_15).unwrap(), [0xA4, 0xA6]);
        assert_eq!(encode("€—“”", Encoding::Windows1252).unwrap(), [0x80, 0x97, 0x93, 0x94]);
        assert_eq!(encode("ñé•", Encoding::MacRoman).unwrap(), [0x96, 0x8E, 0xA5]);
        // Los C1 «no definidos» de Windows-1252 vuelven a su byte (WHATWG).
        assert_eq!(encode("\u{81}\u{9D}", Encoding::Windows1252).unwrap(), [0x81, 0x9D]);
    }

    #[test]
    fn single_byte_unmappable_reports_first_char() {
        assert_eq!(
            encode("ok ¤ 😀", Encoding::Iso8859_15),
            Err(EncodeError::Unmappable {
                encoding: Encoding::Iso8859_15,
                ch: '¤',
                offset: 3
            })
        );
        assert_eq!(
            encode("año 😀", Encoding::Windows1252),
            Err(EncodeError::Unmappable {
                encoding: Encoding::Windows1252,
                ch: '😀',
                offset: 5
            })
        );
        assert!(encode("Ω≠", Encoding::Windows1252).is_err());
    }

    #[test]
    fn eight_bit_codecs_are_bijective() {
        // Para las codificaciones de 8 bits, decodificar y volver a codificar
        // los 256 bytes debe dar exactamente los mismos bytes.
        let all: Vec<u8> = (0u8..=255).collect();
        for e in [
            Encoding::Iso8859_1,
            Encoding::Iso8859_15,
            Encoding::Windows1252,
            Encoding::MacRoman,
        ] {
            let text = decode(&all, e).unwrap();
            assert_eq!(encode(&text, e).unwrap(), all, "{e:?}");
        }
    }

    #[test]
    fn decode_encode_roundtrip_unicode() {
        let text = "Ñandú 😀 日本 \u{0} fin\r\n";
        for e in [
            Encoding::Utf8,
            Encoding::Utf8Bom,
            Encoding::Utf16Le,
            Encoding::Utf16Be,
        ] {
            let bytes = encode(text, e).unwrap();
            assert_eq!(decode(&bytes, e).unwrap(), text, "{e:?}");
        }
    }

    #[test]
    fn error_display_mentions_char_codepoint_and_encoding() {
        let msg = EncodeError::Unmappable {
            encoding: Encoding::Ascii,
            ch: 'ñ',
            offset: 1,
        }
        .to_string();
        assert!(msg.contains('ñ') && msg.contains("U+00F1") && msg.contains("ascii"), "{msg}");
    }
}
