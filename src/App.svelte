<!--
  @component
  **V01 `MainWindow`** (Vista): ventana principal con la barra de menús, el
  editor WYSIWYG (V02), la barra de estado y los diálogos (V12).

  Es una vista pasiva: muestra el estado de `ShellState` y `DialogService`
  y reenvía las órdenes (menús y atajos) con `onCommand`. Quien decide qué
  hacer es C01 `AppController`, que se crea en `main.ts` (composition root)
  cuando el editor está listo (`onEditorReady`).
-->
<script lang="ts">
  import { onMount } from 'svelte'
  import type { IEditorView } from './controller/ports'
  import DialogHost from './view/DialogHost.svelte'
  import type { DialogService } from './view/dialogs.svelte'
  import { ENCODING_LABELS, LINE_ENDING_LABELS } from './view/labels'
  import MenuBar from './view/MenuBar.svelte'
  import { type CommandId, commandForKey, detectPlatform } from './view/menus'
  import { MilkdownEditorView } from './view/milkdown-editor'
  import type { ShellState } from './view/shell.svelte'

  interface Props {
    /** Título y barra de estado (lo escribe el controlador). */
    shell: ShellState
    /** Diálogos modales (los pide el controlador). */
    dialogs: DialogService
    /** Orden emitida desde un menú o un atajo de teclado. */
    onCommand: (id: CommandId) => void
    /** Se llama cuando el editor ya está montado y listo. */
    onEditorReady: (editor: IEditorView) => void
  }

  let { shell, dialogs, onCommand, onEditorReady }: Props = $props()

  /** Versión SemVer inyectada por Vite desde package.json. */
  const version = __APP_VERSION__
  const platform = detectPlatform(navigator.userAgent)
  /** Órdenes que el editor ya gestiona con su propio teclado. */
  const EDITOR_KEYS: CommandId[] = ['edit.undo', 'edit.redo']

  let editorRoot: HTMLDivElement

  onMount(() => {
    let view: MilkdownEditorView | undefined
    MilkdownEditorView.create(editorRoot).then((v) => {
      view = v
      onEditorReady(v)
    })
    return () => view?.destroy()
  })

  function onKeydown(event: KeyboardEvent): void {
    if (dialogs.current) return
    const id = commandForKey(event, platform, { exclude: EDITOR_KEYS })
    if (id) {
      event.preventDefault()
      onCommand(id)
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="window">
  <MenuBar {onCommand} {platform} />

  <main class="editor-area">
    <div class="editor" bind:this={editorRoot}></div>
  </main>

  <footer class="statusbar">
    <span>
      {ENCODING_LABELS[shell.status.encoding]} · {LINE_ENDING_LABELS[shell.status.lineEnding]}
      {#if shell.status.modified}<span class="modified" title="Cambios sin guardar">●</span>{/if}
    </span>
    <span>{shell.status.words} palabras</span>
    <span>v{version}</span>
  </footer>
</div>

<DialogHost service={dialogs} />

<style>
  .window {
    display: grid;
    grid-template-rows: auto 1fr auto;
    height: 100vh;
  }
  .editor-area {
    overflow: auto;
  }
  .editor {
    max-width: 50rem;
    margin: 0 auto;
    padding: 2rem 2.5rem 4rem;
  }
  .statusbar {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.2rem 0.75rem;
    font-size: 0.8rem;
    border-top: 1px solid var(--border);
    background: var(--panel);
  }
  .modified {
    color: var(--accent);
    margin-left: 0.4rem;
  }
</style>
