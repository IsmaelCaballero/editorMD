//! Pruebas OCULTAS de aceptación de la tarea RQ.1 T1 (M09 TextCodec: detección y decodificación).
//! No se entregaron a las ejecuciones: el orquestador las copió en `src-tauri/tests/` solo al evaluar
//! (`scripts/rq1-eval.sh`). Se publican tras el experimento para que sea reproducible.
#![allow(clippy::pedantic)]

use editormd_lib::model::text_codec::{
    decode, decode_auto, detect, DecodeError, Decoded, Detection, DetectionMethod, Encoding,
};

fn hex(s: &str) -> Vec<u8> {
    s.split_whitespace()
        .map(|b| u8::from_str_radix(b, 16).unwrap())
        .collect()
}

// ---------- Encoding: ids, ALL, BOM, serde ----------

const IDS: [(Encoding, &str); 9] = [
    (Encoding::Utf8, "utf-8"),
    (Encoding::Utf8Bom, "utf-8-bom"),
    (Encoding::Utf16Le, "utf-16le"),
    (Encoding::Utf16Be, "utf-16be"),
    (Encoding::Ascii, "ascii"),
    (Encoding::Iso8859_1, "iso-8859-1"),
    (Encoding::Iso8859_15, "iso-8859-15"),
    (Encoding::Windows1252, "windows-1252"),
    (Encoding::MacRoman, "macintosh"),
];

#[test]
fn h01_all_in_declaration_order() {
    let expected: Vec<Encoding> = IDS.iter().map(|(e, _)| *e).collect();
    assert_eq!(Encoding::ALL.to_vec(), expected);
}

#[test]
fn h02_ids_and_from_id_roundtrip() {
    for (e, id) in IDS {
        assert_eq!(e.id(), id);
        assert_eq!(Encoding::from_id(id), Some(e));
    }
    assert_eq!(Encoding::from_id("UTF-8"), None);
    assert_eq!(Encoding::from_id("latin1"), None);
    assert_eq!(Encoding::from_id(""), None);
}

