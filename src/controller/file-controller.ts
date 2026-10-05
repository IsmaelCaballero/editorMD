/**
 * **C02 `FileController`** (Controlador · TS): órdenes del menú Archivo.
 *
 * Nuevo, Abrir, Guardar, Guardar como y Cerrar, con el aviso de cambios sin
 * guardar. Lee y escribe a través del backend Rust (M08 `FileService`, comandos
 * `open_file`/`save_file`) y pide las rutas a los selectores nativos
 * ({@link IFilePicker}).
 *
 * Como el resto de controladores, solo conoce **puertos**: en los tests todas
 * sus dependencias son *mocks*.
 *
 * @packageDocumentation
 */
import { DocumentState, UNTITLED } from '../model/document'
import {
  isFileError,
  type FileError,
  type IBackend,
  type IDialogService,
  type IEditorView,
  type IFilePicker,
  type LossItem,
} from './ports'

/** Dependencias de {@link FileController}. */
export interface FileControllerDeps {
  /** Modelo del documento abierto. */
  readonly document: DocumentState
  /** Vista del editor. */
  readonly editor: IEditorView
  /** Backend Rust (lectura y escritura de ficheros). */
  readonly backend: IBackend
  /** Diálogos modales (errores y cambios sin guardar). */
  readonly dialogs: IDialogService
  /** Selectores nativos de ficheros. */
  readonly picker: IFilePicker
}

/** Máximo de caracteres no representables que se enumeran en un mensaje. */
const MAX_LOSS_ITEMS = 5

/**
 * Controlador de ficheros.
 *
 * @remarks
 * Todas las órdenes devuelven `true` si se completan y `false` si el usuario
 * cancela o hay un error (que ya se le ha mostrado). Mientras no haya pestañas
 * (F7), «Cerrar» y «Nuevo» hacen lo mismo: dejan un documento vacío sin título.
 *
 * @example
 * ```ts
 * const files = new FileController({ document, editor, backend, dialogs, picker })
 * await files.open()
 * await files.save()
 * ```
 */
export class FileController {
  readonly #deps: FileControllerDeps
  #readOnly = false

  /** @param deps - Puertos y modelo inyectados. */
  constructor(deps: FileControllerDeps) {
    this.#deps = deps
  }

  /** `true` si el fichero abierto es de solo lectura («Guardar» pasa a «Guardar como»). */
  get isReadOnly(): boolean {
    return this.#readOnly
  }

  /** «Nuevo»: tras confirmar los cambios sin guardar, deja un documento vacío. */
  newDocument(): Promise<boolean> {
    return this.#clear()
  }

  /** «Cerrar»: igual que «Nuevo» mientras la aplicación tenga un solo documento. */
  close(): Promise<boolean> {
    return this.#clear()
  }

  /**
   * «Abrir»: tras confirmar los cambios sin guardar, lee un fichero y lo muestra.
   * @param path - Ruta a abrir (p. ej. desde ficheros recientes); si se omite,
   *   se pide con el selector nativo.
   */
  async open(path?: string): Promise<boolean> {
    const { document, editor, backend, picker } = this.#deps
    if (!(await this.confirmDiscard())) return false
    const target = path ?? (await picker.pickOpenPath())
    if (target === null) return false
    try {
      const { file, readOnly } = await backend.openFile(target)
      document.replaceWith(
        DocumentState.fromFile(file.text, {
          path: target,
          encoding: file.encoding,
          lineEnding: file.lineEnding,
        }),
      )
      this.#readOnly = readOnly
    } catch (error) {
      await this.#showError('No se pudo abrir el fichero', error)
      return false
    }
    editor.setMarkdown(document.content)
    editor.focus()
    return true
  }

  /** «Guardar»: en su ruta, o con «Guardar como» si no tiene o es de solo lectura. */
  save(): Promise<boolean> {
    const { path } = this.#deps.document
    if (path === null || this.#readOnly) return this.saveAs()
    return this.#writeTo(path)
  }

  /** «Guardar como»: pide una ruta (propone la actual) y guarda en ella. */
  async saveAs(): Promise<boolean> {
    const { document, picker } = this.#deps
    const target = await picker.pickSavePath(document.path ?? `${UNTITLED}.md`)
    if (target === null) return false
    return this.#writeTo(target)
  }

  /**
   * Pregunta qué hacer con los cambios sin guardar (antes de Nuevo, Abrir,
   * Cerrar o cerrar la ventana).
   * @returns `true` si se puede continuar: no había cambios, se descartaron o
   *   se guardaron.
   */
  async confirmDiscard(): Promise<boolean> {
    const { document, dialogs } = this.#deps
    if (!document.isModified) return true
    const choice = await dialogs.confirmUnsaved(document.fileName)
    if (choice === 'cancel') return false
    if (choice === 'discard') return true
    return this.save()
  }

  async #clear(): Promise<boolean> {
    const { document, editor } = this.#deps
    if (!(await this.confirmDiscard())) return false
    document.reset()
    this.#readOnly = false
    editor.setMarkdown(document.content)
    editor.focus()
    return true
  }

  async #writeTo(path: string): Promise<boolean> {
    const { document, backend } = this.#deps
    const text = document.content
    try {
      await backend.saveFile({
        path,
        text,
        encoding: document.encoding,
        lineEnding: document.lineEnding,
      })
    } catch (error) {
      await this.#showError('No se pudo guardar el fichero', error)
      return false
    }
    document.markSaved(path, text)
    this.#readOnly = false
    return true
  }

  async #showError(title: string, error: unknown): Promise<void> {
    const text = isFileError(error)
      ? describeFileError(error)
      : `Error inesperado: ${error instanceof Error ? error.message : String(error)}`
    await this.#deps.dialogs.message(title, text)
  }
}

/**
 * Explica un error de ficheros en lenguaje llano para mostrarlo al usuario.
 * @param error - Error devuelto por el backend.
 */
export function describeFileError(error: FileError): string {
  switch (error.kind) {
    case 'not-found':
      return `No se encuentra «${error.path}».`
    case 'permission-denied':
      return `No tienes permiso para acceder a «${error.path}».`
    case 'read-only':
      return `«${error.path}» es de solo lectura. Usa «Guardar como…» para guardarlo con otro nombre.`
    case 'is-directory':
      return `«${error.path}» es una carpeta, no un fichero.`
    case 'io':
      return `Error de entrada/salida con «${error.path}»: ${error.message}`
    case 'decode':
      return `«${error.path}» no es válido en ${error.encoding}: ${error.message}`
    case 'unmappable': {
      const { encoding, items, total } = error.report
      const listed = items.slice(0, MAX_LOSS_ITEMS).map(describeLoss).join(', ')
      const more = items.length > MAX_LOSS_ITEMS ? ` y ${items.length - MAX_LOSS_ITEMS} más` : ''
      return (
        `${total} carácter(es) no caben en la codificación ${encoding}: ${listed}${more}.\n\n` +
        'No se ha guardado nada. Cambia la codificación del documento (por ejemplo, a UTF-8) ' +
        'o elimina esos caracteres.'
      )
    }
  }
}

function describeLoss({ ch, count, lines }: LossItem): string {
  return `«${ch}» (${count}, ${lines.length === 1 ? 'línea' : 'líneas'} ${lines.join(', ')})`
}
