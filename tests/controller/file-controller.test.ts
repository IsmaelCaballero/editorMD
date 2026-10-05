// Tests de C02 FileController con MOCKS de todos los puertos (TDD: antes de la implementación).
// Se prueba sin ventana, sin Milkdown, sin Rust y sin selectores nativos.
import { describe, expect, it, vi } from 'vitest'
import { FileController, describeFileError } from '../../src/controller/file-controller'
import {
  isFileError,
  type FileError,
  type IBackend,
  type IDialogService,
  type IEditorView,
  type IFilePicker,
  type OpenedFile,
  type UnsavedChoice,
} from '../../src/controller/ports'
import { DocumentState } from '../../src/model/document'

function fakeEditor() {
  let markdown = ''
  const editor = {
    setMarkdown: vi.fn((md: string) => {
      markdown = md
    }),
    getMarkdown: vi.fn(() => markdown),
    onChange: vi.fn(() => () => {}),
    focus: vi.fn(),
    undo: vi.fn(),
    redo: vi.fn(),
  } satisfies IEditorView
  return editor
}

function opened(text: string, extra: Partial<OpenedFile['file']> = {}, readOnly = false) {
  return {
    file: {
      text,
      encoding: 'utf-8',
      lineEnding: 'lf',
      mixedLineEndings: false,
      detection: 'utf8-valid',
      ...extra,
    },
    readOnly,
  } satisfies OpenedFile
}

interface Options {
  /** Documento inicial con ruta `/docs/notas.md` (si no, uno nuevo y vacío). */
  content?: string
  /** Respuestas sucesivas al diálogo de cambios sin guardar. */
  choices?: UnsavedChoice[]
  openPath?: string | null
  savePath?: string | null
}

function setup(options: Options = {}) {
  const document =
    options.content === undefined
      ? DocumentState.createNew()
      : DocumentState.fromFile(options.content, { path: '/docs/notas.md' })
  const editor = fakeEditor()
  const choices = [...(options.choices ?? [])]
  const backend = {
    appInfo: vi.fn(async () => ({ name: 'editorMD', version: '0.0.0' })),
    openFile: vi.fn(async (_path: string) => opened('# Abierto')),
    saveFile: vi.fn(async () => ({ bytesWritten: 3, losses: null })),
  } satisfies IBackend
  const dialogs = {
    message: vi.fn(async () => {}),
    confirmUnsaved: vi.fn(async () => choices.shift() ?? 'cancel'),
  } satisfies IDialogService
  const picker = {
    pickOpenPath: vi.fn(async () =>
      options.openPath === undefined ? '/docs/otro.md' : options.openPath,
    ),
    pickSavePath: vi.fn(async () =>
      options.savePath === undefined ? '/docs/nuevo.md' : options.savePath,
    ),
  } satisfies IFilePicker
  const files = new FileController({ document, editor, backend, dialogs, picker })
  return { files, document, editor, backend, dialogs, picker }
}

const unmappable: FileError = {
  kind: 'unmappable',
  path: '/docs/notas.md',
  report: {
    encoding: 'iso-8859-1',
    items: [
      { ch: '€', count: 2, lines: [1, 3], transliteration: 'EUR' },
      { ch: '—', count: 1, lines: [2], transliteration: '--' },
    ],
    total: 3,
  },
}

