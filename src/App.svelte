<!--
  @component
  **V01 `MainWindow`** (Vista): ventana principal con la barra de menús, el
  editor WYSIWYG (V02) y la barra de estado.

  F1 · paso 3/5: los menús y el editor ya funcionan, pero las órdenes de menú
  solo se registran en la consola. En el paso 4/5, C01 `AppController` se
  conectará en `main.ts` (composition root) y las ejecutará.
-->
<script lang="ts">
  import { onMount } from 'svelte'
  import MenuBar from './view/MenuBar.svelte'
  import { type CommandId, detectPlatform } from './view/menus'
  import { MilkdownEditorView } from './view/milkdown-editor'

  /** Versión SemVer inyectada por Vite desde package.json. */
  const version = __APP_VERSION__
  const platform = detectPlatform(navigator.userAgent)

  /** Documento de bienvenida hasta que exista «Abrir» (F2). */
  const WELCOME = `# Bienvenido a editorMD

Editor **WYSIWYG** de Markdown. Escribe aquí directamente: *cursiva*, **negrita**, \`código\`…

- [x] Editor Milkdown funcionando (F1 · paso 3/5)
- [ ] Abrir y guardar ficheros (F2)

| Atajo | Acción |
| --- | --- |
| Ctrl/⌘ + Z | Deshacer |
| Ctrl/⌘ + Shift + Z | Rehacer |
`

  let editorRoot: HTMLDivElement
  let words = $state(0)

  const countWords = (md: string) => (md.match(/[\p{L}\p{N}]+/gu) ?? []).length

  onMount(() => {
    let view: MilkdownEditorView | undefined
    MilkdownEditorView.create(editorRoot, WELCOME).then((v) => {
      view = v
      words = countWords(v.getMarkdown())
      v.onChange((md) => (words = countWords(md)))
      v.focus()
    })
    return () => view?.destroy()
  })

  function onCommand(id: CommandId): void {
    // Temporal hasta el paso 4/5: el controlador C01 ejecutará las órdenes.
    console.info(`[editorMD] orden «${id}»: la ejecutará C01 AppController (F1 · paso 4/5)`)
  }
</script>

<div class="window">
  <MenuBar {onCommand} {platform} />

  <main class="editor-area">
    <div class="editor" bind:this={editorRoot}></div>
  </main>

  <footer class="statusbar">
    <span>UTF-8 · LF</span>
    <span>{words} palabras</span>
    <span>v{version}</span>
  </footer>
</div>

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
</style>
