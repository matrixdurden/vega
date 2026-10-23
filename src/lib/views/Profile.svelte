<script lang="ts">
  import Backdrop from "../components/Backdrop.svelte";
  import Icon from "../components/Icon.svelte";
  import ProfileIcon from "../components/ProfileIcon.svelte";
  import { t, type Key } from "../i18n.svelte";
  import { app, launch, selectedProfile, stopGame } from "../state.svelte";
  import ModsTab from "./profile/ModsTab.svelte";
  import SettingsTab from "./profile/SettingsTab.svelte";

  const profile = $derived(selectedProfile());
  const percent = $derived(app.launch.total ? Math.round((app.launch.done / app.launch.total) * 100) : 0);
</script>

<section class="profile">
  <Backdrop />

  {#if profile}
    <header>
      <ProfileIcon icon={profile.icon} size={64} />
      <div class="title">
        <h1>{profile.name}</h1>
        <span>{profile.loader === "fabric" ? "Fabric" : "Vanilla"} · {profile.gameVersion}</span>
      </div>

      {#if app.launch.phase === "running"}
        <button class="btn play running" onclick={stopGame}>
          <span class="label-run">{t("profile.running")}</span>
          <span class="label-stop"><Icon name="stop" size={14} />{t("profile.stop")}</span>
        </button>
      {:else if app.launch.phase === "preparing"}
        <div class="btn play preparing" role="progressbar" aria-valuenow={percent} aria-valuemin={0} aria-valuemax={100}>
          <div class="fill" style:width="{percent}%"></div>
          <span class="label">{t(`stage.${app.launch.stage}` as Key)} {percent}%</span>
        </div>
      {:else}
        <button class="btn green play" onclick={launch}>{t("profile.play")}</button>
      {/if}
    </header>

    <div class="tabs" role="tablist">
      <button role="tab" aria-selected={app.tab === "mods"} class:on={app.tab === "mods"} onclick={() => (app.tab = "mods")}>
        {t("tab.mods")}
        {#if app.mods.length}<span class="count">{app.mods.length}</span>{/if}
      </button>
      <button role="tab" aria-selected={app.tab === "settings"} class:on={app.tab === "settings"} onclick={() => (app.tab = "settings")}>
        {t("tab.settings")}
      </button>
    </div>

    <div class="body">
      {#if app.tab === "mods"}
        <ModsTab {profile} />
      {:else}
        {#key profile.id}<SettingsTab {profile} />{/key}
      {/if}
    </div>

    {#if app.logs.length && !app.consoleOpen}
      <button class="console-toggle" onclick={() => (app.consoleOpen = true)}><Icon name="terminal" size={15} />{t("profile.console")}</button>
    {/if}
  {:else if app.ready}
    <div class="empty">
      <h2>{t("profile.empty")}</h2>
      <p class="hint">{t("profile.emptyBody")}</p>
      <button class="btn green" onclick={() => (app.createOpen = true)}><Icon name="plus" size={16} />{t("nav.newProfile")}</button>
    </div>
  {/if}
</section>

<style>
  .profile {
    position: relative;
    height: 100%;
    display: flex;
    flex-direction: column;
    padding: 64px 32px 0;
  }

  header {
    position: relative;
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .title {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  h1 {
    font-size: 26px;
    line-height: 1.2;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    text-shadow: 0 2px 0 rgb(0 0 0 / 0.45);
  }

  .title span {
    color: var(--text-2);
    font-size: 13px;
  }

  .play {
    position: relative;
    height: 52px;
    min-width: 180px;
    padding: 0 28px;
    overflow: hidden;
    font-size: 20px;
  }

  .preparing,
  .running {
    font-size: 13px;
  }

  .preparing {
    background: var(--bg);
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

  .running {
    background: #1d3a1a;
    border-color: #2e5c28;
    color: var(--green-text);
  }

  .running:hover {
    background: #3a1c1c !important;
    border-color: #6b2d2d;
    color: var(--danger);
  }

  .label-stop,
  .running:hover .label-run {
    display: none;
  }

  .running:hover .label-stop {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  .tabs {
    position: relative;
    display: flex;
    gap: 4px;
    margin-top: 24px;
    border-bottom: 1px solid var(--border);
  }

  .tabs [role="tab"] {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 38px;
    padding: 0 14px;
    margin-bottom: -1px;
    border-bottom: 2px solid transparent;
    color: var(--text-3);
    font-family: var(--mc);
    font-size: 14px;
    font-variant-ligatures: none;
  }

  .tabs [role="tab"]:hover {
    color: var(--text);
  }

  .tabs [role="tab"].on {
    color: var(--text);
    border-bottom-color: var(--green-text);
  }

  .count {
    padding: 0 6px;
    border-radius: 3px;
    background: var(--panel-2);
    color: var(--text-2);
    font-size: 11px;
  }

  .body {
    position: relative;
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    margin: 0 -8px;
    padding: 16px 8px 40px;
  }

  .console-toggle {
    position: absolute;
    right: 20px;
    bottom: 18px;
    display: flex;
    align-items: center;
    gap: 7px;
    height: 30px;
    padding: 0 10px;
    border-radius: 4px;
    background: var(--panel);
    border: 1px solid var(--border-strong);
    color: var(--text-2);
    font-size: 12px;
  }

  .empty {
    position: relative;
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    text-align: center;
  }

  .empty h2 {
    font-size: 22px;
  }
</style>
