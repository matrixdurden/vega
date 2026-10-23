<script lang="ts">
  import { t } from "../i18n.svelte";
  import { app } from "../state.svelte";
  import Icon from "./Icon.svelte";

  let { profileId }: { profileId: string } = $props();

  let list = $state<HTMLDivElement>();
  const logs = $derived(app.logs[profileId] ?? []);
  const name = $derived(app.profiles.find((p) => p.id === profileId)?.name ?? "");

  // Follow new lines unless the player scrolled up to read something.
  $effect.pre(() => {
    void logs.length;
    if (!list) return;
    const atBottom = list.scrollHeight - list.scrollTop - list.clientHeight < 40;
    if (atBottom) queueMicrotask(() => list && (list.scrollTop = list.scrollHeight));
  });
</script>

<section class="console">
  <div class="head">
    <span class="mc"><Icon name="terminal" size={16} /> {t("profile.console")} · {name}</span>
    <button aria-label={t("window.close")} onclick={() => (app.console = null)}><Icon name="x" size={16} /></button>
  </div>
  <div class="lines" bind:this={list}>
    {#each logs as entry, i (i)}
      <div class:error={entry.error || /\b(ERROR|FATAL|Exception)\b/.test(entry.line)}>{entry.line}</div>
    {/each}
  </div>
</section>

<style>
  .console {
    position: absolute;
    right: 24px;
    bottom: 24px;
    width: min(760px, calc(100% - 48px));
    height: min(320px, 50%);
    display: flex;
    flex-direction: column;
    border-radius: var(--radius);
    background: #111114;
    border: 1px solid var(--border-strong);
    box-shadow: 0 16px 40px rgb(0 0 0 / 0.5);
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
    background: var(--panel-2);
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
