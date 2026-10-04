/**
 * **M07 `MarkdownCodec`** (Modelo · TS): conversión Markdown ⇄ árbol (mdast).
 *
 * Es la pieza crítica del modo WYSIWYG: el editor trabaja con un árbol y, al
 * guardar, lo vuelve a convertir en texto. Este módulo fija **cómo** se hace
 * esa conversión para que:
 *
 * 1. **No cambie el significado** del documento (comprobado con un corpus de
 *    fidelidad en `tests/corpus/`).
 * 2. El estilo de salida sea **canónico y estable** (idempotente).
 * 3. El *front matter* YAML y los bloques de **R Markdown** salgan intactos.
 *
 * Dialecto: CommonMark + GFM (tablas, tachado, tareas, autoenlaces, notas al
 * pie) + front matter YAML (decisión D-04).
 *
 * @remarks
 * Diferencias de estilo conocidas (el significado no cambia):
 *
 * | Entrada | Salida |
 * |---|---|
 * | `_cursiva_`, `__negrita__` | `*cursiva*`, `**negrita**` |
 * | `* ítem`, `+ ítem` | `- ítem` |
 * | Títulos subrayados (setext) | Títulos con `#` (ATX) |
 * | `***`, `___` | `---` |
 * | `~~~` o código indentado | <code>```</code> |
 * | `1)` en listas numeradas | `1.` |
 * | Salto duro con dos espacios | `\` al final de la línea |
 * | Entidades HTML (`&copy;`) | El carácter (`©`) |
 *
 * v0.2.0 añade {@link splitFrontMatter}, {@link joinFrontMatter} y
 * {@link remarkCodeMetaIntoLang} para proteger el front matter y las cabeceras
 * de bloque en editores WYSIWYG que no los soportan.
 *
 * @packageDocumentation
 */
import type { Code, Nodes, Root } from 'mdast'
import remarkFrontmatter from 'remark-frontmatter'
import remarkGfm from 'remark-gfm'
import remarkParse from 'remark-parse'
import remarkStringify, { type Options as StringifyOptions } from 'remark-stringify'
import { defaultHandlers, type Handle } from 'mdast-util-to-markdown'
import { unified, type Processor } from 'unified'

/**
 * Opciones de serialización: definen el **estilo canónico** del proyecto.
 * Las comparte V02 (editor Milkdown) para que el editor y los tests usen
 * exactamente la misma conversión.
 */
export const STRINGIFY_OPTIONS: Readonly<StringifyOptions> = Object.freeze({
  bullet: '-',
  bulletOther: '*',
  emphasis: '*',
  strong: '*',
  fence: '`',
  fences: true,
  rule: '-',
  ruleRepetition: 3,
  setext: false,
  closeAtx: false,
  listItemIndent: 'one',
  incrementListMarker: true,
})

const processor = unified()
  .use(remarkParse)
  .use(remarkGfm)
  .use(remarkFrontmatter, ['yaml'])
  .use(remarkStringify, STRINGIFY_OPTIONS)
  .freeze()

/**
 * Analiza Markdown y devuelve su árbol sintáctico (mdast).
 *
 * @param markdown - Texto Markdown (con `\n` como fin de línea).
 * @returns Nodo raíz del árbol, con posiciones de origen.
 */
export function parse(markdown: string): Root {
  return processor.parse(markdown)
}

/**
 * Convierte un árbol mdast en Markdown con el estilo canónico.
 *
 * @param tree - Árbol a serializar.
 * @returns Markdown terminado en `\n`, o `""` si el documento está vacío.
 */
export function stringify(tree: Root): string {
  if (tree.children.length === 0) return ''
  return processor.stringify(tree)
}

/**
 * Pasa un documento por el ciclo completo Markdown → árbol → Markdown, como al
 * guardar desde el editor WYSIWYG.
 *
 * @param markdown - Documento original.
 * @returns El mismo documento con el estilo canónico.
 *
 * @example
 * ```ts
 * normalize('* uno\n* dos\n') // '- uno\n- dos\n'
 * ```
 */
export function normalize(markdown: string): string {
  return stringify(parse(markdown))
}

/** Copia del árbol sin la información de posición (líneas y columnas de origen). */
function withoutPositions(node: Nodes): unknown {
  const { position: _position, ...rest } = node as Nodes & { children?: Nodes[] }
  if ('children' in rest && Array.isArray(rest.children)) {
    return { ...rest, children: rest.children.map(withoutPositions) }
  }
  return rest
}

/**
 * Indica si dos documentos son **semánticamente equivalentes**: producen el
 * mismo árbol, aunque difieran en estilo (`*` frente a `_`, setext frente a
 * ATX, líneas en blanco…).
 *
 * @param a - Primer documento Markdown.
 * @param b - Segundo documento Markdown.
 * @returns `true` si significan lo mismo.
 */
export function equivalent(a: string, b: string): boolean {
  return JSON.stringify(withoutPositions(parse(a))) === JSON.stringify(withoutPositions(parse(b)))
}

/**
 * Indica si un bloque de código es un *chunk* de R Markdown
 * (<code>```{r ...}</code>, <code>```{python}</code>…).
 *
 * @param node - Nodo de código.
 * @returns `true` si su información de lenguaje empieza por `{`.
 *
 * @remarks
 * El editor los muestra como bloques opacos: su cabecera y su contenido se
 * guardan sin modificar y nunca se ejecutan.
 */