describe('FileController · Abrir', () => {
  it('abre el fichero elegido, lo muestra en el editor y conserva su formato', async () => {
    const { files, document, editor, backend } = setup({ content: 'Viejo' })
    backend.openFile.mockResolvedValueOnce(
      opened('Hola\nmundo', { encoding: 'windows-1252', lineEnding: 'crlf' }),
    )
    expect(await files.open()).toBe(true)
    expect(backend.openFile).toHaveBeenCalledWith('/docs/otro.md')
    expect(document.content).toBe('Hola\nmundo')
    expect(document.path).toBe('/docs/otro.md')
    expect(document.encoding).toBe('windows-1252')
    expect(document.lineEnding).toBe('crlf')
    expect(document.isModified).toBe(false)
    expect(editor.setMarkdown).toHaveBeenLastCalledWith('Hola\nmundo')
    expect(editor.focus).toHaveBeenCalled()
  })

  it('con una ruta dada (p. ej. ficheros recientes) no muestra el selector', async () => {
    const { files, picker, backend } = setup()
    expect(await files.open('/docs/reciente.md')).toBe(true)
    expect(picker.pickOpenPath).not.toHaveBeenCalled()
    expect(backend.openFile).toHaveBeenCalledWith('/docs/reciente.md')
  })

  it('si el usuario cancela el selector, no cambia nada', async () => {
    const { files, document, backend } = setup({ content: 'Viejo', openPath: null })
    expect(await files.open()).toBe(false)
    expect(backend.openFile).not.toHaveBeenCalled()
    expect(document.content).toBe('Viejo')
  })

  it('si el backend falla, muestra el error y conserva el documento', async () => {
    const { files, document, backend, dialogs } = setup({ content: 'Viejo' })
    backend.openFile.mockRejectedValueOnce({ kind: 'not-found', path: '/docs/otro.md' })
    expect(await files.open()).toBe(false)
    expect(dialogs.message).toHaveBeenCalledWith(
      'No se pudo abrir el fichero',
      expect.stringContaining('No se encuentra «/docs/otro.md»'),
    )
    expect(document.content).toBe('Viejo')
    expect(document.path).toBe('/docs/notas.md')
  })

  it('un error que no es de ficheros también se muestra', async () => {
    const { files, backend, dialogs } = setup()
    backend.openFile.mockRejectedValueOnce(new Error('IPC roto'))
    expect(await files.open()).toBe(false)
    expect(dialogs.message).toHaveBeenCalledWith(
      'No se pudo abrir el fichero',
      expect.stringContaining('IPC roto'),
    )
  })

  it('un rechazo que ni siquiera es un Error se muestra como texto', async () => {
    const { files, backend, dialogs } = setup()
    backend.openFile.mockRejectedValueOnce('fallo raro')
    await files.open()
    expect(dialogs.message).toHaveBeenCalledWith(
      'No se pudo abrir el fichero',
      'Error inesperado: fallo raro',
    )
  })

  it('con cambios sin guardar pregunta antes; «cancelar» no abre nada', async () => {
    const { files, document, picker } = setup({ content: 'A', choices: ['cancel'] })
    document.setContent('B')
    expect(await files.open()).toBe(false)
    expect(picker.pickOpenPath).not.toHaveBeenCalled()
    expect(document.content).toBe('B')
  })

  it('con cambios y «descartar» abre el fichero', async () => {
    const { files, document } = setup({ content: 'A', choices: ['discard'] })
    document.setContent('B')
    expect(await files.open()).toBe(true)
    expect(document.content).toBe('# Abierto')
  })

  it('un fichero de solo lectura se marca como tal', async () => {
    const { files, backend } = setup()
    backend.openFile.mockResolvedValueOnce(opened('x', {}, true))
    await files.open()
    expect(files.isReadOnly).toBe(true)
  })
})

