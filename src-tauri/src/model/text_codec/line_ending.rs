//! Finales de línea (parte de **M09 `TextCodec`**): recuento, detección del
//! predominante, aviso de mezcla y normalización (D-15, `PLAN.md` §2.5).
//!
//! El editor trabaja siempre con `\n`; el fin de línea original se guarda aparte
//! y se restaura al escribir el fichero.

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

/// Fin de línea: LF (Linux/macOS), CRLF (Windows) o CR (Mac clásico).
///
/// Se serializa con los mismos identificadores que el frontend (`"lf"`, `"crlf"`,
/// `"cr"`; ver `LineEnding` en `src/model/document.ts`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LineEnding {
    /// `\n` (Linux, macOS, Git).
    Lf,
    /// `\r\n` (Windows).
    Crlf,
    /// `\r` (Mac OS clásico, anterior a OS X).
    Cr,
}

impl LineEnding {
    /// Los 3 finales de línea, en el orden de la declaración (que es también el
    /// orden de preferencia cuando hay empate al buscar el predominante).
    pub const ALL: [LineEnding; 3] = [LineEnding::Lf, LineEnding::Crlf, LineEnding::Cr];

    /// Identificador estable: `"lf"`, `"crlf"` o `"cr"`.
    pub fn id(self) -> &'static str {
        match self {
            LineEnding::Lf => "lf",
            LineEnding::Crlf => "crlf",
            LineEnding::Cr => "cr",
        }
    }

    /// Inverso de [`LineEnding::id`]; `None` si el identificador no existe.
    pub fn from_id(id: &str) -> Option<LineEnding> {
        LineEnding::ALL.into_iter().find(|le| le.id() == id)
    }

    /// La secuencia de caracteres: `"\n"`, `"\r\n"` o `"\r"`.
    ///
    /// # Examples
    ///
    /// ```
    /// use editormd_lib::model::text_codec::LineEnding;
    ///
    /// assert_eq!(LineEnding::Crlf.as_str(), "\r\n");
    /// ```
    pub fn as_str(self) -> &'static str {
        match self {
            LineEnding::Lf => "\n",
            LineEnding::Crlf => "\r\n",
            LineEnding::Cr => "\r",
        }
    }

    /// El fin de línea habitual del sistema en el que se compiló: CRLF en
    /// Windows y LF en los demás.
    pub fn native() -> LineEnding {
        if cfg!(windows) {
            LineEnding::Crlf
        } else {
            LineEnding::Lf
        }
    }
}

/// Número de finales de línea de cada tipo en un texto.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct LineEndingStats {
    /// `\n` que no van precedidos de `\r`.
    pub lf: usize,
    /// Pares `\r\n`.
    pub crlf: usize,
    /// `\r` que no van seguidos de `\n`.
    pub cr: usize,
}

impl LineEndingStats {
    /// Cuenta los finales de línea de `text`. Un `\r\n` cuenta como un CRLF,
    /// no como un CR más un LF.
    ///
    /// # Examples
    ///
    /// ```
    /// use editormd_lib::model::text_codec::LineEndingStats;
    ///
    /// let s = LineEndingStats::of("a\nb\r\nc\rd\n");
    /// assert_eq!((s.lf, s.crlf, s.cr), (2, 1, 1));
    /// assert!(s.is_mixed());
    /// ```
    pub fn of(text: &str) -> LineEndingStats {
        let mut stats = LineEndingStats::default();
        let mut bytes = text.bytes().peekable();
        while let Some(b) = bytes.next() {
            match b {
                b'\r' if bytes.peek() == Some(&b'\n') => {
                    bytes.next();
                    stats.crlf += 1;
                }
                b'\r' => stats.cr += 1,
                b'\n' => stats.lf += 1,
                _ => {}
            }
        }
        stats
    }

    /// Número total de saltos de línea.
    pub fn total(&self) -> usize {
        self.lf + self.crlf + self.cr
    }

    /// `true` si aparece más de un tipo de fin de línea.
    pub fn is_mixed(&self) -> bool {
        [self.lf, self.crlf, self.cr]
            .iter()
            .filter(|&&n| n > 0)
            .count()
            > 1
    }

    /// El tipo más frecuente; en caso de empate, el primero de [`LineEnding::ALL`].
    /// `None` si el texto no tiene saltos de línea.
    pub fn dominant(&self) -> Option<LineEnding> {
        let counts = [
            (LineEnding::Lf, self.lf),
            (LineEnding::Crlf, self.crlf),
            (LineEnding::Cr, self.cr),
        ];
        // `max_by_key` se queda con el último de los empatados: se recorre al revés
        // para que gane el primero de `LineEnding::ALL`.
        counts
            .into_iter()
            .rev()
            .filter(|&(_, n)| n > 0)
            .max_by_key(|&(_, n)| n)
            .map(|(le, _)| le)
    }
}

