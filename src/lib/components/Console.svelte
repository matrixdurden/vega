<script lang="ts">
  import { t } from "../i18n.svelte";
  import { app } from "../state.svelte";
  import Icon from "./Icon.svelte";

  let list = $state<HTMLDivElement>();

  // Follow new lines unless the player scrolled up to read something.
  $effect.pre(() => {
    void app.logs.length;
    if (!list) return;
    const atBottom = list.scrollHeight - list.scrollTop - list.clientHeight < 40;
    if (atBottom) queueMicrotask(() => list && (list.scrollTop = list.scrollHeight));
  });
</script>

<section class="console">
  <div class="head">
    <span><Icon name="terminal" size={16} /> {t("profile.console")}</span>
    <button aria-label={t("window.close")} onclick={() => (app.consoleOpen = false)}><Icon name="x" size={16} /></button>
  </div>
  <div class="lines" bind:this={list}>
    {#each app.logs as entry, i (i)}
      <div class:error={entry.error || /\b(ERROR|FATAL|Exception)\b/.test(entry.line)}>{entry.line}</div>
    {/each}
  </div>
</section>

<style>
  .console {
    position: absolute;
    right: 24px;
    bottom: 24px;
    width: min(620px, calc(100% - 48px));
    height: 260px;
    display: flex;
    flex-direction: column;
    border-radius: var(--radius);
    background: rgb(10 9 24 / 0.92);
    border: 1px solid var(--border-strong);
    backdrop-filter: blur(12px);
    z-index: 15;
    animation: rise 0.2s var(--ease);
  }

  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
  }

  .head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 10px 8px 14px;
    border-bottom: 1px solid var(--border);
    font-size: 13px;
    color: var(--text-2);
  }

  .head span {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .head button {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: 7px;
    color: var(--text-3);
  }

  .head button:hover {
    background: var(--surface-hover);
    color: var(--text);
  }

  .lines {
    flex: 1;
    overflow: auto;
    padding: 10px 14px;
    font: 12px/1.55 ui-monospace, "Cascadia Code", "JetBrains Mono", monospace;
    color: var(--text-2);
    user-select: text;
    white-space: pre-wrap;
    word-break: break-all;
  }

  .error {
    color: var(--danger);
  }
</style>
