# RQ.1 · Tarea T1 — M09 `TextCodec`: detección y decodificación (F2 paso 1/8)

> Especificación cerrada del experimento RQ.1 (`docs/ai-usage/README.md` §4.2).
> Es **el mismo encargo** para todas las ejecuciones (3 × Opus 5.5 y 3 × Sonnet 5.5).
> La aceptación se decide con **pruebas ocultas** que no forman parte del repositorio,
> además de `cargo test`, clippy (pedantic, `-D warnings`) y rustfmt.

## 1. Contexto

editorMD es un editor Markdown (Rust + Tauri 2, ver `CONTEXT.md` y `PLAN.md` §2.5).
El componente **M09 `TextCodec`** (Modelo, Rust) convierte bytes de un fichero de texto en
texto Unicode y viceversa. Esta tarea cubre **solo la lectura**: detectar la codificación y
decodificar. La escritura, los finales de línea y el informe de pérdidas son tareas posteriores
(no los implementes).

## 2. Qué hay que entregar

1. Nuevo módulo `src-tauri/src/model/text_codec.rs`, declarado en `src-tauri/src/model/mod.rs`
   como `pub mod text_codec;`. Ruta pública: `editormd_lib::model::text_codec`.
2. Dependencias nuevas en `src-tauri/Cargo.toml`: `encoding_rs` y `chardetng`
   (y las que necesites, justificadas). Actualiza `Cargo.lock`.
3. Pruebas unitarias en el propio módulo (`#[cfg(test)] mod tests`) que cubran esta
   especificación, incluidos los casos borde. Opcionalmente, ficheros de ejemplo en
   `fixtures/encodings/` (git los trata como binarios).
4. Documentación rustdoc de todo elemento público (el crate tiene `#![warn(missing_docs)]`),
   con al menos un ejemplo (doctest) en `detect` y en `decode`. El fichero empieza con un
   comentario `//! **M09 `TextCodec`** (Modelo · Rust) — …` como `app_info.rs`.
   Documentación y comentarios en **español**; identificadores en inglés.

## 3. API pública (obligatoria, exacta)

```rust
/// Codificaciones soportadas (D-15).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Encoding {
    Utf8,        // id "utf-8"
    Utf8Bom,     // id "utf-8-bom"
    Utf16Le,     // id "utf-16le"
    Utf16Be,     // id "utf-16be"
    Ascii,       // id "ascii"
    Iso8859_1,   // id "iso-8859-1"
    Iso8859_15,  // id "iso-8859-15"
    Windows1252, // id "windows-1252"
    MacRoman,    // id "macintosh"
}

impl Encoding {
    /// Las 9 codificaciones, en el orden de la declaración.
    pub const ALL: [Encoding; 9];
    /// Identificador estable (el mismo que usa el frontend en `src/model/document.ts`).
    pub fn id(self) -> &'static str;
    /// Inverso de `id`; `None` si el identificador no existe (distingue mayúsculas).
    pub fn from_id(id: &str) -> Option<Encoding>;
    /// BOM de la codificación: EF BB BF (Utf8Bom), FF FE (Utf16Le), FE FF (Utf16Be);
    /// vacío en las demás.
    pub fn bom(self) -> &'static [u8];
}

/// Cómo se ha decidido la codificación.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum DetectionMethod { Bom, Utf8Valid, Heuristic, Fallback }

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct Detection { pub encoding: Encoding, pub method: DetectionMethod }

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Decoded { pub text: String, pub encoding: Encoding, pub method: DetectionMethod }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// Secuencia no válida; `offset` = posición (en bytes, contando el BOM si lo hay)
    /// del primer byte de la primera secuencia no válida en la entrada original.
    InvalidSequence { encoding: Encoding, offset: usize },
    /// UTF-16 con un número impar de bytes (tras quitar el BOM).
    OddLength { encoding: Encoding },
}
// DecodeError implementa std::fmt::Display (mensaje en español que incluye el id de la
// codificación y, en InvalidSequence, el offset) y std::error::Error.

pub fn detect(bytes: &[u8]) -> Detection;
pub fn decode(bytes: &[u8], encoding: Encoding) -> Result<String, DecodeError>;
pub fn decode_auto(bytes: &[u8]) -> Result<Decoded, DecodeError>;
```

**Serialización (serde):** `Encoding` se serializa y deserializa como su `id`
(p. ej. `"utf-8-bom"`). `DetectionMethod` en kebab-case (`"bom"`, `"utf8-valid"`,
`"heuristic"`, `"fallback"`). `Detection` y `Decoded` con campos en camelCase.