/// Sustituye todos los finales de línea (LF, CRLF y CR) por `target`.
/// Si el texto ya está normalizado, no copia nada ([`Cow::Borrowed`]).
///
/// # Examples
///
/// ```
/// use editormd_lib::model::text_codec::{normalize_line_endings, LineEnding};
///
/// assert_eq!(normalize_line_endings("a\r\nb\rc", LineEnding::Lf), "a\nb\nc");
/// ```
pub fn normalize_line_endings(text: &str, target: LineEnding) -> Cow<'_, str> {
    let stats = LineEndingStats::of(text);
    let already = match target {
        LineEnding::Lf => stats.crlf == 0 && stats.cr == 0,
        LineEnding::Crlf => stats.lf == 0 && stats.cr == 0,
        LineEnding::Cr => stats.lf == 0 && stats.crlf == 0,
    };
    if already {
        return Cow::Borrowed(text);
    }
    let eol = target.as_str();
    let mut out = String::with_capacity(text.len() + stats.total());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                out.push_str(eol);
            }
            '\n' => out.push_str(eol),
            _ => out.push(c),
        }
    }
    Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_roundtrip_and_serde() {
        for (le, id, s) in [
            (LineEnding::Lf, "lf", "\n"),
            (LineEnding::Crlf, "crlf", "\r\n"),
            (LineEnding::Cr, "cr", "\r"),
        ] {
            assert_eq!(le.id(), id);
            assert_eq!(le.as_str(), s);
            assert_eq!(LineEnding::from_id(id), Some(le));
            assert_eq!(serde_json::to_string(&le).unwrap(), format!("\"{id}\""));
            assert_eq!(
                serde_json::from_str::<LineEnding>(&format!("\"{id}\"")).unwrap(),
                le
            );
        }
        assert_eq!(LineEnding::from_id("CRLF"), None);
        assert_eq!(LineEnding::from_id(""), None);
    }

    #[test]
    fn native_depends_on_os() {
        let expected = if cfg!(windows) {
            LineEnding::Crlf
        } else {
            LineEnding::Lf
        };
        assert_eq!(LineEnding::native(), expected);
    }

    #[test]
    fn stats_count_each_kind() {
        assert_eq!(LineEndingStats::of(""), LineEndingStats::default());
        assert_eq!(LineEndingStats::of("sin saltos").total(), 0);
        let s = LineEndingStats::of("a\r\n\r\nb\n\rc\r");
        // \r\n, \r\n, \n, \r, \r  → el «\n\r» son un LF y un CR, no un CRLF
        assert_eq!(
            s,
            LineEndingStats {
                lf: 1,
                crlf: 2,
                cr: 2
            }
        );
        assert_eq!(s.total(), 5);
        assert_eq!(LineEndingStats::of("\r\r\n").cr, 1);
    }

    #[test]
    fn stats_mixed_and_dominant() {
        let only = |t| LineEndingStats::of(t);
        assert!(!only("a\nb\n").is_mixed());
        assert!(!only("a\r\nb\r\n").is_mixed());
        assert!(!only("").is_mixed());
        assert!(only("a\nb\r\n").is_mixed());
        assert_eq!(only("").dominant(), None);
        assert_eq!(only("a\r\nb\r\nc\n").dominant(), Some(LineEnding::Crlf));
        assert_eq!(only("a\rb\r").dominant(), Some(LineEnding::Cr));
        // Empates: LF > CRLF > CR
        assert_eq!(only("a\nb\r\n").dominant(), Some(LineEnding::Lf));
        assert_eq!(only("a\r\nb\r").dominant(), Some(LineEnding::Crlf));
    }

    #[test]
    fn normalize_to_each_target() {
        let mixed = "a\nb\r\nc\rd";
        assert_eq!(normalize_line_endings(mixed, LineEnding::Lf), "a\nb\nc\nd");
        assert_eq!(
            normalize_line_endings(mixed, LineEnding::Crlf),
            "a\r\nb\r\nc\r\nd"
        );
        assert_eq!(normalize_line_endings(mixed, LineEnding::Cr), "a\rb\rc\rd");
        assert_eq!(normalize_line_endings("\r\n\r\n", LineEnding::Lf), "\n\n");
        assert_eq!(normalize_line_endings("\n\r", LineEnding::Crlf), "\r\n\r\n");
        assert_eq!(normalize_line_endings("", LineEnding::Crlf), "");
    }

    #[test]
    fn normalize_borrows_when_unchanged() {
        for (text, le) in [
            ("a\nb\n", LineEnding::Lf),
            ("a\r\nb", LineEnding::Crlf),
            ("a\rb", LineEnding::Cr),
            ("sin saltos", LineEnding::Crlf),
        ] {
            assert!(
                matches!(normalize_line_endings(text, le), Cow::Borrowed(_)),
                "{text:?}"
            );
        }
        assert!(matches!(
            normalize_line_endings("a\nb", LineEnding::Crlf),
            Cow::Owned(_)
        ));
    }
}
