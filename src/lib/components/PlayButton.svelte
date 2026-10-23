<script lang="ts">
  import type { Profile } from "../api";
  import { t, type Key } from "../i18n.svelte";
  import { app, launch, stopGame } from "../state.svelte";
  import Icon from "./Icon.svelte";

  let { profile }: { profile: Profile } = $props();

  const run = $derived(app.runs[profile.id]);
  const percent = $derived(run?.total ? Math.round((run.done / run.total) * 100) : 0);
  const hasLogs = $derived((app.logs[profile.id]?.length ?? 0) > 0);
  const toggleConsole = () => (app.console = app.console === profile.id ? null : profile.id);
</script>

<div class="play">
  {#if run?.phase === "preparing"}
    <div class="btn main preparing" role="progressbar" aria-valuenow={percent} aria-valuemin={0} aria-valuemax={100}>
      <div class="fill" style:width="{percent}%"></div>
      <span class="label">{t(`stage.${run.stage}` as Key)} {percent}%</span>
    </div>
  {:else if run}
    <button class="btn main stop" onclick={() => stopGame(profile.id)}>
      <Icon name="stop" size={14} />{t("profile.stop")}
    </button>
  {:else}
    <button class="btn green main" onclick={() => launch(profile.id)}>{t("profile.play")}</button>
  {/if}
  {#if run || hasLogs}
    <!-- The log sits inside the play control, on its right. -->
    <button class="btn log" class:on={app.console === profile.id} aria-label={t("profile.console")} title={t("profile.console")} onclick={toggleConsole}>
      <Icon name="terminal" size={17} />
    </button>
  {/if}
</div>

<style>
  .play {
    display: flex;
    flex-shrink: 0;
  }

  .main {
    position: relative;
    height: 52px;
    min-width: 170px;
    padding: 0 28px;
    overflow: hidden;
    font-size: 20px;
  }

  .play:has(.log) .main {
    border-top-right-radius: 0;
    border-bottom-right-radius: 0;
  }

  .log {
    height: 52px;
    width: 46px;
    padding: 0;
    border-left: none;
    border-top-left-radius: 0;
    border-bottom-left-radius: 0;
    color: var(--text-2);
  }

  .log.on {
    color: var(--green-text);
  }

  .preparing {
    background: var(--bg);
    font-size: 13px;
  }

  .fill {
    position: absolute;
    inset: 0 auto 0 0;
    background: var(--green);
    transition: width 0.2s;
  }

  .label {
    position: relative;
    font-variant-numeric: tabular-nums;
  }

  .stop {
    background: #4a1e1e;
    border-color: #6b2d2d;
    color: #ffd2cc;
    font-size: 16px;
  }

  .stop:hover:not(:disabled) {
    background: #5a2424;
  }
</style>