## 4. Comportamiento

### 4.1 `detect(bytes)`, en este orden
1. **BOM**: empieza por `EF BB BF` → `Utf8Bom`; por `FF FE` → `Utf16Le`; por `FE FF` →
   `Utf16Be`. Método `Bom`. (No hay que distinguir UTF-32.)
2. **UTF-8 válido** (incluidos el fichero vacío y el ASCII puro) → `Utf8`, método `Utf8Valid`.
   Nunca se devuelve `Ascii` ni `Iso8859_1` al detectar.
3. **Heurística** con `chardetng` sobre toda la entrada (sin TLD, sin permitir UTF-8):
   si el resultado es windows-1252 → `Windows1252`; ISO-8859-15 → `Iso8859_15`;
   macintosh → `MacRoman`; método `Heuristic`.
4. Cualquier otro resultado de la heurística (p. ej. windows-1251, Shift_JIS) →
   `Windows1252`, método `Fallback`.

### 4.2 `decode(bytes, encoding)` — estricta y sin pérdidas silenciosas
| Codificación | Regla |
|---|---|
| `Utf8` | UTF-8 estricto. **No** quita un BOM inicial (quedaría `U+FEFF` en el texto). |
| `Utf8Bom` | Quita `EF BB BF` si está al principio (si no está, decodifica igual). UTF-8 estricto. |
| `Utf16Le` / `Utf16Be` | Quita su BOM si está. Número impar de bytes restantes → `OddLength`. Sustituto (*surrogate*) sin pareja → `InvalidSequence` con el offset del primer byte de esa unidad. Un BOM de la otra endianness **no** se quita (se decodifica como carácter). |
| `Ascii` | Cualquier byte ≥ `0x80` → `InvalidSequence` con su offset. |
| `Iso8859_1` | Latin-1 **real**: cada byte `b` → `U+00bb` (incluidos `0x80`-`0x9F`, controles C1). Nunca falla. Ojo: `encoding_rs` trata «ISO-8859-1» como windows-1252, así que no sirve aquí. |
| `Iso8859_15` | Latin-9 (`encoding_rs::ISO_8859_15`): como Latin-1 salvo `A4`=€, `A6`=Š, `A8`=š, `B4`=Ž, `B8`=ž, `BC`=Œ, `BD`=œ, `BE`=Ÿ. Nunca falla. |
| `Windows1252` | `encoding_rs::WINDOWS_1252` (WHATWG: los bytes `81 8D 8F 90 9D` dan `U+0081`…). Nunca falla. |
| `MacRoman` | `encoding_rs::MACINTOSH`. Nunca falla. |

En UTF-8, el `offset` es el del primer byte de la primera secuencia no válida
(equivale a `valid_up_to()` de `std::str::Utf8Error` más la longitud del BOM quitado).
No se sustituye nada por `U+FFFD`: o se decodifica todo o es un error.
Los bytes `0x00` son válidos en todas las codificaciones.

### 4.3 `decode_auto(bytes)`
`detect` y después `decode` con la codificación detectada. Devuelve el `Decoded` con la
codificación y el método de la detección, o el error de `decode` (solo puede fallar tras
un BOM, p. ej. `FF FE` seguido de un número impar de bytes, o `EF BB BF` seguido de UTF-8
no válido).

## 5. Criterios de aceptación
- Pruebas ocultas de aceptación (las ejecuta el orquestador al final) en verde.
- `cargo test --manifest-path src-tauri/Cargo.toml` en verde (incluidos los doctests).
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` sin avisos
  (el crate ya activa `pedantic`).
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check` sin cambios.
- Sin `unsafe` (el crate tiene `unsafe_code = "forbid"`), sin `unwrap()`/`expect()` fuera de
  las pruebas.
- Cobertura alta de las ramas del módulo con tus propias pruebas.

## 6. Forma de trabajar
- Trabaja **solo** dentro de tu *worktree*. No toques otros directorios del disco.
- Proyecto con **TDD**: primer commit con las pruebas en rojo (`test(M09): …`), después la
  implementación en verde (`feat(M09): …`) y, si hace falta, `refactor`/`style`. Mensajes en
  Conventional Commits, en español.
- No modifiques documentación del proyecto (PLAN.md, STATUS.md, web…): eso lo hace el orquestador.
- No hagas `push` ni abras PR.
- Para `cargo`, usa un *timeout* largo (hasta 600 000 ms): la primera compilación de Tauri
  en un *worktree* nuevo tarda varios minutos.
