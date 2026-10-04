/**
 * **M01 `DocumentState`** (Modelo · TS): estado del documento abierto.
 *
 * Guarda el Markdown, la ruta, la codificación y el fin de línea, y sabe si
 * hay cambios sin guardar. Notifica sus cambios a los suscriptores (patrón
 * *Observer*), de modo que los controladores actualizan las vistas sin que el
 * modelo conozca ninguna vista.
 *
 * @packageDocumentation
 */

/** Codificaciones soportadas (decisión D-15, PLAN.md §2.5). */
export type Encoding =
  | 'utf-8'
  | 'utf-8-bom'
  | 'utf-16le'
  | 'utf-16be'
  | 'ascii'
  | 'iso-8859-1'
  | 'iso-8859-15'
  | 'windows-1252'
  | 'macintosh'

/** Finales de línea: LF (Linux/macOS), CRLF (Windows), CR (Mac clásico). */
export type LineEnding = 'lf' | 'crlf' | 'cr'

/** Formato en disco de un documento. */
export interface FileFormat {
  /** Codificación de caracteres. */
  readonly encoding: Encoding
  /** Fin de línea. */
  readonly lineEnding: LineEnding
}

/** Opciones para {@link DocumentState.fromFile}. */
export interface FromFileOptions extends Partial<FileFormat> {
  /** Ruta del fichero en disco. */
  readonly path: string
}

/** Escucha los cambios de un {@link DocumentState}. */
export type DocumentListener = (doc: DocumentState) => void

/** Nombre que se muestra para un documento que aún no se ha guardado. */
export const UNTITLED = 'Sin título'

/** Formato por defecto de los documentos nuevos: UTF-8 sin BOM y LF. */
export const DEFAULT_FORMAT: FileFormat = { encoding: 'utf-8', lineEnding: 'lf' }

/**
 * Estado mutable y observable de un documento.
 *
 * @remarks
 * «Modificado» se calcula comparando el estado actual con el último guardado
 * (contenido, codificación y fin de línea), no con un indicador booleano. Así,
 * si el usuario deshace hasta volver al texto guardado, la marca `*` desaparece.
 *
 * @example
 * ```ts
 * const doc = DocumentState.fromFile('# Hola', { path: 'notas.md' })
 * doc.subscribe((d) => console.log(d.windowTitle('editorMD')))
 * doc.setContent('# Hola mundo') // → "notas.md * — editorMD"
 * ```
 */
export class DocumentState {
  #content: string
  #path: string | null
  #format: FileFormat
  /** Instantánea de lo último guardado (o cargado) en disco. */
  #saved: { content: string; format: FileFormat }
  readonly #listeners = new Set<DocumentListener>()

  private constructor(content: string, path: string | null, format: FileFormat) {
    this.#content = content
    this.#path = path
    this.#format = format
    this.#saved = { content, format }
  }

  /**
   * Crea un documento nuevo, vacío y sin ruta.
   * @param defaults - Formato inicial (preferencias); por defecto {@link DEFAULT_FORMAT}.
   */
  static createNew(defaults: Partial<FileFormat> = {}): DocumentState {
    return new DocumentState('', null, { ...DEFAULT_FORMAT, ...defaults })
  }

  /**
   * Crea un documento a partir de un fichero ya leído y decodificado.
   * @param content - Texto del fichero (ya decodificado, con `\n` como separador).
   * @param options - Ruta y formato detectado.
   */
  static fromFile(content: string, options: FromFileOptions): DocumentState {
    const { path, ...format } = options
    return new DocumentState(content, path, { ...DEFAULT_FORMAT, ...format })
  }

  /** Markdown actual. */
  get content(): string {
    return this.#content
  }

  /** Ruta en disco, o `null` si nunca se ha guardado. */
  get path(): string | null {
    return this.#path
  }

  /** Codificación con la que se guardará. */
  get encoding(): Encoding {
    return this.#format.encoding
  }

  /** Fin de línea con el que se guardará. */
  get lineEnding(): LineEnding {
    return this.#format.lineEnding
  }

  /** `true` si el documento aún no tiene ruta. */
  get isUntitled(): boolean {
    return this.#path === null
  }

  /** `true` si el estado actual difiere del último guardado. */
  get isModified(): boolean {
    return (
      this.#content !== this.#saved.content ||
      this.#format.encoding !== this.#saved.format.encoding ||
      this.#format.lineEnding !== this.#saved.format.lineEnding
    )
  }

  /**
   * Número de palabras (secuencias de letras o dígitos Unicode).
   * No cuenta la sintaxis Markdown ni las casillas de las listas de tareas.
   */
  get wordCount(): number {
    const text = this.#content.replace(/^\s*[-*+]\s+\[[ xX]\]/gm, '')
    return text.match(/[\p{L}\p{N}]+/gu)?.length ?? 0
  }

  /** Nombre del fichero (sin carpetas), o {@link UNTITLED}. Acepta rutas Unix y Windows. */
  get fileName(): string {
    if (this.#path === null) return UNTITLED
    return this.#path.split(/[\\/]/).pop() || this.#path
  }

  /**
   * Título de la ventana: `nombre[ *] — app`.
   * @param appName - Nombre de la aplicación.
   */
  windowTitle(appName: string): string {
    return `${this.fileName}${this.isModified ? ' *' : ''} — ${appName}`
  }

  /**
   * Sustituye el contenido (p. ej. tras una edición en la vista).
   * @param content - Nuevo Markdown. Si es igual al actual, no se notifica nada.
   */
  setContent(content: string): void {
    if (content === this.#content) return
    this.#content = content
    this.#emit()
  }

  /** Cambia la codificación con la que se guardará el documento. */
  setEncoding(encoding: Encoding): void {
    if (encoding === this.#format.encoding) return
    this.#format = { ...this.#format, encoding }
    this.#emit()
  }

  /** Cambia el fin de línea con el que se guardará el documento. */
  setLineEnding(lineEnding: LineEnding): void {
    if (lineEnding === this.#format.lineEnding) return
    this.#format = { ...this.#format, lineEnding }
    this.#emit()
  }

  /**
   * Registra que el estado actual se ha guardado en disco.
   * @param path - Nueva ruta («Guardar como»); si se omite, se mantiene la actual.
   */
  markSaved(path?: string): void {
    if (path !== undefined) this.#path = path
    this.#saved = { content: this.#content, format: this.#format }
    this.#emit()
  }

  /**
   * Carga en este mismo objeto el estado de otro documento (p. ej. al abrir un
   * fichero), conservando los suscriptores.
   * @param other - Documento cuyo estado se copia.
   */
  replaceWith(other: DocumentState): void {
    this.#content = other.#content
    this.#path = other.#path
    this.#format = other.#format
    this.#saved = other.#saved
    this.#emit()
  }

  /**
   * Vuelve a un documento nuevo y vacío («Nuevo»), conservando los suscriptores.
   * @param defaults - Formato inicial; por defecto {@link DEFAULT_FORMAT}.
   */
  reset(defaults: Partial<FileFormat> = {}): void {
    this.replaceWith(DocumentState.createNew(defaults))
  }

  /**
   * Se suscribe a los cambios del documento.
   * @param listener - Recibe el propio documento tras cada cambio.
   * @returns Función que cancela la suscripción.
   */
  subscribe(listener: DocumentListener): () => void {
    this.#listeners.add(listener)
    return () => this.#listeners.delete(listener)
  }

  #emit(): void {
    for (const listener of this.#listeners) listener(this)
  }
}
