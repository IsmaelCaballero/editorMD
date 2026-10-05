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
import type { Encoding, LineEnding } from '../model/document'

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
  /** Deshace la última edición del usuario (historial del editor). */
  undo(): void
  /** Rehace la última edición deshecha. */
  redo(): void
}

/** Identidad de la aplicación devuelta por el backend (M00 `AppInfo`). */
export interface AppInfo {
  /** Nombre del producto, p. ej. `"editorMD"`. */
  readonly name: string
  /** Versión SemVer 2.0.0 de la compilación en ejecución. */
  readonly version: string
}

/** Cómo se detectó la codificación de un fichero (M09 `DetectionMethod`). */
export type DetectionMethod = 'bom' | 'utf8-valid' | 'heuristic' | 'fallback'

/** Fichero de texto leído y decodificado (M09 `TextFile`). */
export interface TextFile {
  /** Contenido, siempre con `\n` como fin de línea y sin BOM. */
  readonly text: string
  /** Codificación con la que se leyó (y con la que se guardará). */
  readonly encoding: Encoding
  /** Fin de línea predominante (y con el que se guardará). */
  readonly lineEnding: LineEnding
  /** `true` si el fichero mezclaba varios finales de línea. */
  readonly mixedLineEndings: boolean
  /** Cómo se detectó la codificación; `null` si la eligió el usuario. */
  readonly detection: DetectionMethod | null
}

/** Resultado del comando `open_file` (M08 `OpenedFile`). */
export interface OpenedFile {
  /** Texto y formato del fichero. */
  readonly file: TextFile
  /** `true` si el fichero está marcado como de solo lectura. */
  readonly readOnly: boolean
}

/** Un carácter que no cabe en la codificación destino (M09 `LossItem`). */
export interface LossItem {
  /** El carácter. */
  readonly ch: string
  /** Número de apariciones. */
  readonly count: number
  /** Líneas (desde 1) en las que aparece. */
  readonly lines: readonly number[]
  /** Lo que se escribiría al transliterar. */
  readonly transliteration: string
}

/** Informe de pérdidas al codificar un texto (M09 `LossReport`). */
export interface LossReport {
  /** Codificación destino. */
  readonly encoding: Encoding
  /** Caracteres afectados, en el orden de su primera aparición. */
  readonly items: readonly LossItem[]
  /** Número total de apariciones afectadas. */
  readonly total: number
}

/** Qué hacer con los caracteres que no caben al guardar (M09 `LossStrategy`). */
export type LossStrategy = 'replace' | 'transliterate' | 'html-entities'

/** Resultado del comando `save_file` (M08 `SavedFile`). */
export interface SavedFile {
  /** Bytes escritos en disco. */
  readonly bytesWritten: number
  /** Pérdidas aplicadas con la estrategia pedida, o `null` si no hubo. */
  readonly losses: LossReport | null
}

/** Petición de guardado para {@link IBackend.saveFile}. */
export interface SaveRequest {
  /** Ruta destino. */
  readonly path: string
  /** Texto con `\n` como fin de línea. */
  readonly text: string
  /** Codificación de salida. */
  readonly encoding: Encoding
  /** Fin de línea de salida. */
  readonly lineEnding: LineEnding
  /** Estrategia de pérdidas; sin ella, si algo no cabe, el guardado falla con `unmappable`. */
  readonly strategy?: LossStrategy
}

/**
 * Error de un comando de ficheros (M08 `FileError`), con el discriminante `kind`.
 *
 * @remarks
 * Es el valor con el que se **rechaza** la promesa de {@link IBackend.openFile}
 * o {@link IBackend.saveFile}. Se reconoce con {@link isFileError}.
 */
export type FileError =
  | FileErrorBase<'not-found'>
  | FileErrorBase<'permission-denied'>
  | FileErrorBase<'read-only'>
  | FileErrorBase<'is-directory'>
  | (FileErrorBase<'io'> & {
      /** Mensaje del error de E/S. */
      readonly message: string
    })
  | (FileErrorBase<'decode'> & {
      /** Codificación con la que se intentó decodificar. */
      readonly encoding: Encoding
      /** Posición en bytes de la secuencia no válida, si se conoce. */
      readonly offset: number | null
      /** Mensaje del error de decodificación. */
      readonly message: string
    })
  | (FileErrorBase<'unmappable'> & {
      /** Caracteres que no caben en la codificación. */
      readonly report: LossReport
    })

/**
 * Campos comunes a todos los {@link FileError}.
 * @typeParam K - Tipo de error.
 */
export interface FileErrorBase<K extends string> {
  /** Tipo de error (discriminante). */
  readonly kind: K
  /** Ruta recibida por el comando. */
  readonly path: string
}

const FILE_ERROR_KINDS: ReadonlySet<string> = new Set([
  'not-found',
  'permission-denied',
  'read-only',
  'is-directory',
  'io',
  'decode',
  'unmappable',
])

/**
 * Indica si un valor rechazado por el backend es un {@link FileError}.
 * @param value - Lo capturado en un `catch`.
 */
export function isFileError(value: unknown): value is FileError {
  if (typeof value !== 'object' || value === null) return false
  const { kind, path } = value as { kind?: unknown; path?: unknown }
  return typeof kind === 'string' && FILE_ERROR_KINDS.has(kind) && typeof path === 'string'
}

/**
 * Backend Rust, accesible mediante comandos Tauri (IPC).
 *
 * @remarks
 * Las operaciones de ficheros rechazan su promesa con un {@link FileError}.
 */
export interface IBackend {
  /** @returns Nombre y versión de la aplicación (comando `app_info`). */
  appInfo(): Promise<AppInfo>
  /**
   * Lee un fichero de texto (comando `open_file`).
   * @param path - Ruta absoluta.
   * @param encoding - Codificación forzada («Reabrir con…»); si se omite, se detecta.
   */
  openFile(path: string, encoding?: Encoding): Promise<OpenedFile>
  /**
   * Guarda un texto de forma atómica (comando `save_file`).
   * @param request - Ruta, texto, formato y estrategia de pérdidas.
   */
  saveFile(request: SaveRequest): Promise<SavedFile>
}

/**
 * Selectores nativos de ficheros (lo implementa A03 `TauriFilePicker`).
 *
 * @remarks
 * Ambos métodos devuelven `null` si el usuario cancela.
 */
export interface IFilePicker {
  /** @returns Ruta del fichero elegido para abrir, o `null`. */
  pickOpenPath(): Promise<string | null>
  /**
   * @param suggestedPath - Ruta o nombre propuesto, p. ej. `"Sin título.md"`.
   * @returns Ruta elegida para guardar, o `null`.
   */
  pickSavePath(suggestedPath: string): Promise<string | null>
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

/** Datos que muestra la barra de estado. */
export interface StatusInfo {
  /** Número de palabras del documento. */
  readonly words: number
  /** Codificación con la que se guardará. */
  readonly encoding: Encoding
  /** Fin de línea con el que se guardará. */
  readonly lineEnding: LineEnding
  /** `true` si hay cambios sin guardar. */
  readonly modified: boolean
}

/** Ventana principal: título y barra de estado (lo implementa V01 `MainWindow`). */
export interface IWindowView {
  /**
   * Cambia el título de la ventana.
   * @param title - Texto completo, p. ej. `"notas.md * — editorMD"`.
   */
  setTitle(title: string): void
  /**
   * Actualiza la barra de estado.
   * @param status - Datos a mostrar.
   */
  setStatus(status: StatusInfo): void
}
