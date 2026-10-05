// Tests de los adaptadores de Tauri (A01, A02, A03) con la API de Tauri simulada (vi.mock).
import { beforeEach, describe, expect, it, vi } from 'vitest'

const tauri = vi.hoisted(() => ({
  isTauri: vi.fn(() => false),
  invoke: vi.fn(),
  setTitle: vi.fn(async () => {}),
  onCloseRequested: vi.fn(),
  open: vi.fn(),
  save: vi.fn(),
}))

vi.mock('@tauri-apps/api/core', () => ({ isTauri: tauri.isTauri, invoke: tauri.invoke }))
vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => ({ setTitle: tauri.setTitle, onCloseRequested: tauri.onCloseRequested }),
}))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: tauri.open, save: tauri.save }))

const { TauriBackend } = await import('../../src/adapters/tauri-backend')
const { onCloseRequested, setNativeTitle } = await import('../../src/adapters/tauri-window')
const { FILE_FILTERS, TauriFilePicker } = await import('../../src/adapters/tauri-file-picker')

beforeEach(() => {
  vi.clearAllMocks()
  vi.stubGlobal('document', { title: '' })
})

describe('A01 TauriBackend', () => {
  it('dentro de Tauri invoca el comando Rust app_info', async () => {
    tauri.isTauri.mockReturnValue(true)
    tauri.invoke.mockResolvedValue({ name: 'editorMD', version: '1.2.3' })
    await expect(new TauriBackend().appInfo()).resolves.toEqual({
      name: 'editorMD',
      version: '1.2.3',
    })
    expect(tauri.invoke).toHaveBeenCalledWith('app_info')
  })

  it('fuera de Tauri (navegador) responde sin backend', async () => {
    tauri.isTauri.mockReturnValue(false)
    const info = await new TauriBackend().appInfo()
    expect(info.name).toBe('editorMD')
    expect(info.version).toContain('navegador')
    expect(tauri.invoke).not.toHaveBeenCalled()
  })
})

describe('A01 TauriBackend · ficheros', () => {
  it('openFile invoca open_file; sin codificación forzada envía null', async () => {
    tauri.isTauri.mockReturnValue(true)
    tauri.invoke.mockResolvedValue({ file: { text: 'x' }, readOnly: false })
    await new TauriBackend().openFile('/a.md')
    expect(tauri.invoke).toHaveBeenCalledWith('open_file', { path: '/a.md', encoding: null })
    await new TauriBackend().openFile('/a.md', 'windows-1252')
    expect(tauri.invoke).toHaveBeenLastCalledWith('open_file', {
      path: '/a.md',
      encoding: 'windows-1252',
    })
  })

  it('saveFile invoca save_file con los argumentos en camelCase', async () => {
    tauri.isTauri.mockReturnValue(true)
    tauri.invoke.mockResolvedValue({ bytesWritten: 1, losses: null })
    const backend = new TauriBackend()
    const request = { path: '/a.md', text: 'x', encoding: 'utf-8', lineEnding: 'crlf' } as const
    await backend.saveFile(request)
    expect(tauri.invoke).toHaveBeenCalledWith('save_file', { ...request, strategy: null })
    await backend.saveFile({ ...request, strategy: 'transliterate' })
    expect(tauri.invoke).toHaveBeenLastCalledWith('save_file', {
      ...request,
      strategy: 'transliterate',
    })
  })

  it('el FileError con el que Rust rechaza la promesa llega tal cual', async () => {
    tauri.isTauri.mockReturnValue(true)
    tauri.invoke.mockRejectedValue({ kind: 'not-found', path: '/a.md' })
    await expect(new TauriBackend().openFile('/a.md')).rejects.toEqual({
      kind: 'not-found',
      path: '/a.md',
    })
  })

  it('fuera de Tauri, abrir y guardar fallan con un error io', async () => {
    tauri.isTauri.mockReturnValue(false)
    const backend = new TauriBackend()
    await expect(backend.openFile('/a.md')).rejects.toMatchObject({ kind: 'io', path: '/a.md' })
    await expect(
      backend.saveFile({ path: '/b.md', text: '', encoding: 'utf-8', lineEnding: 'lf' }),
    ).rejects.toMatchObject({ kind: 'io', path: '/b.md' })
    expect(tauri.invoke).not.toHaveBeenCalled()
  })
})

