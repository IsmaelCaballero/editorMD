// Tests de M07 MarkdownCodec (TDD: escritos antes de la implementación).
// Objetivo: que pasar por el editor WYSIWYG (Markdown → árbol → Markdown)
// NUNCA cambie el significado del documento, y que los bloques R y el
// front matter salgan exactamente igual que entraron.
import { readdirSync, readFileSync } from 'node:fs'
import { join } from 'node:path'
import type { Code } from 'mdast'
import { describe, expect, it } from 'vitest'
import {
  equivalent,
  isRmdChunk,
  normalize,
  parse,
  rmdChunkHeader,
  stringify,
} from '../../src/model/markdown-codec'

const CORPUS_DIR = join(__dirname, '..', 'corpus')
const corpus = readdirSync(CORPUS_DIR)
  .filter((f) => /\.(md|Rmd)$/.test(f))
  .sort()
  .map((name) => [name, readFileSync(join(CORPUS_DIR, name), 'utf-8')] as const)

describe('corpus de fidelidad', () => {
  it('contiene al menos 20 documentos', () => {
    expect(corpus.length).toBeGreaterThanOrEqual(20)
  })

  describe.each(corpus)('%s', (_name, original) => {
    it('ida y vuelta sin cambios de significado', () => {
      expect(equivalent(original, normalize(original))).toBe(true)
    })

    it('normalizar es idempotente (una segunda pasada no cambia nada)', () => {
      const once = normalize(original)
      expect(normalize(once)).toBe(once)
    })

    it('la salida usa solo LF', () => {
      expect(normalize(original)).not.toMatch(/\r/)
    })
  })
})

describe('estilo de salida canónico (diferencias de estilo documentadas)', () => {
  it.each([
    ['_cursiva_\n', '*cursiva*\n'],
    ['__negrita__\n', '**negrita**\n'],
    ['* uno\n* dos\n', '- uno\n- dos\n'],
    ['+ uno\n', '- uno\n'],
    ['Título\n======\n', '# Título\n'],
    ['Título\n------\n', '## Título\n'],
    ['***\n', '---\n'],
    ['~~~\ncódigo\n~~~\n', '```\ncódigo\n```\n'],
    ['    indentado\n', '```\nindentado\n```\n'],
  ])('%j → %j', (input, expected) => {
    expect(normalize(input)).toBe(expected)
    expect(equivalent(input, expected)).toBe(true)
  })

  it('el Markdown ya canónico no cambia', () => {
    const canonical = '# Título\n\n- uno\n- dos\n\n**negrita** y *cursiva*\n'
    expect(normalize(canonical)).toBe(canonical)
  })
})

describe('GFM', () => {
  it('conserva tablas con su alineación', () => {
    const md = '| a | b | c |\n| :- | :-: | -: |\n| 1 | 2 | 3 |\n'
    const out = normalize(md)
    expect(out).toMatch(/\| :-+ \| :-+: \| -+: \|/)
    expect(equivalent(md, out)).toBe(true)
  })

  it('conserva listas de tareas marcadas y sin marcar', () => {
    const out = normalize('- [ ] pendiente\n- [x] hecha\n')
    expect(out).toBe('- [ ] pendiente\n- [x] hecha\n')
  })

  it('conserva el tachado', () => {
    expect(normalize('~~tachado~~\n')).toBe('~~tachado~~\n')
  })

  it('reconoce los nodos GFM al analizar', () => {
    const types = parse('| a |\n| - |\n| 1 |\n\n~~x~~\n').children.map((n) => n.type)
    expect(types).toContain('table')
  })
})

describe('front matter YAML', () => {
  const fm = '---\ntitle: "Informe"\ndate: 2026-10-04\ntags: [a, b]\n---\n'

  it('se reconoce como nodo yaml', () => {
    expect(parse(fm + '\n# Hola\n').children[0].type).toBe('yaml')
  })

  it('se conserva exactamente igual', () => {
    expect(normalize(fm + '\n# Hola\n').startsWith(fm)).toBe(true)
  })

  it('un --- que no está al principio sigue siendo una línea horizontal', () => {
    expect(parse('Texto\n\n---\n\nMás\n').children.map((n) => n.type)).toEqual([
      'paragraph',
      'thematicBreak',
      'paragraph',
    ])
  })
})

describe('R Markdown', () => {
  const chunkOf = (md: string) => parse(md).children[0] as Code

  it.each([
    '```{r}\nx <- 1\n```\n',
    '```{r setup, include=FALSE}\nknitr::opts_chunk$set(echo = TRUE)\n```\n',
    '```{r grafico, echo=FALSE, fig.cap="Velocidad"}\nplot(cars)\n```\n',
    '```{python}\nimport pandas as pd\n```\n',
  ])('el bloque %j sale idéntico', (md) => {
    expect(normalize(md)).toBe(md)
  })

  it('isRmdChunk distingue bloques R de bloques de código normales', () => {
    expect(isRmdChunk(chunkOf('```{r setup}\nx\n```\n'))).toBe(true)
    expect(isRmdChunk(chunkOf('```{python}\nx\n```\n'))).toBe(true)
    expect(isRmdChunk(chunkOf('```r\nx\n```\n'))).toBe(false)
    expect(isRmdChunk(chunkOf('```\nx\n```\n'))).toBe(false)
  })

  it('rmdChunkHeader devuelve la cabecera completa con opciones', () => {
    expect(rmdChunkHeader(chunkOf('```{r a, echo=FALSE}\nx\n```\n'))).toBe('{r a, echo=FALSE}')
    expect(rmdChunkHeader(chunkOf('```js\nx\n```\n'))).toBeNull()
  })

  it('el código R en línea se conserva', () => {
    const md = 'La media es `r mean(x)` km/h.\n'
    expect(normalize(md)).toBe(md)
  })
})

describe('equivalent', () => {
  it('detecta cambios reales de contenido', () => {
    expect(equivalent('# Hola\n', '# Adiós\n')).toBe(false)
    expect(equivalent('*a*\n', '**a**\n')).toBe(false)
    expect(equivalent('- [ ] t\n', '- [x] t\n')).toBe(false)
  })

  it('ignora diferencias de estilo y posición', () => {
    expect(equivalent('* a\n* b\n', '- a\n- b\n')).toBe(true)
    expect(equivalent('# T\n\n\n\nTexto\n', '# T\n\nTexto\n')).toBe(true)
  })
})

describe('stringify', () => {
  it('termina en salto de línea y no deja líneas en blanco extra al final', () => {
    expect(stringify(parse('Hola'))).toBe('Hola\n')
  })

  it('un documento vacío sigue vacío', () => {
    expect(normalize('')).toBe('')
  })
})
