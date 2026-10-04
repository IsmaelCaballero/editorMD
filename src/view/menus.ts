/**
 * **V01 `MainWindow` · modelo de menús** (Vista · TS).
 *
 * Los menús se describen como **datos** (qué opciones hay, su atajo y en qué
 * fase se implementan) y el componente `MenuBar.svelte` solo los dibuja. Así:
 *
 * - se prueban sin interfaz gráfica;
 * - la Vista no decide nada: al pulsar, emite el {@link CommandId} y es el
 *   Controlador quien actúa (*Passive View*);
 * - las opciones de fases futuras se ven deshabilitadas, con la fase en la
 *   que llegarán (útil para seguir el plan desde la propia aplicación).
 *
 * @packageDocumentation
 */

/** Fases del plan (PLAN.md §7). */
export type Phase = 'F1' | 'F2' | 'F3' | 'F4' | 'F5' | 'F6' | 'F7'

/** Fase actual del desarrollo: determina qué opciones están disponibles. */
export const CURRENT_PHASE: Phase = 'F1'

/** Identificadores de todas las órdenes que puede emitir la vista. */
export type CommandId =
  | 'file.new'
  | 'file.open'
  | 'file.save'
  | 'file.saveAs'
  | 'file.close'
  | 'file.import'
  | 'file.export'
  | 'file.encoding'
  | 'edit.undo'
  | 'edit.redo'
  | 'edit.cut'
  | 'edit.copy'
  | 'edit.paste'
  | 'edit.selectAll'
  | 'edit.find'
  | 'edit.replace'
  | 'format.h1'
  | 'format.h2'
  | 'format.h3'
  | 'format.bold'
  | 'format.italic'
  | 'format.strike'
  | 'format.code'
  | 'format.quote'
  | 'format.clear'
  | 'insert.table'
  | 'insert.bulletList'
  | 'insert.orderedList'
  | 'insert.taskList'
  | 'insert.link'
  | 'insert.image'
  | 'insert.codeBlock'
  | 'insert.rule'
  | 'view.sourceMode'
  | 'view.theme'
  | 'help.about'

/** Opción de menú. */
export interface MenuItem {
  /** Distingue opciones de separadores. */
  readonly kind: 'item'
  /** Orden que se emite al pulsarla. */
  readonly id: CommandId
  /** Texto visible. */
  readonly label: string
  /** Atajo en formato neutro: `Mod` = Ctrl (Windows/Linux) o ⌘ (macOS). */
  readonly shortcut?: string
  /** Fase del plan en la que se implementa. */
  readonly phase: Phase
}

/** Línea separadora dentro de un menú. */
export interface MenuSeparator {
  /** Distingue separadores de opciones. */
  readonly kind: 'separator'
}

/** Entrada de un menú: opción o separador. */
export type MenuEntry = MenuItem | MenuSeparator

/** Menú de la barra principal. */
export interface Menu {
  /** Identificador estable. */
  readonly id: string
  /** Texto visible. */
  readonly label: string
  /** Opciones y separadores, en orden. */
  readonly entries: readonly MenuEntry[]
}

const item = (id: CommandId, label: string, phase: Phase, shortcut?: string): MenuItem => ({
  kind: 'item',
  id,
  label,
  phase,
  ...(shortcut ? { shortcut } : {}),
})
const sep: MenuSeparator = { kind: 'separator' }

