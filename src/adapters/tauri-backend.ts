/**
 * **A01 `TauriBackend`** (Adaptador · TS): implementa el puerto {@link IBackend}
 * llamando a los comandos Rust de Tauri por IPC.
 *
 * Fuera de Tauri (p. ej. `npm run dev` en un navegador) no hay backend Rust:
 * `appInfo` responde con valores del propio frontend para que la interfaz siga
 * siendo usable durante el desarrollo, y las operaciones de ficheros fallan con
 * un {@link FileError} de tipo `io`.
 *
 * @packageDocumentation
 */
import { invoke, isTauri } from '@tauri-apps/api/core'
import type { Encoding } from '../model/document'
import type {
  AppInfo,
  FileError,
  IBackend,
  OpenedFile,
  SavedFile,
  SaveRequest,
} from '../controller/ports'

/** Adaptador del backend Rust (IPC de Tauri). */
export class TauriBackend implements IBackend {
  async appInfo(): Promise<AppInfo> {
    if (!isTauri()) {
      return { name: 'editorMD', version: `${__APP_VERSION__} (navegador, sin backend)` }
    }
    return invoke<AppInfo>('app_info')
  }

  async openFile(path: string, encoding?: Encoding): Promise<OpenedFile> {
    if (!isTauri()) throw noBackend(path)
    return invoke<OpenedFile>('open_file', { path, encoding: encoding ?? null })
  }

  async saveFile(request: SaveRequest): Promise<SavedFile> {
    if (!isTauri()) throw noBackend(request.path)
    const { path, text, encoding, lineEnding, strategy } = request
    return invoke<SavedFile>('save_file', {
      path,
      text,
      encoding,
      lineEnding,
      strategy: strategy ?? null,
    })
  }
}

function noBackend(path: string): FileError {
  return {
    kind: 'io',
    path,
    message: 'no hay backend Rust (la aplicación se ejecuta en un navegador)',
  }
}
