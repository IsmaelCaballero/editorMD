/**
 * **V02 `WysiwygEditorView`** (Vista · TS): editor WYSIWYG basado en
 * [Milkdown *kit*](https://milkdown.dev) (ProseMirror + remark).
 *
 * Implementa el puerto {@link IEditorView}: el controlador solo intercambia
 * Markdown en texto con él y nunca ve ProseMirror ni Milkdown.
 *
 * Usa M07 (`MarkdownCodec`) para que el editor convierta igual que los tests:
 * - mismo estilo de salida ({@link STRINGIFY_OPTIONS});
 * - el *front matter* YAML no entra en el editor: se guarda aparte y se reinserta
 *   intacto ({@link splitFrontMatter});
 * - las cabeceras de bloque como <code>```{r setup, include=FALSE}</code> se
 *   protegen con {@link remarkCodeMetaIntoLang}.
 *
 * @remarks
 * Limitaciones conocidas de Milkdown 7 (el resultado se ve igual, pero el texto cambia):
 * - Los enlaces de referencia (`[texto][ref]` + `[ref]: url`) se guardan como enlaces en línea.
 * - Las URL sueltas (autoenlace GFM) se guardan entre `<` `>`.
 *
 * @packageDocumentation
 */
import {
  defaultValueCtx,
  Editor,
  editorViewCtx,
  remarkStringifyOptionsCtx,
  rootCtx,
} from '@milkdown/kit/core'
import { clipboard } from '@milkdown/kit/plugin/clipboard'
import { history, redoCommand, undoCommand } from '@milkdown/kit/plugin/history'
import { listener, listenerCtx } from '@milkdown/kit/plugin/listener'
import { commonmark } from '@milkdown/kit/preset/commonmark'
import { gfm } from '@milkdown/kit/preset/gfm'
import { $remark, callCommand, getMarkdown, replaceAll } from '@milkdown/kit/utils'
import type { IEditorView, Unsubscribe } from '../controller/ports'
import {
  joinFrontMatter,
  remarkCodeMetaIntoLang,
  splitFrontMatter,
  STRINGIFY_OPTIONS,
} from '../model/markdown-codec'

/**
 * Plugin de remark de M07 registrado en Milkdown. La conversión de tipo es
 * necesaria porque los tipos de plugin de Milkdown y de unified no coinciden
 * exactamente, aunque en ejecución son la misma clase de función.
 */
const codeMetaPlugin = $remark('editormdCodeMetaIntoLang', () => remarkCodeMetaIntoLang as never)

/**
 * Editor WYSIWYG que cumple {@link IEditorView}.
 *
 * @example
 * ```ts
 * const view = await MilkdownEditorView.create(document.getElementById('editor')!, '# Hola')
 * view.onChange((md) => console.log(md))
 * ```
 */
export class MilkdownEditorView implements IEditorView {
  readonly #editor: Editor
  readonly #listeners = new Set<(markdown: string) => void>()
  #frontMatter = ''
  /** Último Markdown conocido: evita notificar cambios que no hizo el usuario. */
  #last = ''

  private constructor(editor: Editor) {
    this.#editor = editor
  }

  /**
   * Crea el editor dentro de un elemento del DOM.
   *
   * @param root - Contenedor donde se monta el editor.
   * @param markdown - Documento inicial.
   * @returns La vista ya lista para usarse.
   */
  static async create(root: HTMLElement, markdown = ''): Promise<MilkdownEditorView> {
    const { frontMatter, body } = splitFrontMatter(markdown)
    // El callback se registra antes de que exista la vista: se rellena después.
    const ref: { view?: MilkdownEditorView } = {}
    const editor = await Editor.make()
      .config((ctx) => {
        ctx.set(rootCtx, root)
        ctx.set(defaultValueCtx, body)
        ctx.set(remarkStringifyOptionsCtx, { ...STRINGIFY_OPTIONS })
        ctx.get(listenerCtx).markdownUpdated((_ctx, md) => {
          if (ref.view) ref.view.#handleUpdate(md)
        })
      })
      .use(commonmark)
      .use(gfm)
      .use(history)
      .use(listener)
      .use(clipboard)
      .use(codeMetaPlugin)
      .create()
    const view = new MilkdownEditorView(editor)
    view.#frontMatter = frontMatter
    view.#last = view.getMarkdown()
    ref.view = view
    return view
  }

  /**
   * Acceso al editor Milkdown subyacente (para tests y usos avanzados).
   * El controlador **no** debe usarlo: trabaja solo con {@link IEditorView}.
   */
  get milkdown(): Editor {
    return this.#editor
  }

  setMarkdown(markdown: string): void {
    const { frontMatter, body } = splitFrontMatter(markdown)
    this.#frontMatter = frontMatter
    this.#editor.action(replaceAll(body, true))
    this.#last = this.getMarkdown()
  }

  getMarkdown(): string {
    return joinFrontMatter(this.#frontMatter, this.#editor.action(getMarkdown()))
  }

  onChange(listener: (markdown: string) => void): Unsubscribe {
    this.#listeners.add(listener)
    return () => this.#listeners.delete(listener)
  }

  focus(): void {
    this.#editor.action((ctx) => ctx.get(editorViewCtx).focus())
  }

  undo(): void {
    this.#editor.action(callCommand(undoCommand.key))
  }

  redo(): void {
    this.#editor.action(callCommand(redoCommand.key))
  }

  /** Libera el editor y sus recursos. */
  async destroy(): Promise<void> {
    this.#listeners.clear()
    await this.#editor.destroy()
  }

  #handleUpdate(body: string): void {
    const markdown = joinFrontMatter(this.#frontMatter, body)
    if (markdown === this.#last) return
    this.#last = markdown
    for (const listener of this.#listeners) listener(markdown)
  }
}
