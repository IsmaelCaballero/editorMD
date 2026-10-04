//! Conversión con pérdida (parte de **M09 `TextCodec`**): informe de los
//! caracteres que no caben en una codificación y codificación con sustitución,
//! transliteración o entidades HTML (`PLAN.md` §2.5, punto 6).
//!
//! Flujo previsto: antes de guardar con otra codificación, el controlador pide
//! un [`LossReport`]; si no está vacío, el usuario elige cancelar o una
//! [`LossStrategy`], y se codifica con [`encode_lossy`].

use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use super::{Encoding, encode};

/// Qué hacer con los caracteres que no caben en la codificación destino.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LossStrategy {
    /// Sustituir cada carácter por `?`.
    Replace,
    /// Transliterar a ASCII (`€` → `EUR`, `ñ` → `n`, `—` → `--`); si no hay
    /// transliteración, `?`.
    Transliterate,
    /// Entidad HTML numérica (`€` → `&#x20AC;`); solo tiene sentido al exportar a HTML.
    HtmlEntities,
}

/// Un carácter que no cabe en la codificación destino.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LossItem {
    /// El carácter.
    pub ch: char,
    /// Número de apariciones.
    pub count: usize,
    /// Líneas (desde 1, sin repetir y en orden) en las que aparece.
    pub lines: Vec<usize>,
    /// Lo que se escribiría con [`LossStrategy::Transliterate`].
    pub transliteration: String,
}

/// Informe de pérdidas al codificar un texto en una codificación.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LossReport {
    /// Codificación destino.
    pub encoding: Encoding,
    /// Caracteres afectados, en el orden de su primera aparición.
    pub items: Vec<LossItem>,
    /// Número total de apariciones afectadas.
    pub total: usize,
}

impl LossReport {
    /// `true` si el texto cabe entero en la codificación.
    pub fn is_lossless(&self) -> bool {
        self.items.is_empty()
    }
}

/// `true` si `ch` se puede representar en `encoding`.
///
/// # Examples
///
/// ```
/// use editormd_lib::model::text_codec::{can_encode, Encoding};
///
/// assert!(can_encode('ñ', Encoding::Iso8859_1));
/// assert!(!can_encode('€', Encoding::Iso8859_1));
/// assert!(can_encode('€', Encoding::Iso8859_15));
/// ```
pub fn can_encode(ch: char, encoding: Encoding) -> bool {
    match encoding {
        Encoding::Utf8 | Encoding::Utf8Bom | Encoding::Utf16Le | Encoding::Utf16Be => true,
        Encoding::Ascii => ch.is_ascii(),
        Encoding::Iso8859_1 => u32::from(ch) <= 0xFF,
        // Codificaciones de 8 bits de `encoding_rs`: se prueba a codificar el carácter.
        Encoding::Iso8859_15 | Encoding::Windows1252 | Encoding::MacRoman => {
            ch.is_ascii() || encode(ch.encode_utf8(&mut [0; 4]), encoding).is_ok()
        }
    }
}

/// Analiza qué caracteres de `text` no caben en `encoding`. Las líneas se
/// cuentan por `\n` (el texto del editor).
///
/// # Examples
///
/// ```
/// use editormd_lib::model::text_codec::{loss_report, Encoding};
///
/// let r = loss_report("10 €\n20 €", Encoding::Iso8859_1);
/// assert_eq!(r.total, 2);
/// assert_eq!(r.items[0].ch, '€');
/// assert_eq!(r.items[0].lines, vec![1, 2]);
/// assert_eq!(r.items[0].transliteration, "EUR");
/// ```
pub fn loss_report(text: &str, encoding: Encoding) -> LossReport {
    let mut items: Vec<LossItem> = Vec::new();
    let mut total = 0;
    for (index, line) in text.split('\n').enumerate() {
        for ch in line.chars().filter(|&c| !can_encode(c, encoding)) {
            total += 1;
            let line_number = index + 1;
            match items.iter_mut().find(|item| item.ch == ch) {
                Some(item) => {
                    item.count += 1;
                    if item.lines.last() != Some(&line_number) {
                        item.lines.push(line_number);
                    }
                }
                None => items.push(LossItem {
                    ch,
                    count: 1,
                    lines: vec![line_number],
                    transliteration: transliterate(ch),
                }),
            }
        }
    }
    LossReport {
        encoding,
        items,
        total,
    }
}

