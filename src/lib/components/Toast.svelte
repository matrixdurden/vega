<script lang="ts">
  import { app } from "../state.svelte";
  import Icon from "./Icon.svelte";
</script>

{#if app.toast}
  {#key app.toast.id}
    <div class="toast" class:error={app.toast.kind === "error"} role="status">
      <Icon name={app.toast.kind === "error" ? "alert" : "check"} size={18} />
      <span>{app.toast.text}</span>
      <button aria-label="OK" onclick={() => (app.toast = null)}><Icon name="x" size={14} /></button>
    </div>
  {/key}
{/if}

<style>
  .toast {
    position: fixed;
    top: 18px;
    left: 50%;
    z-index: 60;
    display: flex;
    align-items: center;
    gap: 10px;
    max-width: min(520px, calc(100vw - 32px));
    padding: 10px 10px 10px 14px;
    border-radius: 12px;
    background: var(--bg-raised);
    border: 1px solid var(--border-strong);
    box-shadow: 0 16px 40px rgb(0 0 0 / 0.45);
    color: var(--success);
    transform: translateX(-50%);
    animation: drop 0.25s var(--ease);
  }

  .toast.error {
    color: var(--danger);
  }

  span {
    color: var(--text);
    font-size: 13px;
  }

  button {
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    border-radius: 6px;
    color: var(--text-3);
  }

  button:hover {
    background: var(--surface-hover);
  }

  @keyframes drop {
    from {
      opacity: 0;
      transform: translate(-50%, -8px);
    }
  }
</style>