/** Barra de menús completa de editorMD (PLAN.md §2). */
export const MENUS: readonly Menu[] = [
  {
    id: 'file',
    label: 'Archivo',
    entries: [
      item('file.new', 'Nuevo', 'F1', 'Mod+N'),
      item('file.open', 'Abrir…', 'F2', 'Mod+O'),
      item('file.save', 'Guardar', 'F2', 'Mod+S'),
      item('file.saveAs', 'Guardar como…', 'F2', 'Mod+Shift+S'),
      item('file.encoding', 'Codificación y fin de línea…', 'F2'),
      sep,
      item('file.import', 'Importar…', 'F6'),
      item('file.export', 'Exportar…', 'F6', 'Mod+E'),
      sep,
      item('file.close', 'Cerrar', 'F2', 'Mod+W'),
    ],
  },
  {
    id: 'edit',
    label: 'Editar',
    entries: [
      item('edit.undo', 'Deshacer', 'F1', 'Mod+Z'),
      item('edit.redo', 'Rehacer', 'F1', 'Mod+Shift+Z'),
      sep,
      item('edit.cut', 'Cortar', 'F3', 'Mod+X'),
      item('edit.copy', 'Copiar', 'F3', 'Mod+C'),
      item('edit.paste', 'Pegar', 'F3', 'Mod+V'),
      item('edit.selectAll', 'Seleccionar todo', 'F3', 'Mod+A'),
      sep,
      item('edit.find', 'Buscar…', 'F3', 'Mod+F'),
      item('edit.replace', 'Reemplazar…', 'F3', 'Mod+H'),
    ],
  },
  {
    id: 'format',
    label: 'Formato',
    entries: [
      item('format.h1', 'Título 1', 'F4', 'Mod+1'),
      item('format.h2', 'Título 2', 'F4', 'Mod+2'),
      item('format.h3', 'Título 3', 'F4', 'Mod+3'),
      sep,
      item('format.bold', 'Negrita', 'F4', 'Mod+B'),
      item('format.italic', 'Cursiva', 'F4', 'Mod+I'),
      item('format.strike', 'Tachado', 'F4', 'Mod+Shift+X'),
      item('format.code', 'Código en línea', 'F4', 'Mod+`'),
      item('format.quote', 'Cita', 'F4', 'Mod+Shift+Q'),
      sep,
      item('format.clear', 'Quitar formato', 'F4', 'Mod+\\'),
    ],
  },
  {
    id: 'insert',
    label: 'Insertar',
    entries: [
      item('insert.table', 'Tabla…', 'F5'),
      item('insert.bulletList', 'Lista con viñetas', 'F5', 'Mod+Shift+8'),
      item('insert.orderedList', 'Lista numerada', 'F5', 'Mod+Shift+7'),
      item('insert.taskList', 'Lista de tareas', 'F5', 'Mod+Shift+9'),
      sep,
      item('insert.link', 'Enlace…', 'F5', 'Mod+K'),
      item('insert.image', 'Imagen…', 'F5'),
      item('insert.codeBlock', 'Bloque de código', 'F5', 'Mod+Alt+C'),
      item('insert.rule', 'Línea horizontal', 'F5'),
    ],
  },
  {
    id: 'view',
    label: 'Ver',
    entries: [
      item('view.sourceMode', 'Modo código fuente', 'F4', 'Mod+/'),
      item('view.theme', 'Tema claro/oscuro', 'F7'),
    ],
  },
  {
    id: 'help',
    label: 'Ayuda',
    entries: [item('help.about', 'Acerca de editorMD', 'F1')],
  },
]

const PHASE_ORDER: readonly Phase[] = ['F1', 'F2', 'F3', 'F4', 'F5', 'F6', 'F7']

/**
 * Indica si una opción ya está implementada en la fase dada.
 *
 * @param entry - Opción de menú.
 * @param current - Fase actual; por defecto {@link CURRENT_PHASE}.
 * @returns `true` si su fase es anterior o igual a la actual.
 */
export function isAvailable(entry: MenuItem, current: Phase = CURRENT_PHASE): boolean {
  return PHASE_ORDER.indexOf(entry.phase) <= PHASE_ORDER.indexOf(current)
}

/** Todas las opciones (sin separadores) de todos los menús. */
export function allItems(menus: readonly Menu[] = MENUS): MenuItem[] {
  return menus.flatMap((m) => m.entries.filter((e): e is MenuItem => e.kind === 'item'))
}

/** Plataforma, a efectos de cómo se muestran los atajos. */
export type Platform = 'mac' | 'other'