describe('FileController · Guardar', () => {
  it('guarda con la ruta, la codificación y el fin de línea del documento', async () => {
    const { files, document, backend, picker } = setup({ content: 'A' })
    document.setEncoding('utf-16le')
    document.setLineEnding('crlf')
    document.setContent('B')
    expect(await files.save()).toBe(true)
    expect(picker.pickSavePath).not.toHaveBeenCalled()
    expect(backend.saveFile).toHaveBeenCalledWith({
      path: '/docs/notas.md',
      text: 'B',
      encoding: 'utf-16le',
      lineEnding: 'crlf',
    })
    expect(document.isModified).toBe(false)
  })

  it('un documento sin título pasa a «Guardar como»', async () => {
    const { files, document, picker, backend } = setup()
    document.setContent('Nuevo')
    expect(await files.save()).toBe(true)
    expect(picker.pickSavePath).toHaveBeenCalledWith('Sin título.md')
    expect(backend.saveFile).toHaveBeenCalledWith(
      expect.objectContaining({ path: '/docs/nuevo.md', text: 'Nuevo' }),
    )
    expect(document.path).toBe('/docs/nuevo.md')
    expect(document.isModified).toBe(false)
  })

  it('un fichero de solo lectura pasa a «Guardar como» y deja de serlo', async () => {
    const { files, backend, picker, document } = setup()
    backend.openFile.mockResolvedValueOnce(opened('x', {}, true))
    await files.open('/docs/ro.md')
    expect(await files.save()).toBe(true)
    expect(picker.pickSavePath).toHaveBeenCalledWith('/docs/ro.md')
    expect(document.path).toBe('/docs/nuevo.md')
    expect(files.isReadOnly).toBe(false)
  })

  it('si el backend falla, muestra el error y el documento sigue modificado', async () => {
    const { files, document, backend, dialogs } = setup({ content: 'A' })
    document.setContent('B')
    backend.saveFile.mockRejectedValueOnce({ kind: 'permission-denied', path: '/docs/notas.md' })
    expect(await files.save()).toBe(false)
    expect(dialogs.message).toHaveBeenCalledWith(
      'No se pudo guardar el fichero',
      expect.stringContaining('permiso'),
    )
    expect(document.isModified).toBe(true)
  })

  it('caracteres que no caben: informa de cuáles y no guarda', async () => {
    const { files, document, backend, dialogs } = setup({ content: 'A' })
    document.setContent('B')
    backend.saveFile.mockRejectedValueOnce(unmappable)
    expect(await files.save()).toBe(false)
    const text = vi.mocked(dialogs.message).mock.calls[0]?.[1] ?? ''
    expect(text).toContain('iso-8859-1')
    expect(text).toContain('«€» (2, líneas 1, 3)')
    expect(text).toContain('«—» (1, línea 2)')
    expect(document.isModified).toBe(true)
  })

  it('lo que se escribe mientras se guarda sigue contando como cambio sin guardar', async () => {
    const { files, document, backend } = setup({ content: 'A' })
    document.setContent('B')
    backend.saveFile.mockImplementationOnce(async () => {
      document.setContent('BC') // el usuario sigue escribiendo
      return { bytesWritten: 1, losses: null }
    })
    expect(await files.save()).toBe(true)
    expect(document.content).toBe('BC')
    expect(document.isModified).toBe(true)
  })
})

describe('FileController · Guardar como', () => {
  it('propone la ruta actual y guarda en la elegida', async () => {
    const { files, document, picker, backend } = setup({ content: 'A' })
    expect(await files.saveAs()).toBe(true)
    expect(picker.pickSavePath).toHaveBeenCalledWith('/docs/notas.md')
    expect(backend.saveFile).toHaveBeenCalledWith(
      expect.objectContaining({ path: '/docs/nuevo.md', text: 'A' }),
    )
    expect(document.path).toBe('/docs/nuevo.md')
  })

  it('si el usuario cancela el selector, no guarda', async () => {
    const { files, document, backend } = setup({ content: 'A', savePath: null })
    document.setContent('B')
    expect(await files.saveAs()).toBe(false)
    expect(backend.saveFile).not.toHaveBeenCalled()
    expect(document.path).toBe('/docs/notas.md')
    expect(document.isModified).toBe(true)
  })

  it('si falla, conserva la ruta anterior', async () => {
    const { files, document, backend } = setup({ content: 'A' })
    backend.saveFile.mockRejectedValueOnce({ kind: 'is-directory', path: '/docs/nuevo.md' })
    expect(await files.saveAs()).toBe(false)
    expect(document.path).toBe('/docs/notas.md')
  })
})

