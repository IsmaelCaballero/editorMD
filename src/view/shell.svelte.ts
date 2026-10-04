/**
 * **V01 `MainWindow` · estado de la ventana** (Vista · TS + runes de Svelte).
 * Implementa el puerto {@link IWindowView}: el controlador escribe aquí el
 * título y los datos de la barra de estado, y los componentes los muestran de
 * forma reactiva.
 *
 * @packageDocumentation
 */
import type { IWindowView, StatusInfo } from '../controller/ports'

/** Estado inicial de la barra de estado. */
const INITIAL_STATUS: StatusInfo = {
  words: 0,
  encoding: 'utf-8',
  lineEnding: 'lf',
  modified: false,
}

/** Estado reactivo de la ventana principal. */
export class ShellState implements IWindowView {
  /** Título actual de la ventana. */
  title = $state('editorMD')
  /** Datos actuales de la barra de estado. */
  status = $state<StatusInfo>(INITIAL_STATUS)
  readonly #onTitle?: (title: string) => void

  /**
   * @param onTitle - Se llama con cada título nuevo (p. ej. para la ventana nativa).
   */
  constructor(onTitle?: (title: string) => void) {
    this.#onTitle = onTitle
  }

  setTitle(title: string): void {
    this.title = title
    this.#onTitle?.(title)
  }

  setStatus(status: StatusInfo): void {
    this.status = status
  }
}
