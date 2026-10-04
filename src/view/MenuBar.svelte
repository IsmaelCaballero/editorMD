<!--
  @component
  **V01 `MainWindow` · barra de menús.** Dibuja el modelo declarativo de
  `menus.ts` y emite el `CommandId` de la opción pulsada. No decide nada:
  la acción la ejecuta el controlador (Passive View).

  Las opciones de fases futuras aparecen deshabilitadas, con la fase en que
  llegarán como ayuda emergente.
-->
<script lang="ts">
  import {
    type CommandId,
    CURRENT_PHASE,
    formatShortcut,
    isAvailable,
    type Menu,
    MENUS,
    type Phase,
    type Platform,
  } from './menus'

  interface Props {
    /** Orden emitida al pulsar una opción disponible. */
    onCommand: (id: CommandId) => void
    /** Menús a dibujar; por defecto, los de la aplicación. */
    menus?: readonly Menu[]
    /** Fase actual (decide qué opciones están habilitadas). */
    phase?: Phase
    /** Plataforma, para mostrar los atajos (Ctrl o ⌘). */
    platform?: Platform
  }

  let { onCommand, menus = MENUS, phase = CURRENT_PHASE, platform = 'other' }: Props = $props()

  /** Menú desplegado, o `null` si no hay ninguno abierto. */
  let open = $state<string | null>(null)

  function toggle(id: string): void {
    open = open === id ? null : id
  }

  function run(id: CommandId): void {
    open = null
    onCommand(id)
  }

  function onKeydown(event: KeyboardEvent): void {
    if (event.key === 'Escape') open = null
  }
</script>

<svelte:window onkeydown={onKeydown} />

<nav class="menubar" aria-label="Menú principal">
  {#each menus as menu (menu.id)}
    <div class="menu">
      <button
        type="button"
        class="menu-title"
        aria-haspopup="menu"
        aria-expanded={open === menu.id}
        onclick={() => toggle(menu.id)}
        onmouseenter={() => open !== null && (open = menu.id)}>{menu.label}</button
      >
      {#if open === menu.id}
        <ul class="dropdown" role="menu" aria-label={menu.label}>
          {#each menu.entries as entry, i (i)}
            {#if entry.kind === 'separator'}
              <li class="separator" role="separator"></li>
            {:else}
              {@const enabled = isAvailable(entry, phase)}
              <li role="none">
                <button
                  type="button"
                  role="menuitem"
                  data-command={entry.id}
                  disabled={!enabled}
                  title={enabled ? undefined : `Disponible en ${entry.phase}`}
                  onclick={() => run(entry.id)}
                >
                  <span>{entry.label}</span>
                  {#if entry.shortcut}
                    <kbd>{formatShortcut(entry.shortcut, platform)}</kbd>
                  {/if}
                </button>
              </li>
            {/if}
          {/each}
        </ul>
      {/if}
    </div>
  {/each}
</nav>

<style>
  .menubar {
    display: flex;
    gap: 0.1rem;
    padding: 0.15rem 0.4rem;
    border-bottom: 1px solid var(--border);
    background: var(--panel);
    user-select: none;
  }
  .menu {
    position: relative;
  }
  .menu-title {
    background: none;
    border: 0;
    border-radius: 4px;
    padding: 0.25rem 0.6rem;
    color: inherit;
    font: inherit;
    cursor: default;
  }
  .menu-title:hover,
  .menu-title[aria-expanded='true'] {
    background: var(--hover);
  }
  .dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    z-index: 10;
    min-width: 15rem;
    margin: 0.15rem 0 0;
    padding: 0.3rem;
    list-style: none;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 6px;
    box-shadow: 0 6px 18px rgb(0 0 0 / 0.15);
  }
  .dropdown button {
    display: flex;
    justify-content: space-between;
    gap: 2rem;
    width: 100%;
    padding: 0.3rem 0.6rem;
    background: none;
    border: 0;
    border-radius: 4px;
    color: inherit;
    font: inherit;
    text-align: left;
  }
  .dropdown button:not(:disabled):hover {
    background: var(--accent);
    color: #fff;
  }
  .dropdown button:disabled {
    opacity: 0.45;
  }
  kbd {
    font: inherit;
    font-size: 0.85em;
    opacity: 0.7;
  }
  .separator {
    height: 1px;
    margin: 0.3rem 0.2rem;
    background: var(--border);
  }
</style>
