/**
 * Textos de la barra de estado para codificaciones y finales de línea (Vista).
 * @packageDocumentation
 */
import type { Encoding, LineEnding } from '../model/document'

/** Nombre visible de cada codificación. */
export const ENCODING_LABELS: Readonly<Record<Encoding, string>> = {
  'utf-8': 'UTF-8',
  'utf-8-bom': 'UTF-8 con BOM',
  'utf-16le': 'UTF-16 LE',
  'utf-16be': 'UTF-16 BE',
  ascii: 'ASCII',
  'iso-8859-1': 'ISO-8859-1',
  'iso-8859-15': 'ISO-8859-15',
  'windows-1252': 'Windows-1252',
  macintosh: 'Mac Roman',
}

/** Nombre visible de cada fin de línea, con el sistema típico. */
export const LINE_ENDING_LABELS: Readonly<Record<LineEnding, string>> = {
  lf: 'LF',
  crlf: 'CRLF',
  cr: 'CR',
}
