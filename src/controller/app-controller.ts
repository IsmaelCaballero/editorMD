/**
 * **C01 `AppController`** (Controlador · TS): orquesta la aplicación.
 *
 * Recibe las órdenes de la vista ({@link CommandId}), opera sobre el modelo
 * (M01 {@link DocumentState}) y actualiza las vistas a través de los
 * **puertos** (`IEditorView`, `IWindowView`, `IDialogService`, `IBackend`).
 *
 * No conoce Svelte, Milkdown ni Tauri: todas sus dependencias se le
 * **inyectan** en el constructor desde el *composition root* (`main.ts`), y en
 * los tests se sustituyen por *mocks*.
 *
 * @packageDocumentation
 */
import { DocumentState } from '../model/document'
import type { CommandId } from '../view/menus'
import type { IBackend, IDialogService, IEditorView, IWindowView, Unsubscribe } from './ports'

/** Nombre de la aplicación que aparece en el título de la ventana. */
export const APP_NAME = 'editorMD'

/** Dependencias de {@link AppController}. */
export interface AppControllerDeps {
  /** Modelo del documento abierto. */
  readonly document: DocumentState
  /** Vista del editor. */
  readonly editor: IEditorView
  /** Backend Rust (IPC). */
  readonly backend: IBackend
  /** Diálogos modales. */
  readonly dialogs: IDialogService
  /** Ventana principal (título y barra de estado). */
  readonly window: IWindowView
}

/**
 * Controlador principal.
 *
 * @example
 * ```ts
 * const controller = new AppController({ document, editor, backend, dialogs, window })
 * controller.start()
 * await controller.execute('file.new')
 * ```
 */
export class AppController {
  readonly #deps: AppControllerDeps
  #subscriptions: Unsubscribe[] = []

  /** @param deps - Puertos y modelo inyectados. */
  constructor(deps: AppControllerDeps) {
    this.#deps = deps
  }

  /**
   * Conecta modelo y vistas: muestra el documento, se suscribe a las ediciones
   * del usuario (vista → modelo) y a los cambios del modelo (modelo → ventana).
   */
  start(): void {
    const { document, editor } = this.#deps
    editor.setMarkdown(document.content)
    this.#subscriptions = [
      editor.onChange((markdown) => document.setContent(markdown)),
      document.subscribe(() => this.#render()),
    ]
    this.#render()
    editor.focus()
  }

  /**
   * Ejecuta una orden de menú o de teclado.
   *
   * @param command - Orden emitida por la vista.
   * @returns `true` si la orden está implementada en esta fase; `false` si no.
   */
  async execute(command: CommandId): Promise<boolean> {
    switch (command) {
      case 'file.new':
        await this.#newDocument()
        return true
      case 'edit.undo':
        this.#deps.editor.undo()
        return true
      case 'edit.redo':
        this.#deps.editor.redo()
        return true
      case 'help.about':
        await this.#about()
        return true
      default:
        return false
    }
  }

  /** Cancela todas las suscripciones (al cerrar la ventana o en los tests). */
  dispose(): void {
    for (const unsubscribe of this.#subscriptions) unsubscribe()
    this.#subscriptions = []
  }

  /** Lleva el estado del modelo al título y a la barra de estado. */
  #render(): void {
    const { document, window } = this.#deps
    window.setTitle(document.windowTitle(APP_NAME))
    window.setStatus({
      words: document.wordCount,
      encoding: document.encoding,
      lineEnding: document.lineEnding,
      modified: document.isModified,
    })
  }

  /**
   * «Nuevo»: si hay cambios sin guardar, pregunta antes de descartarlos.
   * En F1 todavía no existe «Guardar», así que esa respuesta solo avisa.
   */
  async #newDocument(): Promise<void> {
    const { document, editor, dialogs } = this.#deps
    if (document.isModified) {
      const choice = await dialogs.confirmUnsaved(document.fileName)
      if (choice === 'cancel') return
      if (choice === 'save') {
        await dialogs.message(
          'Guardar aún no está disponible',
          'Guardar llegará en la fase F2. El documento no se ha descartado.',
        )
        return
      }
    }
    document.reset()
    editor.setMarkdown(document.content)
    editor.focus()
  }

  /** «Acerca de»: nombre y versión desde el backend Rust (M00 `AppInfo`). */
  async #about(): Promise<void> {
    const { name, version } = await this.#deps.backend.appInfo()
    await this.#deps.dialogs.message(
      `Acerca de ${APP_NAME}`,
      `${name} ${version}\n\nEditor Markdown WYSIWYG ligero y multiplataforma.\n` +
        'Rust + Tauri 2 + Svelte 5 + Milkdown · Licencia MIT.',
    )
  }
}
