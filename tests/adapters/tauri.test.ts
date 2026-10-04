// Tests de los adaptadores de Tauri (A01, A02) con la API de Tauri simulada (vi.mock).
import { beforeEach, describe, expect, it, vi } from 'vitest'

const tauri = vi.hoisted(() => ({
  isTauri: vi.fn(() => false),
  invoke: vi.fn(),
  setTitle: vi.fn(async () => {}),
}))

vi.mock('@tauri-apps/api/core', () => ({ isTauri: tauri.isTauri, invoke: tauri.invoke }))
vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => ({ setTitle: tauri.setTitle }),
}))

const { TauriBackend } = await import('../../src/adapters/tauri-backend')
const { setNativeTitle } = await import('../../src/adapters/tauri-window')

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