describe('FileController · Nuevo y Cerrar', () => {
  for (const action of ['newDocument', 'close'] as const) {
    it(`${action} sin cambios: deja un documento vacío sin título`, async () => {
      const { files, document, editor, dialogs } = setup({ content: 'Algo' })
      expect(await files[action]()).toBe(true)
      expect(dialogs.confirmUnsaved).not.toHaveBeenCalled()
      expect(document.content).toBe('')
      expect(document.isUntitled).toBe(true)
      expect(editor.setMarkdown).toHaveBeenLastCalledWith('')
      expect(editor.focus).toHaveBeenCalled()
    })

    it(`${action} con cambios y «cancelar»: no cambia nada`, async () => {
      const { files, document, editor } = setup({ content: 'A', choices: ['cancel'] })
      document.setContent('B')
      expect(await files[action]()).toBe(false)
      expect(document.content).toBe('B')
      expect(editor.setMarkdown).not.toHaveBeenCalled()
    })
  }

  it('«Nuevo» con cambios y «guardar»: guarda y después vacía', async () => {
    const { files, document, backend, dialogs } = setup({ content: 'A', choices: ['save'] })
    document.setContent('B')
    expect(await files.newDocument()).toBe(true)
    expect(dialogs.confirmUnsaved).toHaveBeenCalledWith('notas.md')
    expect(backend.saveFile).toHaveBeenCalledWith(expect.objectContaining({ text: 'B' }))
    expect(document.content).toBe('')
  })

  it('«guardar» un sin título y cancelar «Guardar como» no descarta nada', async () => {
    const { files, document } = setup({ choices: ['save'], savePath: null })
    document.setContent('Borrador')
    expect(await files.close()).toBe(false)
    expect(document.content).toBe('Borrador')
  })

  it('«guardar» que falla no descarta nada', async () => {
    const { files, document, backend } = setup({ content: 'A', choices: ['save'] })
    document.setContent('B')
    backend.saveFile.mockRejectedValueOnce(unmappable)
    expect(await files.close()).toBe(false)
    expect(document.content).toBe('B')
  })

  it('tras cerrar un fichero de solo lectura, el nuevo documento no lo es', async () => {
    const { files, backend } = setup()
    backend.openFile.mockResolvedValueOnce(opened('x', {}, true))
    await files.open('/docs/ro.md')
    await files.close()
    expect(files.isReadOnly).toBe(false)
  })
})

describe('FileController · confirmDiscard (cerrar la ventana)', () => {
  it('sin cambios permite cerrar sin preguntar', async () => {
    const { files, dialogs } = setup({ content: 'A' })
    expect(await files.confirmDiscard()).toBe(true)
    expect(dialogs.confirmUnsaved).not.toHaveBeenCalled()
  })

  it.each([
    ['discard', true],
    ['cancel', false],
    ['save', true],
  ] as const)('con cambios y «%s» → %s', async (choice, expected) => {
    const { files, document } = setup({ content: 'A', choices: [choice] })
    document.setContent('B')
    expect(await files.confirmDiscard()).toBe(expected)
  })
})

describe('describeFileError', () => {
  it.each<[FileError, string]>([
    [{ kind: 'not-found', path: 'a.md' }, 'No se encuentra «a.md».'],
    [{ kind: 'permission-denied', path: 'a.md' }, 'No tienes permiso para acceder a «a.md».'],
    [{ kind: 'read-only', path: 'a.md' }, '«a.md» es de solo lectura'],
    [{ kind: 'is-directory', path: 'a' }, '«a» es una carpeta, no un fichero.'],
    [{ kind: 'io', path: 'a.md', message: 'disco lleno' }, 'disco lleno'],
    [
      { kind: 'decode', path: 'a.md', encoding: 'utf-16le', offset: null, message: 'impar' },
      '«a.md» no es válido en utf-16le: impar',
    ],
  ])('%o', (error, expected) => {
    expect(describeFileError(error)).toContain(expected)
  })

  it('resume como mucho 5 caracteres no representables', () => {
    const items = [...'abcdefg'].map((ch) => ({ ch, count: 1, lines: [1], transliteration: ch }))
    const text = describeFileError({
      kind: 'unmappable',
      path: 'a.md',
      report: { encoding: 'ascii', items, total: 7 },
    })
    expect(text).toContain('«e»')
    expect(text).not.toContain('«f»')
    expect(text).toContain('y 2 más')
  })
})

describe('isFileError', () => {
  it.each<[unknown, boolean]>([
    [{ kind: 'not-found', path: 'a' }, true],
    [{ kind: 'unmappable', path: 'a', report: {} }, true],
    [{ kind: 'otro', path: 'a' }, false],
    [{ kind: 'io' }, false],
    [{ path: 'a' }, false],
    [new Error('x'), false],
    ['not-found', false],
    [null, false],
    [undefined, false],
  ])('%o → %s', (value, expected) => {
    expect(isFileError(value)).toBe(expected)
  })
})
