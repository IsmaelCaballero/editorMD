/**
 * **A02 `TauriWindow`** (Adaptador · TS): acceso a la ventana nativa.
 *
 * Permisos necesarios (en `src-tauri/capabilities/default.json`):
 * `core:window:allow-set-title` para el título y `core:window:allow-destroy`
 * para cerrar la ventana tras confirmar los cambios sin guardar.
 *
 * @packageDocumentation
 */
import { isTauri } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import type { Unsubscribe } from '../controller/ports'

/**
 * Pone el título en la ventana nativa (Tauri) y en el documento HTML.
 *
 * @param title - Título completo.
 */
export async function setNativeTitle(title: string): Promise<void> {
  document.title = title
  if (isTauri()) await getCurrentWindow().setTitle(title)
}

/**
 * Intercepta el cierre de la ventana nativa (botón X, Alt+F4, Cmd+Q…).
 *
 * @param canClose - Se consulta antes de cerrar; si resuelve `false`, la
 *   ventana sigue abierta.
 * @returns Función que retira la interceptación. Fuera de Tauri no hace nada.
 */
export async function onCloseRequested(canClose: () => Promise<boolean>): Promise<Unsubscribe> {
  if (!isTauri()) return () => {}
  return getCurrentWindow().onCloseRequested(async (event) => {
    if (!(await canClose())) event.preventDefault()
  })
}
