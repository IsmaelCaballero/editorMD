// @vitest-environment jsdom
// Tests de V12 DialogService + DialogHost.svelte y del estado de la ventana (ShellState).
import { flushSync, mount, unmount } from 'svelte'
import { afterEach, describe, expect, it, vi } from 'vitest'
import DialogHost from '../../src/view/DialogHost.svelte'
import { DialogService } from '../../src/view/dialogs.svelte'
import { ShellState } from '../../src/view/shell.svelte'

let component: ReturnType<typeof mount> | undefined

function render() {
  const service = new DialogService()
  component = mount(DialogHost, { target: document.body, props: { service } })
  return service
}

const button = (text: string) =>
  [...document.querySelectorAll('button')].find((b) => b.textContent?.trim() === text)!

afterEach(() => {
  if (component) unmount(component)
  document.body.innerHTML = ''
})

describe('DialogService + DialogHost', () => {
  it('sin peticiones no se muestra nada', () => {
    render()
    expect(document.querySelector('[role="dialog"]')).toBeNull()
  })

  it('message muestra título y texto, y «Aceptar» resuelve la promesa', async () => {
    const service = render()
    const done = service.message('Acerca de', 'editorMD 0.0.1')
    flushSync()
    expect(document.querySelector('#dialog-title')?.textContent).toBe('Acerca de')
    expect(document.body.textContent).toContain('editorMD 0.0.1')
    button('Aceptar').click()
    await expect(done).resolves.toBeUndefined()
    flushSync()
    expect(document.querySelector('[role="dialog"]')).toBeNull()
  })

  it.each([
    ['Guardar', 'save'],
    ['Descartar cambios', 'discard'],
    ['Cancelar', 'cancel'],
  ])('confirmUnsaved: «%s» responde %s', async (label, choice) => {
    const service = render()
    const answer = service.confirmUnsaved('notas.md')
    flushSync()
    expect(document.body.textContent).toContain('«notas.md» tiene cambios sin guardar')
    button(label).click()
    await expect(answer).resolves.toBe(choice)
  })

  it('Escape cancela el diálogo de cambios sin guardar', async () => {
    const service = render()
    const answer = service.confirmUnsaved('a.md')
    flushSync()
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    await expect(answer).resolves.toBe('cancel')
  })

  it('Escape cierra un mensaje', async () => {
    const service = render()
    const done = service.message('t', 'x')
    flushSync()
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    await expect(done).resolves.toBeUndefined()
  })

  it('el botón principal recibe el foco', () => {
    const service = render()
    void service.confirmUnsaved('a.md')
    flushSync()
    expect(document.activeElement?.textContent?.trim()).toBe('Guardar')
  })

  it('un diálogo nuevo cancela el anterior de forma prudente', async () => {
    const service = render()
    const first = service.confirmUnsaved('a.md')
    const second = service.message('t', 'x')
    await expect(first).resolves.toBe('cancel')
    flushSync()
    button('Aceptar').click()
    await expect(second).resolves.toBeUndefined()
  })
})

describe('ShellState', () => {
  it('guarda título y estado y avisa del título nuevo', () => {
    const onTitle = vi.fn()
    const shell = new ShellState(onTitle)
    shell.setTitle('notas.md * — editorMD')
    shell.setStatus({ words: 3, encoding: 'ascii', lineEnding: 'crlf', modified: true })
    expect(shell.title).toBe('notas.md * — editorMD')
    expect(shell.status).toEqual({
      words: 3,
      encoding: 'ascii',
      lineEnding: 'crlf',
      modified: true,
    })
    expect(onTitle).toHaveBeenCalledWith('notas.md * — editorMD')
  })

  it('funciona sin callback', () => {
    expect(() => new ShellState().setTitle('x')).not.toThrow()
  })
})
