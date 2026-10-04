// @vitest-environment jsdom
// Tests de V02 WysiwygEditorView: el editor Milkdown real, montado en un DOM simulado.
import { readdirSync, readFileSync } from 'node:fs'
import { join } from 'node:path'
import { editorViewCtx } from '@milkdown/kit/core'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { equivalent } from '../../src/model/markdown-codec'
import { MilkdownEditorView } from '../../src/view/milkdown-editor'

const CORPUS_DIR = join(__dirname, '..', 'corpus')
const corpus = readdirSync(CORPUS_DIR)
  .filter((f) => /\.(md|Rmd)$/.test(f))
  .sort()
  .map((name) => [name, readFileSync(join(CORPUS_DIR, name), 'utf-8')] as const)

/**
 * Limitaciones conocidas de Milkdown 7: el documento se VE igual, pero el texto
 * cambia. Si una versión futura de Milkdown las corrige, estos tests avisarán.
 */
const KNOWN_LIMITATIONS: Record<string, (out: string) => void> = {
  '09-links-images.md': (out) => {
    // Los enlaces de referencia pasan a ser enlaces en línea...
    expect(out).toContain('[Referencia](https://example.com/ref)')
    expect(out).not.toContain('[ref]: https://example.com/ref')
    // ...y las URL sueltas se envuelven en < >.
    expect(out).toContain('<https://www.uclm.es>')
  },
}

const views: MilkdownEditorView[] = []
async function mount(markdown = ''): Promise<MilkdownEditorView> {
  const root = document.createElement('div')
  document.body.appendChild(root)
  const view = await MilkdownEditorView.create(root, markdown)
  views.push(view)
  return view
}

/** Simula que el usuario escribe texto al principio del documento. */
function typeText(view: MilkdownEditorView, text: string): void {
  view.milkdown.action((ctx) => {
    const pm = ctx.get(editorViewCtx)
    pm.dispatch(pm.state.tr.insertText(text, 1))
  })
}

afterEach(async () => {
  await Promise.all(views.splice(0).map((v) => v.destroy()))
  document.body.innerHTML = ''
})

describe('fidelidad del editor real sobre el corpus', () => {
  it.each(corpus)('%s', async (name, original) => {
    const out = (await mount(original)).getMarkdown()
    const limitation = KNOWN_LIMITATIONS[name]
    if (limitation) limitation(out)
    else expect(equivalent(original, out)).toBe(true)
  })

  it('el front matter sale byte a byte igual', async () => {
    const original = corpus.find(([n]) => n === '16-front-matter.md')![1]
    const fm = original.slice(0, original.indexOf('# Contenido'))
    expect((await mount(original)).getMarkdown().startsWith(fm)).toBe(true)
  })

  it('las cabeceras de los bloques R salen idénticas', async () => {
    const original = corpus.find(([n]) => n === '18-rmd-options.Rmd')![1]
    const out = (await mount(original)).getMarkdown()
    expect(out).toContain(
      '```{r grafico, echo=FALSE, fig.width=7, fig.height=4, fig.cap="Velocidad"}',
    )
    expect(out).toContain('```{python}')
  })
})

describe('IEditorView', () => {
  it('getMarkdown devuelve el documento inicial', async () => {
    expect((await mount('# Hola\n')).getMarkdown()).toBe('# Hola\n')
  })

  it('setMarkdown sustituye el contenido y conserva el front matter nuevo', async () => {
    const view = await mount('# Uno\n')
    view.setMarkdown('---\ntitle: x\n---\n\n# Dos\n')
    expect(view.getMarkdown()).toBe('---\ntitle: x\n---\n\n# Dos\n')
  })

  it('setMarkdown NO notifica onChange (no es una edición del usuario)', async () => {
    const view = await mount('# Uno\n')
    const listener = vi.fn()
    view.onChange(listener)
    view.setMarkdown('# Dos\n')
    await new Promise((r) => setTimeout(r, 400))
    expect(listener).not.toHaveBeenCalled()
  })

  it('una edición del usuario notifica el Markdown completo (con front matter)', async () => {
    const view = await mount('---\na: 1\n---\n\nTexto\n')
    const listener = vi.fn()
    view.onChange(listener)
    typeText(view, '¡Hola! ')
    await vi.waitFor(() => expect(listener).toHaveBeenCalled(), { timeout: 2000 })
    expect(listener).toHaveBeenLastCalledWith('---\na: 1\n---\n\n¡Hola! Texto\n')
  })

  it('la función devuelta por onChange cancela la suscripción', async () => {
    const view = await mount('Texto\n')
    const listener = vi.fn()
    view.onChange(listener)()
    typeText(view, 'x')
    await new Promise((r) => setTimeout(r, 400))
    expect(listener).not.toHaveBeenCalled()
  })

  it('undo deshace la última edición y redo la rehace', async () => {
    const view = await mount('Texto\n')
    typeText(view, 'Nuevo ')
    expect(view.getMarkdown()).toBe('Nuevo Texto\n')
    view.undo()
    expect(view.getMarkdown()).toBe('Texto\n')
    view.redo()
    expect(view.getMarkdown()).toBe('Nuevo Texto\n')
  })

  it('focus no lanza errores', async () => {
    const view = await mount('Texto\n')
    expect(() => view.focus()).not.toThrow()
  })
})