describe('A02 setNativeTitle', () => {
  it('dentro de Tauri cambia el título de la ventana nativa y del documento', async () => {
    tauri.isTauri.mockReturnValue(true)
    await setNativeTitle('notas.md — editorMD')
    expect(tauri.setTitle).toHaveBeenCalledWith('notas.md — editorMD')
    expect(document.title).toBe('notas.md — editorMD')
  })

  it('fuera de Tauri solo cambia el título del documento', async () => {
    tauri.isTauri.mockReturnValue(false)
    await setNativeTitle('x')
    expect(tauri.setTitle).not.toHaveBeenCalled()
    expect(document.title).toBe('x')
  })
})

describe('A02 onCloseRequested', () => {
  /** Simula que el usuario pulsa X: llama al manejador registrado. */
  async function requestClose() {
    const handler = tauri.onCloseRequested.mock.calls[0]?.[0] as (event: {
      preventDefault: () => void
    }) => Promise<void>
    const event = { preventDefault: vi.fn() }
    await handler(event)
    return event
  }

  it('dentro de Tauri impide cerrar si canClose resuelve false', async () => {
    tauri.isTauri.mockReturnValue(true)
    const unlisten = vi.fn()
    tauri.onCloseRequested.mockResolvedValue(unlisten)
    const canClose = vi.fn(async () => false)
    expect(await onCloseRequested(canClose)).toBe(unlisten)
    const event = await requestClose()
    expect(canClose).toHaveBeenCalled()
    expect(event.preventDefault).toHaveBeenCalled()
  })

  it('dentro de Tauri deja cerrar si canClose resuelve true', async () => {
    tauri.isTauri.mockReturnValue(true)
    tauri.onCloseRequested.mockResolvedValue(vi.fn())
    await onCloseRequested(async () => true)
    const event = await requestClose()
    expect(event.preventDefault).not.toHaveBeenCalled()
  })

  it('fuera de Tauri no intercepta nada', async () => {
    tauri.isTauri.mockReturnValue(false)
    const unlisten = await onCloseRequested(async () => false)
    unlisten()
    expect(tauri.onCloseRequested).not.toHaveBeenCalled()
  })
})

describe('A03 TauriFilePicker', () => {
  it('pickOpenPath abre el selector nativo con los filtros de Markdown', async () => {
    tauri.isTauri.mockReturnValue(true)
    tauri.open.mockResolvedValue('/docs/a.md')
    await expect(new TauriFilePicker().pickOpenPath()).resolves.toBe('/docs/a.md')
    expect(tauri.open).toHaveBeenCalledWith({
      multiple: false,
      directory: false,
      filters: FILE_FILTERS,
    })
    expect(FILE_FILTERS[0]?.extensions).toContain('Rmd')
  })

  it('pickOpenPath devuelve null si el usuario cancela', async () => {
    tauri.isTauri.mockReturnValue(true)
    tauri.open.mockResolvedValue(null)
    await expect(new TauriFilePicker().pickOpenPath()).resolves.toBeNull()
  })

  it('pickSavePath propone la ruta sugerida y devuelve la elegida o null', async () => {
    tauri.isTauri.mockReturnValue(true)
    tauri.save.mockResolvedValueOnce('/docs/b.md').mockResolvedValueOnce(null)
    const picker = new TauriFilePicker()
    await expect(picker.pickSavePath('Sin título.md')).resolves.toBe('/docs/b.md')
    expect(tauri.save).toHaveBeenCalledWith({ defaultPath: 'Sin título.md', filters: FILE_FILTERS })
    await expect(picker.pickSavePath('x.md')).resolves.toBeNull()
  })

  it('fuera de Tauri no hay selectores: devuelve null', async () => {
    tauri.isTauri.mockReturnValue(false)
    const picker = new TauriFilePicker()
    await expect(picker.pickOpenPath()).resolves.toBeNull()
    await expect(picker.pickSavePath('x.md')).resolves.toBeNull()
    expect(tauri.open).not.toHaveBeenCalled()
    expect(tauri.save).not.toHaveBeenCalled()
  })
})
