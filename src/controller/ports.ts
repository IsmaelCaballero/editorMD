/**
 * **Puertos MVC**: los contratos entre los controladores y el exterior
 * (vistas, backend Rust y diálogos).
 *
 * @remarks
 * Los controladores dependen **solo** de estas interfaces, nunca de Svelte,
 * Milkdown o Tauri directamente. Así, en los tests se sustituyen por *mocks*
 * y el controlador se prueba sin ventana ni Rust (patrón *Passive View* +
 * inyección de dependencias, PLAN.md §4.1).
 *
 * @packageDocumentation
 */

/** Función que cancela una suscripción a eventos. */
export type Unsubscribe = () => void

/**
 * Vista del editor de documentos (la implementa V02 `WysiwygEditorView`).
 *
 * @remarks
 * El intercambio con el controlador es siempre **Markdown en texto**, que es la
 * fuente de verdad. El modelo interno del editor WYSIWYG no sale de la vista.
 */
export interface IEditorView {
  /**
   * Sustituye todo el contenido del editor.
   * @param markdown - Documento completo en Markdown.
   */
  setMarkdown(markdown: string): void
  /** @returns El documento actual del editor, serializado a Markdown. */
  getMarkdown(): string
  /**
   * Se suscribe a los cambios hechos por el usuario en el editor.
   * @param listener - Recibe el Markdown resultante tras cada cambio.
   * @returns Función para cancelar la suscripción.
   */
  onChange(listener: (markdown: string) => void): Unsubscribe
  /** Pone el foco del teclado en el editor. */
  focus(): void
}

/** Identidad de la aplicación devuelta por el backend (M00 `AppInfo`). */
export interface AppInfo {
  /** Nombre del producto, p. ej. `"editorMD"`. */
  readonly name: string
  /** Versión SemVer 2.0.0 de la compilación en ejecución. */
  readonly version: string
}

/**
 * Backend Rust, accesible mediante comandos Tauri (IPC).
 *
 * @remarks
 * Crecerá en F2 con la lectura/escritura de ficheros y las codificaciones.
 */
export interface IBackend {
  /** @returns Nombre y versión de la aplicación (comando `app_info`). */
  appInfo(): Promise<AppInfo>
}

/** Respuesta a un diálogo de cambios sin guardar. */
export type UnsavedChoice = 'save' | 'discard' | 'cancel'

/** Diálogos modales que el controlador puede pedir a la vista. */
export interface IDialogService {
  /**
   * Muestra un mensaje informativo.
   * @param title - Título del diálogo.
   * @param text - Texto del mensaje.
   */
  message(title: string, text: string): Promise<void>
  /**
   * Pregunta qué hacer con los cambios sin guardar de un documento.
   * @param documentName - Nombre del documento, p. ej. `"notas.md"`.
   * @returns La opción elegida por el usuario.
   */
  confirmUnsaved(documentName: string): Promise<UnsavedChoice>
}