const MAC_SYMBOLS: Readonly<Record<string, string>> = {
  Ctrl: '⌃',
  Alt: '⌥',
  Shift: '⇧',
  Mod: '⌘',
}
/** Orden canónico de los modificadores en macOS (Human Interface Guidelines). */
const MAC_ORDER = ['Ctrl', 'Alt', 'Shift', 'Mod']

/**
 * Muestra un atajo según las convenciones de cada sistema operativo.
 *
 * @param shortcut - Atajo neutro, p. ej. `"Mod+Shift+S"`.
 * @param platform - `"mac"` o `"other"`.
 * @returns `"⇧⌘S"` en macOS; `"Ctrl+Shift+S"` en Windows y Linux.
 */
export function formatShortcut(shortcut: string, platform: Platform): string {
  const parts = shortcut.split('+')
  const key = parts.pop() as string
  if (platform === 'mac') {
    const mods = MAC_ORDER.filter((m) => parts.includes(m)).map((m) => MAC_SYMBOLS[m])
    return mods.join('') + key
  }
  return [...parts.map((m) => (m === 'Mod' ? 'Ctrl' : m)), key].join('+')
}

/**
 * Detecta la plataforma a partir de la cadena del navegador (WebView).
 *
 * @param userAgent - Normalmente `navigator.userAgent`.
 */
export function detectPlatform(userAgent: string): Platform {
  return /Mac|iPhone|iPad/.test(userAgent) ? 'mac' : 'other'
}

/** Teclas de un evento de teclado relevantes para los atajos. */
export type KeyInput = Pick<
  KeyboardEvent,
  'key' | 'code' | 'ctrlKey' | 'metaKey' | 'shiftKey' | 'altKey'
>

/**
 * Indica si un evento de teclado corresponde a un atajo.
 *
 * @remarks
 * `Mod` es Ctrl en Windows/Linux y ⌘ en macOS. La tecla se compara por
 * `key` y también por `code` (`KeyN`, `Digit8`), porque con Mayúsculas o con
 * otras distribuciones de teclado `key` cambia (`Shift+8` → `*` o `(`).
 *
 * @param event - Evento de teclado.
 * @param shortcut - Atajo neutro, p. ej. `"Mod+Shift+S"`.
 * @param platform - Plataforma.
 */
export function matchShortcut(event: KeyInput, shortcut: string, platform: Platform): boolean {
  const parts = shortcut.split('+')
  const key = (parts.pop() as string).toUpperCase()
  const mod = platform === 'mac' ? event.metaKey : event.ctrlKey
  const extraCtrl = platform === 'mac' ? event.ctrlKey : false
  if (mod !== parts.includes('Mod')) return false
  if (event.shiftKey !== parts.includes('Shift')) return false
  if (event.altKey !== parts.includes('Alt')) return false
  if (extraCtrl !== parts.includes('Ctrl')) return false
  if (platform !== 'mac' && event.metaKey) return false
  return (
    event.key.toUpperCase() === key || event.code === `Key${key}` || event.code === `Digit${key}`
  )
}

/**
 * Busca la orden disponible cuyo atajo coincide con un evento de teclado.
 *
 * @param event - Evento de teclado.
 * @param platform - Plataforma.
 * @param options - Fase actual y órdenes que no deben interceptarse (p. ej.
 *   deshacer/rehacer, que el editor ya gestiona con su propio teclado).
 * @returns La orden, o `null` si no hay ninguna.
 */
export function commandForKey(
  event: KeyInput,
  platform: Platform,
  options: { phase?: Phase; exclude?: readonly CommandId[] } = {},
): CommandId | null {
  const { phase = CURRENT_PHASE, exclude = [] } = options
  const hit = allItems().find(
    (i) =>
      i.shortcut !== undefined &&
      !exclude.includes(i.id) &&
      isAvailable(i, phase) &&
      matchShortcut(event, i.shortcut, platform),
  )
  return hit?.id ?? null
}
