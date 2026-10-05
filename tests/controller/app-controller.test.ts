// Tests de C01 AppController con MOCKS de todos los puertos (TDD: antes de la implementación).
// El controlador se prueba sin ventana, sin Milkdown y sin Rust.
import { describe, expect, it, vi } from 'vitest'
import { AppController } from '../../src/controller/app-controller'
import type {
  IBackend,
  IDialogService,
  IEditorView,
  IFilePicker,
  IWindowView,
  UnsavedChoice,
} from '../../src/controller/ports'
import { DocumentState } from '../../src/model/document'

/** Editor falso: guarda el Markdown y permite simular ediciones del usuario. */
function fakeEditor() {
  let markdown = ''
  let listener: ((md: string) => void) | undefined
  const editor = {
    setMarkdown: vi.fn((md: string) => {
      markdown = md
    }),
    getMarkdown: vi.fn(() => markdown),
    onChange: vi.fn((l: (md: string) => void) => {
      listener = l
      return () => {
        listener = undefined
      }
    }),
    focus: vi.fn(),
    undo: vi.fn(),
    redo: vi.fn(),
  } satisfies IEditorView
  /** Simula que el usuario escribe. */
  const type = (md: string) => {
    markdown = md
    listener?.(md)
  }
  return { editor, type, hasListener: () => listener !== undefined }
}

function setup(options: { content?: string; choice?: UnsavedChoice } = {}) {
  const document = options.content
    ? DocumentState.fromFile(options.content, { path: '/docs/notas.md' })
    : DocumentState.createNew()
  const { editor, type, hasListener } = fakeEditor()
  const backend: IBackend = {
    appInfo: vi.fn(async () => ({ name: 'editorMD', version: '9.8.7' })),
    openFile: vi.fn(async () => ({
      file: {
        text: '# Abierto',
        encoding: 'utf-8' as const,
        lineEnding: 'lf' as const,
        mixedLineEndings: false,
        detection: 'bom' as const,
      },
      readOnly: false,
    })),
    saveFile: vi.fn(async () => ({ bytesWritten: 1, losses: null })),
  }
  const dialogs: IDialogService = {
    message: vi.fn(async () => {}),
    confirmUnsaved: vi.fn(async () => options.choice ?? 'cancel'),
  }
  const window: IWindowView = { setTitle: vi.fn(), setStatus: vi.fn() }
  const picker: IFilePicker = {
    pickOpenPath: vi.fn(async () => '/docs/otro.md'),
    pickSavePath: vi.fn(async () => '/docs/copia.md'),
  }
  const controller = new AppController({ document, editor, backend, dialogs, window, picker })
  return { controller, document, editor, type, hasListener, backend, dialogs, window, picker }
}

describe('AppController · arranque', () => {
  it('start muestra el documento en el editor y actualiza título y estado', () => {
    const { controller, editor, window } = setup({ content: '# Hola mundo' })
    controller.start()
    expect(editor.setMarkdown).toHaveBeenCalledWith('# Hola mundo')
    expect(window.setTitle).toHaveBeenLastCalledWith('notas.md — editorMD')
    expect(window.setStatus).toHaveBeenLastCalledWith({
      words: 2,
      encoding: 'utf-8',
      lineEnding: 'lf',
      modified: false,
    })
  })

  it('start pone el foco en el editor', () => {
    const { controller, editor } = setup()
    controller.start()
    expect(editor.focus).toHaveBeenCalled()
  })
})

describe('AppController · edición', () => {
  it('lo que escribe el usuario pasa al modelo y marca el documento como modificado', () => {
    const { controller, document, type, window } = setup({ content: 'Texto' })
    controller.start()
    type('Texto nuevo')
    expect(document.content).toBe('Texto nuevo')
    expect(window.setTitle).toHaveBeenLastCalledWith('notas.md * — editorMD')
    expect(window.setStatus).toHaveBeenLastCalledWith(
      expect.objectContaining({ words: 2, modified: true }),
    )
  })

  it('deshacer hasta el texto original quita la marca de modificado', () => {
    const { controller, type, window } = setup({ content: 'Texto' })
    controller.start()
    type('Otro')
    type('Texto')
    expect(window.setTitle).toHaveBeenLastCalledWith('notas.md — editorMD')
  })
})

describe('AppController · órdenes', () => {
  it('edit.undo y edit.redo se delegan en el editor', async () => {
    const { controller, editor } = setup()
    controller.start()
    expect(await controller.execute('edit.undo')).toBe(true)
    expect(await controller.execute('edit.redo')).toBe(true)
    expect(editor.undo).toHaveBeenCalledTimes(1)
    expect(editor.redo).toHaveBeenCalledTimes(1)
  })

  it('help.about muestra nombre y versión obtenidos del backend', async () => {
    const { controller, backend, dialogs } = setup()
    controller.start()
    await controller.execute('help.about')
    expect(backend.appInfo).toHaveBeenCalled()
    expect(dialogs.message).toHaveBeenCalledWith(
      'Acerca de editorMD',
      expect.stringContaining('editorMD 9.8.7'),
    )
  })

  it('una orden de una fase futura no hace nada y devuelve false', async () => {
    const { controller, editor, dialogs } = setup()
    controller.start()
    expect(await controller.execute('format.bold')).toBe(false)
    expect(editor.setMarkdown).toHaveBeenCalledTimes(1) // solo el de start
    expect(dialogs.message).not.toHaveBeenCalled()
  })
})