#[test]
fn h03_boms() {
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
fn h04_serde_encoding_as_id() {
    for (e, id) in IDS {
        assert_eq!(serde_json::to_string(&e).unwrap(), format!("\"{id}\""));
        let back: Encoding = serde_json::from_str(&format!("\"{id}\"")).unwrap();
        assert_eq!(back, e);
    }
    assert!(serde_json::from_str::<Encoding>("\"utf8\"").is_err());
}

#[test]
fn h05_serde_detection_and_decoded() {
    let d = Detection { encoding: Encoding::Utf8Bom, method: DetectionMethod::Bom };
    assert_eq!(
        serde_json::to_value(d).unwrap(),
        serde_json::json!({"encoding": "utf-8-bom", "method": "bom"})
    );
    let methods = [
        (DetectionMethod::Bom, "bom"),
        (DetectionMethod::Utf8Valid, "utf8-valid"),
        (DetectionMethod::Heuristic, "heuristic"),
        (DetectionMethod::Fallback, "fallback"),
    ];
    for (m, s) in methods {
        assert_eq!(serde_json::to_string(&m).unwrap(), format!("\"{s}\""));
    }
    let dec = Decoded {
        text: "hola".into(),
        encoding: Encoding::Windows1252,
        method: DetectionMethod::Heuristic,
    };
    assert_eq!(
        serde_json::to_value(&dec).unwrap(),
        serde_json::json!({"text": "hola", "encoding": "windows-1252", "method": "heuristic"})
    );
}

// ---------- detect ----------

#[test]
fn h10_detect_boms() {
    let cases = [
        (hex("EF BB BF 68 6F 6C 61"), Encoding::Utf8Bom),
        (hex("FF FE 68 00"), Encoding::Utf16Le),
        (hex("FE FF 00 68"), Encoding::Utf16Be),
        (hex("EF BB BF"), Encoding::Utf8Bom),
        (hex("FF FE"), Encoding::Utf16Le),
        // BOM seguido de basura: la detección sigue siendo por BOM
        (hex("FF FE 41"), Encoding::Utf16Le),
        (hex("EF BB BF FF"), Encoding::Utf8Bom),
    ];
    for (bytes, enc) in cases {
        assert_eq!(
            detect(&bytes),
            Detection { encoding: enc, method: DetectionMethod::Bom },
            "{bytes:02X?}"
        );
    }
}

#[test]
fn h11_detect_utf8_valid_empty_ascii_multibyte() {
    for bytes in [
        Vec::new(),
        b"# Titulo\n\nTexto ASCII puro.\r\n".to_vec(),
        "Ñandú, pingüino, €, 😀, 日本".as_bytes().to_vec(),
        vec![0x00, 0x41, 0x00],
    ] {
        assert_eq!(
            detect(&bytes),
            Detection { encoding: Encoding::Utf8, method: DetectionMethod::Utf8Valid },
            "{bytes:02X?}"
        );
    }
}

#[test]
fn h12_detect_partial_bom_is_not_bom() {
    // EF BB sin BF: no es BOM ni UTF-8 válido → heurística/fallback, nunca Bom.
    let d = detect(&hex("EF BB 41 42"));
    assert_ne!(d.method, DetectionMethod::Bom);
    assert_ne!(d.method, DetectionMethod::Utf8Valid);
}

#[test]
fn h13_detect_spanish_windows1252() {
    let text = "El pingüino Ñandú comió año tras año en la montaña; «cita» – €5. \
                Está aquí también la canción del corazón.\r\n";
    let (bytes, _, _) = encoding_rs::WINDOWS_1252.encode(text);
    let d = detect(&bytes);
    assert_eq!(d.encoding, Encoding::Windows1252);
    assert_eq!(d.method, DetectionMethod::Heuristic);
}

#[test]
fn h14_detect_never_ascii_or_latin1() {
    let samples: Vec<Vec<u8>> = vec![
        b"plain".to_vec(),
        hex("E1 E9 ED F3 FA F1"),
        hex("80 81 82 83"),
        hex("41 FF 42"),
    ];
    for s in samples {
        let d = detect(&s);
        assert_ne!(d.encoding, Encoding::Ascii);
        assert_ne!(d.encoding, Encoding::Iso8859_1);
    }
}

#[test]
fn h15_detect_non_latin_falls_back_to_windows1252() {
    // Ruso en windows-1251: chardetng no devuelve una codificación soportada.
    let (bytes, _, _) = encoding_rs::WINDOWS_1251
        .encode("Съешь же ещё этих мягких французских булок, да выпей чаю. Привет, мир!");
    let d = detect(&bytes);
    assert_eq!(d.encoding, Encoding::Windows1252);
    assert_eq!(d.method, DetectionMethod::Fallback);
}

// ---------- decode: UTF-8 ----------

#[test]
fn h20_utf8_keeps_bom_char() {
    assert_eq!(decode(&hex("EF BB BF 41"), Encoding::Utf8).unwrap(), "\u{FEFF}A");
}

#[test]
fn h21_utf8bom_strips_optional_bom() {
    assert_eq!(decode(&hex("EF BB BF 41"), Encoding::Utf8Bom).unwrap(), "A");
    assert_eq!(decode(b"A", Encoding::Utf8Bom).unwrap(), "A");
    assert_eq!(decode(&hex("EF BB BF"), Encoding::Utf8Bom).unwrap(), "");
    // Solo se quita un BOM
    assert_eq!(
        decode(&hex("EF BB BF EF BB BF 41"), Encoding::Utf8Bom).unwrap(),
        "\u{FEFF}A"
    );
}

#[test]
fn h22_utf8_invalid_offsets() {
    assert_eq!(
        decode(&hex("41 42 FF 43"), Encoding::Utf8),
        Err(DecodeError::InvalidSequence { encoding: Encoding::Utf8, offset: 2 })
    );
    // Secuencia truncada al final
    assert_eq!(
        decode(&hex("41 C3"), Encoding::Utf8),
        Err(DecodeError::InvalidSequence { encoding: Encoding::Utf8, offset: 1 })
    );
    // Con BOM: el offset cuenta el BOM
    assert_eq!(
        decode(&hex("EF BB BF 41 FF"), Encoding::Utf8Bom),
        Err(DecodeError::InvalidSequence { encoding: Encoding::Utf8Bom, offset: 4 })
    );
    // Sobrelargo / surrogate codificado en UTF-8: no válidos
    assert!(decode(&hex("C0 AF"), Encoding::Utf8).is_err());
    assert!(decode(&hex("ED A0 80"), Encoding::Utf8).is_err());
}

#[test]
fn h23_nul_bytes_are_valid() {
    assert_eq!(decode(&hex("41 00 42"), Encoding::Utf8).unwrap(), "A\0B");
    assert_eq!(decode(&hex("41 00 42"), Encoding::Ascii).unwrap(), "A\0B");
    assert_eq!(decode(&hex("00 00"), Encoding::Utf16Le).unwrap(), "\0");
}

// ---------- decode: UTF-16 ----------

#[test]
fn h30_utf16_le_be_with_and_without_bom() {
    let le = hex("48 00 6f 00 6c 00 61 00 20 00 f1 00 20 00 3d d8 00 de");
    let be = hex("00 48 00 6f 00 6c 00 61 00 20 00 f1 00 20 d8 3d de 00");
    assert_eq!(decode(&le, Encoding::Utf16Le).unwrap(), "Hola ñ 😀");
    assert_eq!(decode(&be, Encoding::Utf16Be).unwrap(), "Hola ñ 😀");
    let mut le_bom = hex("FF FE");
    le_bom.extend(&le);
    let mut be_bom = hex("FE FF");
    be_bom.extend(&be);
    assert_eq!(decode(&le_bom, Encoding::Utf16Le).unwrap(), "Hola ñ 😀");
    assert_eq!(decode(&be_bom, Encoding::Utf16Be).unwrap(), "Hola ñ 😀");
    assert_eq!(decode(&hex("FF FE"), Encoding::Utf16Le).unwrap(), "");
    assert_eq!(decode(&[], Encoding::Utf16Be).unwrap(), "");
}

#[test]
fn h31_utf16_other_endian_bom_not_stripped() {
    // FE FF leído como LE = U+FFFE (no carácter, pero escalar válido)
    assert_eq!(decode(&hex("FE FF 41 00"), Encoding::Utf16Le).unwrap(), "\u{FFFE}A");
}

#[test]
fn h32_utf16_odd_length() {
    assert_eq!(
        decode(&hex("41 00 42"), Encoding::Utf16Le),
        Err(DecodeError::OddLength { encoding: Encoding::Utf16Le })
    );
    assert_eq!(
        decode(&hex("FE FF 00"), Encoding::Utf16Be),
        Err(DecodeError::OddLength { encoding: Encoding::Utf16Be })
    );
}

#[test]
fn h33_utf16_unpaired_surrogates() {
    // LE, sin BOM: A, high surrogate solo, B → offset 2
    assert_eq!(
        decode(&hex("41 00 3D D8 42 00"), Encoding::Utf16Le),
        Err(DecodeError::InvalidSequence { encoding: Encoding::Utf16Le, offset: 2 })
    );
    // LE con BOM: low surrogate suelto en la posición 4 de la entrada original
    assert_eq!(
        decode(&hex("FF FE 41 00 00 DE"), Encoding::Utf16Le),
        Err(DecodeError::InvalidSequence { encoding: Encoding::Utf16Le, offset: 4 })
    );
    // BE: high surrogate al final
    assert_eq!(
        decode(&hex("00 41 D8 3D"), Encoding::Utf16Be),
        Err(DecodeError::InvalidSequence { encoding: Encoding::Utf16Be, offset: 2 })
    );
}

// ---------- decode: 8 bits ----------

#[test]
fn h40_ascii() {
    assert_eq!(decode(b"Hola\r\nmundo\t~", Encoding::Ascii).unwrap(), "Hola\r\nmundo\t~");
    assert_eq!(
        decode(&hex("41 42 43 80"), Encoding::Ascii),
        Err(DecodeError::InvalidSequence { encoding: Encoding::Ascii, offset: 3 })
    );
    assert_eq!(
        decode(&hex("C3 B1"), Encoding::Ascii),
        Err(DecodeError::InvalidSequence { encoding: Encoding::Ascii, offset: 0 })
    );
}

#[test]
fn h41_latin1_is_real_latin1() {
    let all: Vec<u8> = (0u8..=255).collect();
    let text = decode(&all, Encoding::Iso8859_1).unwrap();
    let expected: String = (0u32..=255).map(|c| char::from_u32(c).unwrap()).collect();
    assert_eq!(text, expected);
    assert_eq!(decode(&hex("80 9F A4"), Encoding::Iso8859_1).unwrap(), "\u{80}\u{9F}\u{A4}");
}

#[test]
fn h42_latin9() {
    let bytes = hex("d1 61 6e 64 fa 20 a4 20 a6 20 a8 20 b4 20 b8 20 bc 20 bd 20 be");
    assert_eq!(decode(&bytes, Encoding::Iso8859_15).unwrap(), "Ñandú € Š š Ž ž Œ œ Ÿ");
    let all: Vec<u8> = (0u8..=255).collect();
    assert_eq!(decode(&all, Encoding::Iso8859_15).unwrap().chars().count(), 256);
}

#[test]
fn h43_windows1252() {
    let bytes = hex(
        "d1 61 6e 64 fa 3a 20 61 f1 6f 2c 20 70 69 6e 67 fc 69 6e 6f 2c 20 80 35 2e 20 \
         8c 75 76 72 65 20 97 20 ab 63 69 74 61 bb",
    );
    assert_eq!(
        decode(&bytes, Encoding::Windows1252).unwrap(),
        "Ñandú: año, pingüino, €5. Œuvre — «cita»"
    );
    // Bytes no definidos en Windows-1252: WHATWG los pasa a C1
    assert_eq!(
        decode(&hex("81 8D 8F 90 9D"), Encoding::Windows1252).unwrap(),
        "\u{81}\u{8D}\u{8F}\u{90}\u{9D}"
    );
    let all: Vec<u8> = (0u8..=255).collect();
    assert_eq!(decode(&all, Encoding::Windows1252).unwrap().chars().count(), 256);
}

#[test]
fn h44_macroman() {
    let bytes = hex(
        "4d 61 96 61 6e 61 20 8e 6c 20 63 6f 6d 69 97 20 63 72 8f 6d 65 20 62 72 9e 6c 8e 65 20 a5 20 82",
    );
    assert_eq!(
        decode(&bytes, Encoding::MacRoman).unwrap(),
        "Mañana él comió crème brûlée • Ç"
    );
    let all: Vec<u8> = (0u8..=255).collect();
    assert_eq!(decode(&all, Encoding::MacRoman).unwrap().chars().count(), 256);
}

#[test]
fn h45_eight_bit_encodings_never_fail_and_preserve_length() {
    let all: Vec<u8> = (0u8..=255).collect();
    for e in [
        Encoding::Iso8859_1,
        Encoding::Iso8859_15,
        Encoding::Windows1252,
        Encoding::MacRoman,
    ] {
        let t = decode(&all, e).unwrap();
        assert_eq!(t.chars().count(), 256, "{e:?}");
        assert!(!t.contains('\u{FFFD}'), "{e:?}");
    }
}

#[test]
fn h46_line_endings_untouched() {
    for e in Encoding::ALL {
        if matches!(e, Encoding::Utf16Le | Encoding::Utf16Be) {
            continue;
        }
        assert_eq!(decode(b"a\r\nb\rc\n", e).unwrap(), "a\r\nb\rc\n", "{e:?}");
    }
    assert_eq!(
        decode(&hex("61 00 0D 00 0A 00 62 00"), Encoding::Utf16Le).unwrap(),
        "a\r\nb"
    );
}

// ---------- decode_auto ----------

#[test]
fn h50_decode_auto_paths() {
    assert_eq!(
        decode_auto(&hex("EF BB BF 41")).unwrap(),
        Decoded { text: "A".into(), encoding: Encoding::Utf8Bom, method: DetectionMethod::Bom }
    );
    assert_eq!(
        decode_auto(&hex("FF FE 41 00")).unwrap(),
        Decoded { text: "A".into(), encoding: Encoding::Utf16Le, method: DetectionMethod::Bom }
    );
    assert_eq!(
        decode_auto(&hex("FE FF 00 41")).unwrap(),
        Decoded { text: "A".into(), encoding: Encoding::Utf16Be, method: DetectionMethod::Bom }
    );
    assert_eq!(
        decode_auto("año".as_bytes()).unwrap(),
        Decoded {
            text: "año".into(),
            encoding: Encoding::Utf8,
            method: DetectionMethod::Utf8Valid
        }
    );
    assert_eq!(
        decode_auto(&[]).unwrap(),
        Decoded { text: String::new(), encoding: Encoding::Utf8, method: DetectionMethod::Utf8Valid }
    );
}

#[test]
fn h51_decode_auto_windows1252_text() {
    let text = "La niña soñó con el pingüino del año pasado; «perfecto» — dijo él. \
                Mañana habrá más canciones en la montaña.\r\n";
    let (bytes, _, _) = encoding_rs::WINDOWS_1252.encode(text);
    let d = decode_auto(&bytes).unwrap();
    assert_eq!(d.encoding, Encoding::Windows1252);
    assert_eq!(d.method, DetectionMethod::Heuristic);
    assert_eq!(d.text, text);
}

#[test]
fn h52_decode_auto_errors_after_bom() {
    assert_eq!(
        decode_auto(&hex("FF FE 41")),
        Err(DecodeError::OddLength { encoding: Encoding::Utf16Le })
    );
    assert_eq!(
        decode_auto(&hex("EF BB BF 41 FF")),
        Err(DecodeError::InvalidSequence { encoding: Encoding::Utf8Bom, offset: 4 })
    );
}

#[test]
fn h53_decode_auto_never_fails_without_bom() {
    let samples: Vec<Vec<u8>> = vec![
        hex("FF 00 FE"),
        hex("80 81 82"),
        hex("C3 28"),
        (0u8..=255).filter(|b| *b != 0xEF && *b != 0xFE && *b != 0xFF).collect(),
    ];
    for s in samples {
        assert!(decode_auto(&s).is_ok(), "{s:02X?}");
    }
}

// ---------- errores ----------

#[test]
fn h60_error_traits() {
    let e = DecodeError::OddLength { encoding: Encoding::Utf16Be };
    let msg = e.to_string();
    assert!(!msg.is_empty());
    let boxed: Box<dyn std::error::Error> = Box::new(e.clone());
    assert_eq!(boxed.to_string(), msg);
    let e2 = DecodeError::InvalidSequence { encoding: Encoding::Ascii, offset: 7 };
    assert!(e2.to_string().contains('7'), "el mensaje debería incluir el offset: {e2}");
}
