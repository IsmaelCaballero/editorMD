/**
 * **A03 `TauriFilePicker`** (Adaptador · TS): implementa el puerto
 * {@link IFilePicker} con los diálogos nativos del sistema operativo
 * (`tauri-plugin-dialog`).
 *
 * Permisos necesarios (en `src-tauri/capabilities/default.json`):
 * `dialog:allow-open` y `dialog:allow-save`. Fuera de Tauri no hay diálogos
 * nativos y los dos métodos devuelven `null` (como si el usuario cancelara).
 *
 * @packageDocumentation
 */
import { isTauri } from '@tauri-apps/api/core'
import { open, save, type DialogFilter } from '@tauri-apps/plugin-dialog'
import type { IFilePicker } from '../controller/ports'

/** Filtros de los selectores: Markdown (incluido R Markdown), texto y todos. */
export const FILE_FILTERS: DialogFilter[] = [
  { name: 'Markdown', extensions: ['md', 'markdown', 'Rmd', 'rmd'] },
  { name: 'Texto', extensions: ['txt'] },
  { name: 'Todos los ficheros', extensions: ['*'] },
]

/** Selectores nativos de abrir y guardar. */
export class TauriFilePicker implements IFilePicker {
  async pickOpenPath(): Promise<string | null> {
    if (!isTauri()) return null
    const chosen = await open({ multiple: false, directory: false, filters: FILE_FILTERS })
    return typeof chosen === 'string' ? chosen : null
  }

  async pickSavePath(suggestedPath: string): Promise<string | null> {
    if (!isTauri()) return null
    return (await save({ defaultPath: suggestedPath, filters: FILE_FILTERS })) ?? null
  }
}