describe('AppController · Nuevo (file.new)', () => {
  it('sin cambios: vacía el documento sin preguntar', async () => {
    const { controller, document, editor, dialogs, window } = setup({ content: 'Algo' })
    controller.start()
    await controller.execute('file.new')
    expect(dialogs.confirmUnsaved).not.toHaveBeenCalled()
    expect(document.content).toBe('')
    expect(document.isUntitled).toBe(true)
    expect(editor.setMarkdown).toHaveBeenLastCalledWith('')
    expect(window.setTitle).toHaveBeenLastCalledWith('Sin título — editorMD')
  })

  it('con cambios y «descartar»: pregunta y vacía', async () => {
    const { controller, document, type, dialogs } = setup({ content: 'A', choice: 'discard' })
    controller.start()
    type('B')
    await controller.execute('file.new')
    expect(dialogs.confirmUnsaved).toHaveBeenCalledWith('notas.md')
    expect(document.content).toBe('')
  })

  it('con cambios y «cancelar»: no cambia nada', async () => {
    const { controller, document, type, editor } = setup({ content: 'A', choice: 'cancel' })
    controller.start()
    type('B')
    await controller.execute('file.new')
    expect(document.content).toBe('B')
    expect(editor.setMarkdown).toHaveBeenCalledTimes(1)
  })

  it('con cambios y «guardar»: guarda (C02) y después vacía', async () => {
    const { controller, document, type, backend } = setup({ content: 'A', choice: 'save' })
    controller.start()
    type('B')
    await controller.execute('file.new')
    expect(backend.saveFile).toHaveBeenCalledWith(expect.objectContaining({ text: 'B' }))
    expect(document.content).toBe('')
  })

  it('tras «Nuevo», el foco vuelve al editor', async () => {
    const { controller, editor } = setup()
    controller.start()
    editor.focus.mockClear()
    await controller.execute('file.new')
    expect(editor.focus).toHaveBeenCalled()
  })
})

describe('AppController · menú Archivo (delegado en C02)', () => {
  it('file.open abre el fichero elegido y actualiza el título', async () => {
    const { controller, document, editor, window } = setup()
    controller.start()
    expect(await controller.execute('file.open')).toBe(true)
    expect(document.path).toBe('/docs/otro.md')
    expect(editor.setMarkdown).toHaveBeenLastCalledWith('# Abierto')
    expect(window.setTitle).toHaveBeenLastCalledWith('otro.md — editorMD')
  })

  it('file.save guarda y quita la marca de modificado del título', async () => {
    const { controller, type, backend, window } = setup({ content: 'A' })
    controller.start()
    type('B')
    expect(await controller.execute('file.save')).toBe(true)
    expect(backend.saveFile).toHaveBeenCalledWith(
      expect.objectContaining({ path: '/docs/notas.md', text: 'B' }),
    )
    expect(window.setTitle).toHaveBeenLastCalledWith('notas.md — editorMD')
  })

  it('file.saveAs guarda en la ruta elegida', async () => {
    const { controller, document, picker } = setup({ content: 'A' })
    controller.start()
    expect(await controller.execute('file.saveAs')).toBe(true)
    expect(picker.pickSavePath).toHaveBeenCalledWith('/docs/notas.md')
    expect(document.path).toBe('/docs/copia.md')
  })

  it('file.close deja un documento vacío sin título', async () => {
    const { controller, document } = setup({ content: 'A' })
    controller.start()
    expect(await controller.execute('file.close')).toBe(true)
    expect(document.isUntitled).toBe(true)
    expect(document.content).toBe('')
  })

  it('file.encoding avisa de que el diálogo llega en el paso 6', async () => {
    const { controller, dialogs } = setup()
    controller.start()
    expect(await controller.execute('file.encoding')).toBe(true)
    expect(dialogs.message).toHaveBeenCalledWith(
      'Codificación y fin de línea',
      expect.stringContaining('paso 6/8'),
    )
  })

  it('confirmClose: sin cambios se puede cerrar; con cambios y «cancelar», no', async () => {
    const { controller, type, dialogs } = setup({ content: 'A', choice: 'cancel' })
    controller.start()
    expect(await controller.confirmClose()).toBe(true)
    type('B')
    expect(await controller.confirmClose()).toBe(false)
    expect(dialogs.confirmUnsaved).toHaveBeenCalledWith('notas.md')
  })
})

describe('AppController · dispose', () => {
  it('cancela las suscripciones al editor y al modelo', () => {
    const { controller, hasListener, document, window } = setup()
    controller.start()
    controller.dispose()
    expect(hasListener()).toBe(false)
    const calls = vi.mocked(window.setTitle).mock.calls.length
    document.setContent('cambio externo')
    expect(window.setTitle).toHaveBeenCalledTimes(calls)
  })
})
