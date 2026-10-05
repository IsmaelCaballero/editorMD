/**
 * **Composition root** del frontend: el único lugar donde se crean las piezas
 * concretas (modelo, adaptadores, vistas, controlador) y se conectan entre sí.
 *
 * El resto del código solo conoce interfaces (puertos), así que cambiar una
 * pieza (p. ej. otro editor u otro backend) solo afecta a este fichero.
 *
 * @packageDocumentation
 */
import { mount } from 'svelte'
import { TauriBackend } from './adapters/tauri-backend'
import { TauriFilePicker } from './adapters/tauri-file-picker'
import { onCloseRequested, setNativeTitle } from './adapters/tauri-window'
import './app.css'
import App from './App.svelte'
import { AppController } from './controller/app-controller'
import type { IEditorView } from './controller/ports'
import { DocumentState } from './model/document'
import { DialogService } from './view/dialogs.svelte'
import type { CommandId } from './view/menus'
import { ShellState } from './view/shell.svelte'

/** Documento inicial: sin título y sin cambios, así que «Guardar» pide una ruta. */
const WELCOME = `# Bienvenido a editorMD

Editor **WYSIWYG** de Markdown. Escribe aquí directamente: *cursiva*, **negrita**, \`código\`…

- [x] Editor Milkdown (F1 · paso 3/5)
- [x] Menús conectados al controlador (F1 · paso 4/5)
- [x] Abrir, guardar, guardar como y cerrar ficheros (F2 · paso 5/8): prueba *Archivo → Abrir…* (Ctrl+O)
- [ ] Elegir la codificación y el fin de línea (F2 · paso 6/8)
`

// Modelo y adaptadores.
const document = DocumentState.createNew({}, WELCOME)
const backend = new TauriBackend()
const picker = new TauriFilePicker()
const dialogs = new DialogService()
const shell = new ShellState((title) => void setNativeTitle(title))

let controller: AppController | undefined

const app = mount(App, {
  target: window.document.getElementById('app')!,
  props: {
    shell,
    dialogs,
    onCommand: (id: CommandId) => void controller?.execute(id),
    onEditorReady: (editor: IEditorView) => {
      controller = new AppController({ document, editor, backend, dialogs, window: shell, picker })
      controller.start()
    },
  },
})

// Cerrar la ventana con cambios sin guardar pregunta antes (antes de crear el
// controlador no hay nada que perder).
void onCloseRequested(() => controller?.confirmClose() ?? Promise.resolve(true))

export default app
