/**
 * **V12 `DialogService`** (Vista · TS + runes de Svelte): diálogos modales
 * propios de la aplicación. Implementa el puerto {@link IDialogService}.
 *
 * @remarks
 * Se usan diálogos HTML en lugar de los nativos del sistema para que tengan el
 * mismo aspecto y los mismos botones en Windows, Linux y macOS, y para poder
 * probarlos en jsdom. El estado es **reactivo** (`$state`): `DialogHost.svelte`
 * dibuja la petición actual y, al pulsar un botón, se resuelve la promesa que
 * espera el controlador.
 *
 * Solo hay un diálogo a la vez; si llega otro, sustituye al anterior, que se
 * resuelve como «cancelar».
 *
 * @packageDocumentation
 */
import type { IDialogService, UnsavedChoice } from '../controller/ports'

/** Diálogo informativo con un solo botón. */
export interface MessageRequest {
  /** Tipo de diálogo. */
  readonly kind: 'message'
  /** Título. */
  readonly title: string
  /** Texto (los saltos de línea se respetan). */
  readonly text: string
  /** Cierra el diálogo. */
  readonly close: () => void
}

/** Diálogo de cambios sin guardar con tres botones. */
export interface UnsavedRequest {
  /** Tipo de diálogo. */
  readonly kind: 'unsaved'
  /** Nombre del documento afectado. */
  readonly documentName: string
  /** Responde y cierra el diálogo. */
  readonly answer: (choice: UnsavedChoice) => void
}

/** Petición de diálogo pendiente. */
export type DialogRequest = MessageRequest | UnsavedRequest

/** Servicio de diálogos reactivo. */
export class DialogService implements IDialogService {
  /** Diálogo visible, o `null` si no hay ninguno. */
  current = $state<DialogRequest | null>(null)

  message(title: string, text: string): Promise<void> {
    this.#dismissCurrent()
    return new Promise((resolve) => {
      this.current = {
        kind: 'message',
        title,
        text,
        close: () => {
          this.current = null
          resolve()
        },
      }
    })
  }

  confirmUnsaved(documentName: string): Promise<UnsavedChoice> {
    this.#dismissCurrent()
    return new Promise((resolve) => {
      this.current = {
        kind: 'unsaved',
        documentName,
        answer: (choice) => {
          this.current = null
          resolve(choice)
        },
      }
    })
  }

  /** Cierra el diálogo actual con su respuesta más prudente. */
  #dismissCurrent(): void {
    const c = this.current
    if (c?.kind === 'message') c.close()
    else if (c?.kind === 'unsaved') c.answer('cancel')
  }
}
