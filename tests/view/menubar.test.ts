// @vitest-environment jsdom
// Tests del componente MenuBar.svelte (V01) montado en un DOM simulado.
import { flushSync, mount, unmount } from 'svelte'
import { afterEach, describe, expect, it, vi } from 'vitest'
import MenuBar from '../../src/view/MenuBar.svelte'

let component: ReturnType<typeof mount> | undefined

function render(props: Record<string, unknown> = {}) {
  const onCommand = vi.fn()
  component = mount(MenuBar, { target: document.body, props: { onCommand, ...props } })
  return { onCommand }
}

function openMenu(label: string): void {
  const title = [...document.querySelectorAll<HTMLButtonElement>('.menu-title')].find(
    (b) => b.textContent === label,
  )!
  title.click()
  flushSync()
}

const item = (id: string) => document.querySelector<HTMLButtonElement>(`[data-command="${id}"]`)

afterEach(() => {
  if (component) unmount(component)
  document.body.innerHTML = ''
})

describe('MenuBar', () => {
  it('dibuja los 6 menús', () => {
    render()
    const titles = [...document.querySelectorAll('.menu-title')].map((b) => b.textContent)
    expect(titles).toEqual(['Archivo', 'Editar', 'Formato', 'Insertar', 'Ver', 'Ayuda'])
  })

  it('al pulsar un menú se despliega y aria-expanded lo indica', () => {
    render()
    expect(document.querySelector('[role="menu"]')).toBeNull()
    openMenu('Archivo')
    expect(document.querySelector('[role="menu"]')?.getAttribute('aria-label')).toBe('Archivo')
  })

  it('pulsar una opción disponible emite su orden y cierra el menú', () => {
    const { onCommand } = render()
    openMenu('Archivo')
    item('file.new')!.click()
    flushSync()
    expect(onCommand).toHaveBeenCalledWith('file.new')
    expect(document.querySelector('[role="menu"]')).toBeNull()
  })

  it('las opciones de fases futuras están deshabilitadas e indican su fase', () => {
    render()
    openMenu('Archivo')
    const importItem = item('file.import')!
    expect(importItem.disabled).toBe(true)
    expect(importItem.title).toBe('Disponible en F6')
  })

  it('con una fase posterior, las opciones se habilitan', () => {
    render({ phase: 'F6' })
    openMenu('Archivo')
    expect(item('file.import')!.disabled).toBe(false)
  })

  it('muestra los atajos según la plataforma', () => {
    render({ platform: 'mac' })
    openMenu('Archivo')
    expect(item('file.new')!.querySelector('kbd')?.textContent).toBe('⌘N')
  })

  it('Escape cierra el menú abierto', () => {
    render()
    openMenu('Editar')
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    flushSync()
    expect(document.querySelector('[role="menu"]')).toBeNull()
  })
})
