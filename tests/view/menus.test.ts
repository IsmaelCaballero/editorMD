// Tests de V01 · modelo de menús (datos puros, sin interfaz).
import { describe, expect, it } from 'vitest'
import {
  allItems,
  type CommandId,
  commandForKey,
  CURRENT_PHASE,
  detectPlatform,
  formatShortcut,
  isAvailable,
  matchShortcut,
  MENUS,
} from '../../src/view/menus'

describe('estructura de menús', () => {
  it('tiene los 6 menús en el orden habitual', () => {
    expect(MENUS.map((m) => m.label)).toEqual([
      'Archivo',
      'Editar',
      'Formato',
      'Insertar',
      'Ver',
      'Ayuda',
    ])
  })

  it('los identificadores de orden son únicos', () => {
    const ids = allItems().map((i) => i.id)
    expect(new Set(ids).size).toBe(ids.length)
  })

  it('los atajos de teclado no se repiten', () => {
    const shortcuts = allItems()
      .map((i) => i.shortcut)
      .filter(Boolean)
    expect(new Set(shortcuts).size).toBe(shortcuts.length)
  })

  it('ningún menú empieza o termina con separador, ni tiene dos seguidos', () => {
    for (const menu of MENUS) {
      const kinds = menu.entries.map((e) => e.kind)
      expect(kinds[0]).toBe('item')
      expect(kinds.at(-1)).toBe('item')
      expect(kinds.join(',')).not.toContain('separator,separator')
    }
  })

  // Requisitos literales del usuario (CONTEXT.md, puntos 1-4).
  it.each<CommandId>([
    'file.open',
    'file.close',
    'file.save',
    'file.saveAs',
    'file.import',
    'file.export',
    'edit.selectAll',
    'edit.copy',
    'edit.paste',
    'edit.find',
    'edit.replace',
    'format.h1',
    'format.h2',
    'format.h3',
    'format.bold',
    'format.italic',
    'insert.table',
    'insert.bulletList',
    'insert.orderedList',
  ])('incluye la orden requerida %s', (id) => {
    expect(allItems().some((i) => i.id === id)).toBe(true)
  })
})

describe('disponibilidad por fase', () => {
  it('la fase actual es F1', () => {
    expect(CURRENT_PHASE).toBe('F1')
  })

  it('en F1 solo están disponibles Nuevo, Deshacer, Rehacer y Acerca de', () => {
    const available = allItems()
      .filter((i) => isAvailable(i))
      .map((i) => i.id)
    expect(available).toEqual(['file.new', 'edit.undo', 'edit.redo', 'help.about'])
  })

  it('una opción de F4 está disponible desde F4 en adelante', () => {
    const bold = allItems().find((i) => i.id === 'format.bold')!
    expect(isAvailable(bold, 'F3')).toBe(false)
    expect(isAvailable(bold, 'F4')).toBe(true)
    expect(isAvailable(bold, 'F7')).toBe(true)
  })
})

describe('formatShortcut', () => {
  it.each([
    ['Mod+S', 'other', 'Ctrl+S'],
    ['Mod+Shift+S', 'other', 'Ctrl+Shift+S'],
    ['Mod+Alt+C', 'other', 'Ctrl+Alt+C'],
    ['Mod+S', 'mac', '⌘S'],
    ['Mod+Shift+S', 'mac', '⇧⌘S'],
    ['Mod+Alt+C', 'mac', '⌥⌘C'],
    ['Shift+Mod+Z', 'mac', '⇧⌘Z'],
  ] as const)('%s en %s → %s', (shortcut, platform, expected) => {
    expect(formatShortcut(shortcut, platform)).toBe(expected)
  })
})

describe('detectPlatform', () => {
  it.each([
    ['Mozilla/5.0 (Macintosh; Intel Mac OS X 14_0) AppleWebKit/605.1.15', 'mac'],
    ['Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Edg/141.0', 'other'],
    ['Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/605.1.15', 'other'],
  ] as const)('%s → %s', (ua, expected) => {
    expect(detectPlatform(ua)).toBe(expected)
  })
})

describe('atajos de teclado (matchShortcut / commandForKey)', () => {
  const key = (
    k: string,
    mods: Partial<Record<'ctrl' | 'meta' | 'shift' | 'alt', boolean>> = {},
    code = '',
  ) => ({
    key: k,
    code,
    ctrlKey: !!mods.ctrl,
    metaKey: !!mods.meta,
    shiftKey: !!mods.shift,
    altKey: !!mods.alt,
  })

  it.each([
    [key('n', { ctrl: true }), 'Mod+N', 'other', true],
    [key('n', { meta: true }), 'Mod+N', 'mac', true],
    [key('n', { ctrl: true }), 'Mod+N', 'mac', false],
    [key('n', { meta: true }), 'Mod+N', 'other', false],
    [key('N', { ctrl: true, shift: true }), 'Mod+N', 'other', false],
    [key('S', { ctrl: true, shift: true }), 'Mod+Shift+S', 'other', true],
    [key('*', { ctrl: true, shift: true }, 'Digit8'), 'Mod+Shift+8', 'other', true],
    [key('c', { ctrl: true, alt: true }), 'Mod+Alt+C', 'other', true],
    [key('n'), 'Mod+N', 'other', false],
  ] as const)('%o con %s en %s → %s', (event, shortcut, platform, expected) => {
    expect(matchShortcut(event, shortcut, platform)).toBe(expected)
  })

  it('Ctrl+N en F1 corresponde a file.new', () => {
    expect(commandForKey(key('n', { ctrl: true }), 'other')).toBe('file.new')
  })

  it('no devuelve órdenes de fases futuras (Ctrl+B es Negrita, F4)', () => {
    expect(commandForKey(key('b', { ctrl: true }), 'other')).toBeNull()
    expect(commandForKey(key('b', { ctrl: true }), 'other', { phase: 'F4' })).toBe('format.bold')
  })

  it('respeta las exclusiones (deshacer lo gestiona el editor)', () => {
    const ctrlZ = key('z', { ctrl: true })
    expect(commandForKey(ctrlZ, 'other')).toBe('edit.undo')
    expect(commandForKey(ctrlZ, 'other', { exclude: ['edit.undo'] })).toBeNull()
  })
})