export function isRmdChunk(node: Code): boolean {
  return node.lang?.startsWith('{') ?? false
}

/**
 * Devuelve la cabecera completa de un *chunk* de R Markdown.
 *
 * @param node - Nodo de código.
 * @returns P. ej. `"{r grafico, echo=FALSE}"`, o `null` si no es un chunk.
 *
 * @remarks
 * remark separa la información del bloque en `lang` (hasta el primer espacio)
 * y `meta` (el resto); aquí se vuelven a unir.
 */
export function rmdChunkHeader(node: Code): string | null {
  if (!isRmdChunk(node)) return null
  return node.meta ? `${node.lang} ${node.meta}` : (node.lang as string)
}

// ---------------------------------------------------------------------------
// v0.2.0 — Utilidades para editores WYSIWYG que no conocen el front matter
// ni el `meta` de los bloques de código (p. ej. Milkdown).
// ---------------------------------------------------------------------------

/** Resultado de {@link splitFrontMatter}. */
export interface FrontMatterSplit {
  /**
   * Bloque YAML inicial tal cual (delimitadores `---` y líneas en blanco
   * posteriores incluidos), o `""` si el documento no tiene.
   */
  readonly frontMatter: string
  /** Resto del documento, que es lo que se edita en el WYSIWYG. */
  readonly body: string
}

/** Front matter YAML: `---` en la primera línea, hasta el siguiente `---`, más las líneas en blanco que lo siguen. */
const FRONT_MATTER_RE = /^---\r?\n(?:[\s\S]*?\r?\n)?---[ \t]*(?:\r?\n|$)(?:[ \t]*\r?\n)*/

/**
 * Separa el *front matter* YAML del cuerpo del documento.
 *
 * @remarks
 * El editor WYSIWYG solo recibe el cuerpo; el front matter se guarda aparte y
 * se vuelve a unir con {@link joinFrontMatter}, así sale **byte a byte igual**.
 *
 * @param markdown - Documento completo.
 * @returns El front matter (o `""`) y el cuerpo.
 *
 * @example
 * ```ts
 * splitFrontMatter('---\ntitle: x\n---\n\n# Hola\n')
 * // { frontMatter: '---\ntitle: x\n---\n\n', body: '# Hola\n' }
 * ```
 */
export function splitFrontMatter(markdown: string): FrontMatterSplit {
  const match = FRONT_MATTER_RE.exec(markdown)
  if (!match) return { frontMatter: '', body: markdown }
  return { frontMatter: match[0], body: markdown.slice(match[0].length) }
}

/**
 * Operación inversa de {@link splitFrontMatter}.
 *
 * @param frontMatter - Bloque devuelto por `splitFrontMatter` (o `""`).
 * @param body - Cuerpo, posiblemente editado.
 * @returns El documento completo.
 */
export function joinFrontMatter(frontMatter: string, body: string): string {
  return frontMatter + body
}

/** Recorre el árbol y une `meta` dentro de `lang` en cada bloque de código. */
function mergeCodeMeta(node: Nodes): void {
  if (node.type === 'code' && node.meta) {
    node.lang = node.lang ? `${node.lang} ${node.meta}` : node.meta
    node.meta = null
  }
  if ('children' in node) {
    for (const child of node.children) mergeCodeMeta(child)
  }
}

/**
 * Manejador de serialización para bloques de código: si `lang` contiene la
 * cabecera completa (p. ej. `{r setup, include=FALSE}`), la vuelve a separar en
 * `lang` + `meta` antes de delegar en el manejador estándar. Sin esto,
 * remark-stringify escaparía los espacios (`{r&#x20;setup,...}`), porque en
 * CommonMark el primer espacio separa el lenguaje del `meta`.
 */
const codeHandler: Handle = (node, parent, state, info) => {
  const code = node as Code
  const space = code.lang && !code.meta ? code.lang.search(/\s/) : -1
  const fixed =
    space > 0 && code.lang
      ? { ...code, lang: code.lang.slice(0, space), meta: code.lang.slice(space + 1) }
      : code
  return defaultHandlers.code(fixed, parent, state, info)
}

/**
 * Plugin de remark (unified) que protege la cabecera completa de los bloques
 * de código (<code>```{r setup, include=FALSE}</code>, <code>```js title="a.js"</code>)
 * en editores que solo guardan el lenguaje.
 *
 * @remarks
 * Actúa en los dos sentidos, igual que funciona Milkdown:
 *
 * - **Al cargar** (transformador, `runSync`): une `meta` dentro de `lang`,
 *   que es el único atributo que conserva el nodo de código de Milkdown.
 * - **Al guardar** (extensión de `mdast-util-to-markdown`, usada por `stringify`):
 *   vuelve a separar `lang` y `meta`, de modo que la salida es idéntica al original.
 *
 * Se registra en el editor con `$remark` (V02).
 *
 * @example
 * ```ts
 * unified().use(remarkParse).use(remarkStringify).use(remarkCodeMetaIntoLang)
 * ```
 */
export function remarkCodeMetaIntoLang(this: Processor): (tree: Root) => void {
  const data = this.data()
  data.toMarkdownExtensions ??= []
  data.toMarkdownExtensions.push({ handlers: { code: codeHandler } })
  return (tree) => mergeCodeMeta(tree)
}