/// Transliteración ASCII de un carácter (`deunicode`); `?` si no la hay.
fn transliterate(ch: char) -> String {
    deunicode::deunicode_char(ch).unwrap_or("?").to_owned()
}

/// Codifica `text` en `encoding` aplicando `strategy` a los caracteres que no
/// caben. Nunca falla: el resultado se puede volver a leer con `encoding`.
///
/// # Examples
///
/// ```
/// use editormd_lib::model::text_codec::{encode_lossy, Encoding, LossStrategy};
///
/// assert_eq!(encode_lossy("10 €", Encoding::Ascii, LossStrategy::Replace), b"10 ?");
/// assert_eq!(encode_lossy("10 €", Encoding::Ascii, LossStrategy::Transliterate), b"10 EUR");
/// assert_eq!(encode_lossy("10 €", Encoding::Ascii, LossStrategy::HtmlEntities), b"10 &#x20AC;");
/// ```
pub fn encode_lossy(text: &str, encoding: Encoding, strategy: LossStrategy) -> Vec<u8> {
    let mut safe = String::with_capacity(text.len());
    for ch in text.chars() {
        if can_encode(ch, encoding) {
            safe.push(ch);
        } else {
            match strategy {
                LossStrategy::Replace => safe.push('?'),
                LossStrategy::Transliterate => safe.push_str(&transliterate(ch)),
                // Escribir en un `String` nunca falla.
                LossStrategy::HtmlEntities => {
                    let _ = write!(safe, "&#x{:X};", u32::from(ch));
                }
            }
        }
    }
    // Todo lo que queda cabe en `encoding`: la transliteración y las entidades
    // son ASCII, y ASCII cabe en todas las codificaciones soportadas.
    encode(&safe, encoding).unwrap_or_else(|_| safe.bytes().filter(u8::is_ascii).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::text_codec::decode;

    #[test]
    fn can_encode_per_encoding() {
        for e in [
            Encoding::Utf8,
            Encoding::Utf8Bom,
            Encoding::Utf16Le,
            Encoding::Utf16Be,
        ] {
            assert!(can_encode('😀', e) && can_encode('\u{0}', e), "{e:?}");
        }
        assert!(can_encode('~', Encoding::Ascii));
        assert!(!can_encode('ñ', Encoding::Ascii));
        assert!(can_encode('\u{80}', Encoding::Iso8859_1));
        assert!(!can_encode('Œ', Encoding::Iso8859_1));
        assert!(can_encode('Œ', Encoding::Iso8859_15));
        assert!(!can_encode('¤', Encoding::Iso8859_15));
        assert!(can_encode('“', Encoding::Windows1252));
        assert!(!can_encode('Ω', Encoding::Windows1252));
        assert!(can_encode('Ω', Encoding::MacRoman));
        assert!(!can_encode('😀', Encoding::MacRoman));
    }

    #[test]
    fn report_is_empty_when_everything_fits() {
        for e in Encoding::ALL {
            let r = loss_report("plain ascii\n", e);
            assert!(r.is_lossless(), "{e:?}");
            assert_eq!((r.total, r.items.len(), r.encoding), (0, 0, e));
        }
        assert!(loss_report("Ñandú 😀", Encoding::Utf16Be).is_lossless());
        assert!(loss_report("", Encoding::Ascii).is_lossless());
    }

    #[test]
    fn report_groups_by_char_in_order_of_appearance() {
        let text = "Año 2024: 10 €\nPingüino €\n\nÑu";
        let r = loss_report(text, Encoding::Ascii);
        let summary: Vec<(char, usize, Vec<usize>, &str)> = r
            .items
            .iter()
            .map(|i| (i.ch, i.count, i.lines.clone(), i.transliteration.as_str()))
            .collect();
        assert_eq!(
            summary,
            vec![
                ('ñ', 1, vec![1], "n"),
                ('€', 2, vec![1, 2], "EUR"),
                ('ü', 1, vec![2], "u"),
                ('Ñ', 1, vec![4], "N"),
            ]
        );
        assert_eq!(r.total, 5);
        assert!(!r.is_lossless());
    }

    #[test]
    fn report_lines_are_unique() {
        let r = loss_report("€€€\n€", Encoding::Iso8859_1);
        assert_eq!(r.items[0].count, 4);
        assert_eq!(r.items[0].lines, vec![1, 2]);
    }

    #[test]
    fn replace_strategy() {
        assert_eq!(
            encode_lossy("año 😀", Encoding::Ascii, LossStrategy::Replace),
            b"a?o ?"
        );
        assert_eq!(
            encode_lossy("10 € ñ", Encoding::Iso8859_1, LossStrategy::Replace),
            b"10 ? \xf1"
        );
    }

    #[test]
    fn transliterate_strategy_keeps_what_fits() {
        assert_eq!(
            encode_lossy(
                "Señor — 10 € «ok»",
                Encoding::Ascii,
                LossStrategy::Transliterate
            ),
            b"Senor -- 10 EUR <<ok>>"
        );
        // En Latin-1 la ñ cabe y se conserva; solo se translitera el €.
        assert_eq!(
            encode_lossy("ñ €", Encoding::Iso8859_1, LossStrategy::Transliterate),
            b"\xf1 EUR"
        );
        // Un acento combinante sin transliteración desaparece.
        assert_eq!(
            encode_lossy("e\u{301}", Encoding::Ascii, LossStrategy::Transliterate),
            b"e"
        );
    }

    #[test]
    fn html_entities_strategy() {
        assert_eq!(
            encode_lossy("€ 😀 ñ", Encoding::Iso8859_1, LossStrategy::HtmlEntities),
            b"&#x20AC; &#x1F600; \xf1"
        );
    }

    #[test]
    fn lossy_output_always_decodes_with_the_target_encoding() {
        let text = "Ñandú — 10 € «ok» 😀 日本 Ω e\u{301}\r\n";
        for e in Encoding::ALL {
            for s in [
                LossStrategy::Replace,
                LossStrategy::Transliterate,
                LossStrategy::HtmlEntities,
            ] {
                let bytes = encode_lossy(text, e, s);
                assert!(decode(&bytes, e).is_ok(), "{e:?} {s:?}");
            }
        }
    }

    #[test]
    fn lossless_text_is_identical_to_strict_encode() {
        let text = "Año, pingüino «ok»";
        for e in [
            Encoding::Iso8859_1,
            Encoding::Windows1252,
            Encoding::Utf16Le,
        ] {
            for s in [
                LossStrategy::Replace,
                LossStrategy::Transliterate,
                LossStrategy::HtmlEntities,
            ] {
                assert_eq!(
                    encode_lossy(text, e, s),
                    encode(text, e).unwrap(),
                    "{e:?} {s:?}"
                );
            }
        }
    }

    #[test]
    fn serializes_for_the_frontend() {
        let r = loss_report("€", Encoding::Ascii);
        assert_eq!(
            serde_json::to_value(&r).unwrap(),
            serde_json::json!({
                "encoding": "ascii",
                "items": [{ "ch": "€", "count": 1, "lines": [1], "transliteration": "EUR" }],
                "total": 1
            })
        );
        assert_eq!(
            serde_json::from_str::<LossStrategy>("\"html-entities\"").unwrap(),
            LossStrategy::HtmlEntities
        );
    }
}
