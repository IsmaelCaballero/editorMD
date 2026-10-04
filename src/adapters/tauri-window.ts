/**
 * **A02 `TauriWindow`** (Adaptador · TS): acceso a la ventana nativa.
 *
 * Cambiar el título de la ventana del sistema operativo requiere el permiso
 * `core:window:allow-set-title` (en `src-tauri/capabilities/default.json`).
 *
 * @packageDocumentation
 */
import { isTauri } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'

/**
 * Pone el título en la ventana nativa (Tauri) y en el documento HTML.
 *
 * @param title - Título completo.
 */
export async function setNativeTitle(title: string): Promise<void> {
  document.title = title
  if (isTauri()) await getCurrentWindow().setTitle(title)
}
