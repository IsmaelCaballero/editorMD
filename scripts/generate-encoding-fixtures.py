"""Genera los ficheros de prueba de M09 TextCodec (F2 paso 2/8).

Uso: python scripts/generate-encoding-fixtures.py

Cada fichero <codificación>-<fin de línea>.txt contiene el mismo texto codificado
con esa codificación, ese fin de línea y su BOM (si lo lleva). Las pruebas de Rust
comprueban que leer y volver a escribir cada fichero da exactamente los mismos bytes.
Los ficheros están en fixtures/encodings/; git los trata como binarios (.gitattributes), así que no altera ni un byte.
"""

from pathlib import Path

HERE = Path(__file__).resolve().parent.parent / "fixtures" / "encodings"

# Texto representable en todas las codificaciones de 8 bits (Latin-1, Latin-9, Windows-1252, Mac Roman).
LATIN = "# Título\n\nAño, pingüino, ¿qué tal? ¡Olé!\nÚltima línea con ñ y ç.\n"
ASCII = "# Title\n\nPlain ASCII text.\nLast line.\n"

# id de la codificación → (codec de Python, BOM, texto)
ENCODINGS = {
    "utf-8": ("utf-8", b"", LATIN + "Emoji 😀 y € fuera de Latin-1.\n"),
    "utf-8-bom": ("utf-8", b"\xef\xbb\xbf", LATIN + "Emoji 😀 y € fuera de Latin-1.\n"),
    "utf-16le": ("utf-16-le", b"\xff\xfe", LATIN + "Emoji 😀 y € fuera de Latin-1.\n"),
    "utf-16be": ("utf-16-be", b"\xfe\xff", LATIN + "Emoji 😀 y € fuera de Latin-1.\n"),
    "ascii": ("ascii", b"", ASCII),
    "iso-8859-1": ("latin-1", b"", LATIN),
    "iso-8859-15": ("iso8859-15", b"", LATIN + "Euro: €.\n"),
    "windows-1252": ("cp1252", b"", LATIN + "Euro: € y comillas “así”.\n"),
    "macintosh": ("mac_roman", b"", LATIN),
}
EOLS = {"lf": "\n", "crlf": "\r\n", "cr": "\r"}

for enc_id, (codec, bom, text) in ENCODINGS.items():
    for eol_id, eol in EOLS.items():
        data = bom + text.replace("\n", eol).encode(codec)
        (HERE / f"{enc_id}-{eol_id}.txt").write_bytes(data)

# Fin de línea mixto: 2 LF, 1 CRLF y 1 CR (UTF-8).
(HERE / "mixed-eol.txt").write_bytes("uno\ndos\r\ntres\rcuatro\ncinco".encode("utf-8"))
