<!--
  @component
  **V01 `MainWindow`** (Vista), esqueleto de F0.
  Por ahora solo muestra la estructura de la ventana y la versión. Los menús y
  el editor Milkdown llegan en F1, cuando ya estén conectados a sus controladores.
-->
<script lang="ts">
  /** Menús previstos (PLAN.md §2). Todavía son decorativos. */
  const menus = ['Archivo', 'Editar', 'Formato', 'Insertar', 'Ver', 'Ayuda']
  /** Versión SemVer inyectada por Vite desde package.json. */
  const version = __APP_VERSION__
</script>

<div class="window">
  <nav class="menubar" aria-label="Menú principal">
    {#each menus as menu (menu)}
      <button type="button" disabled>{menu}</button>
    {/each}
  </nav>

  <main class="editor-area">
    <h1>editorMD</h1>
    <p>Esqueleto F0 · el editor WYSIWYG llegará en F1.</p>
  </main>

  <footer class="statusbar">
    <span>UTF-8 · LF</span>
    <span>v{version}</span>
  </footer>
</div>

<style>
  .window {
    display: grid;
    grid-template-rows: auto 1fr auto;
    height: 100vh;
  }
  .menubar {
    display: flex;
    gap: 0.25rem;
    padding: 0.25rem 0.5rem;
    border-bottom: 1px solid var(--border);
    background: var(--panel);
  }
  .menubar button {
    background: none;
    border: 0;
    padding: 0.25rem 0.6rem;
    color: inherit;
    font: inherit;
  }
  .editor-area {
    padding: 2rem;
    overflow: auto;
  }
  .statusbar {
    display: flex;
    justify-content: space-between;
    padding: 0.2rem 0.75rem;
    font-size: 0.8rem;
    border-top: 1px solid var(--border);
    background: var(--panel);
  }
</style>
