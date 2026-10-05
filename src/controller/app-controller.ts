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
import { FileController } from './file-controller'
import type {
  IBackend,
  IDialogService,
  IEditorView,
  IFilePicker,
  IWindowView,
  Unsubscribe,
} from './ports'

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
  /** Selectores nativos de ficheros (para C02 `FileController`). */
  readonly picker: IFilePicker
}

/**
 * Controlador principal.
 *
 * @example
 * ```ts
 * const controller = new AppController({ document, editor, backend, dialogs, window, picker })
 * controller.start()
 * await controller.execute('file.new')
 * ```
 */
export class AppController {
  readonly #deps: AppControllerDeps
  readonly #files: FileController
  #subscriptions: Unsubscribe[] = []

  /**
   * @param deps - Puertos y modelo inyectados. Con ellos se crea también el
   *   controlador de ficheros (C02), en el que se delegan las órdenes `file.*`.
   */
  constructor(deps: AppControllerDeps) {
    this.#deps = deps
    this.#files = new FileController(deps)
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
        await this.#files.newDocument()
        return true
      case 'file.open':
        await this.#files.open()
        return true
      case 'file.save':
        await this.#files.save()
        return true
      case 'file.saveAs':
        await this.#files.saveAs()
        return true
      case 'file.close':
        await this.#files.close()
        return true
      case 'file.encoding':
        await this.#deps.dialogs.message(
          'Codificación y fin de línea',
          'Este diálogo llega en el paso 6/8 de F2. Mientras tanto, los ficheros se ' +
            'guardan con la codificación y el fin de línea con los que se abrieron.',
        )
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

  /**
   * Decide si se puede cerrar la ventana: si hay cambios sin guardar, pregunta.
   * @returns `true` si se puede cerrar.
   */
  confirmClose(): Promise<boolean> {
    return this.#files.confirmDiscard()
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
