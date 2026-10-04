/**
 * **A01 `TauriBackend`** (Adaptador · TS): implementa el puerto {@link IBackend}
 * llamando a los comandos Rust de Tauri por IPC.
 *
 * Fuera de Tauri (p. ej. `npm run dev` en un navegador) no hay backend Rust:
 * responde con valores del propio frontend para que la interfaz siga siendo
 * usable durante el desarrollo.
 *
 * @packageDocumentation
 */
import { invoke, isTauri } from '@tauri-apps/api/core'
import type { AppInfo, IBackend } from '../controller/ports'

/** Adaptador del backend Rust (IPC de Tauri). */
export class TauriBackend implements IBackend {
  async appInfo(): Promise<AppInfo> {
    if (!isTauri()) {
      return { name: 'editorMD', version: `${__APP_VERSION__} (navegador, sin backend)` }
    }
    return invoke<AppInfo>('app_info')
  }
}
