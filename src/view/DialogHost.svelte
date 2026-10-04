<!--
  @component
  **V12 · DialogHost.** Dibuja el diálogo modal pendiente de un `DialogService`.
  Escape equivale a «Aceptar» (mensajes) o «Cancelar» (cambios sin guardar).
-->
<script lang="ts">
  import type { DialogService } from './dialogs.svelte'

  interface Props {
    /** Servicio cuyo diálogo actual se muestra. */
    service: DialogService
  }

  let { service }: Props = $props()

  /** Pone el foco en el botón principal al abrir el diálogo. */
  function autofocus(node: HTMLElement): void {
    node.focus()
  }

  function onKeydown(event: KeyboardEvent): void {
    const c = service.current
    if (!c || event.key !== 'Escape') return
    event.preventDefault()
    if (c.kind === 'message') c.close()
    else c.answer('cancel')
  }
</script>

<svelte:window onkeydown={onKeydown} />

{#if service.current}
  {@const c = service.current}
  <div class="backdrop">
    <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="dialog-title">
      {#if c.kind === 'message'}
        <h2 id="dialog-title">{c.title}</h2>
        <p class="text">{c.text}</p>
        <div class="buttons">
          <button type="button" class="primary" use:autofocus onclick={() => c.close()}
            >Aceptar</button
          >
        </div>
      {:else}
        <h2 id="dialog-title">Cambios sin guardar</h2>
        <p class="text">«{c.documentName}» tiene cambios sin guardar. ¿Qué quieres hacer?</p>
        <div class="buttons">
          <button type="button" data-choice="cancel" onclick={() => c.answer('cancel')}
            >Cancelar</button
          >
          <button type="button" data-choice="discard" onclick={() => c.answer('discard')}
            >Descartar cambios</button
          >
          <button
            type="button"
            class="primary"
            data-choice="save"
            use:autofocus
            onclick={() => c.answer('save')}>Guardar</button
          >
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 100;
    display: grid;
    place-items: center;
    background: rgb(0 0 0 / 0.35);
  }
  .dialog {
    width: min(28rem, calc(100vw - 2rem));
    padding: 1.25rem 1.5rem;
    background: var(--bg);
    color: var(--fg);
    border: 1px solid var(--border);
    border-radius: 10px;
    box-shadow: 0 12px 40px rgb(0 0 0 / 0.3);
  }
  h2 {
    margin: 0 0 0.75rem;
    font-size: 1.1rem;
  }
  .text {
    margin: 0 0 1.25rem;
    white-space: pre-line;
    line-height: 1.5;
  }
  .buttons {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
  }
  button {
    font: inherit;
    padding: 0.4rem 0.9rem;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--panel);
    color: inherit;
  }
  button.primary {
    background: var(--accent);
    border-color: var(--accent);
    color: #fff;
  }
</style>
