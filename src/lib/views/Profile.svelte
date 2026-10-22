<script lang="ts">
  import { api } from "../api";
  import Icon from "../components/Icon.svelte";
  import Planet from "../components/Planet.svelte";
  import ProfileIcon from "../components/ProfileIcon.svelte";
  import { relativeTime, t, type Key } from "../i18n.svelte";
  import { app, attempt, launch, selectedProfile, stopGame, type ProfileTab } from "../state.svelte";
  import ModsTab from "./profile/ModsTab.svelte";
  import SettingsTab from "./profile/SettingsTab.svelte";
  import WorldsTab from "./profile/WorldsTab.svelte";

  const profile = $derived(selectedProfile());
  const percent = $derived(app.launch.total ? Math.round((app.launch.done / app.launch.total) * 100) : 0);
  const tabs = $derived<{ id: ProfileTab; label: Key; count?: number }[]>([
    { id: "mods", label: "tab.mods", count: app.mods.length },
    { id: "worlds", label: "tab.worlds", count: app.worlds.length },
    { id: "settings", label: "tab.settings" },
  ]);
</script>

<section class="profile">
  <Planet />

  {#if profile}
    <header>
      <ProfileIcon icon={profile.icon} size={68} />
      <div class="title">
        <h1>{profile.name}</h1>
        <div class="chips">
          <span class="chip"><Icon name="cube" size={13} />Minecraft {profile.gameVersion}</span>
          <span class="chip" class:fabric={profile.loader === "fabric"}>
            {profile.loader === "fabric" ? `Fabric ${profile.loaderVersion ?? ""}` : "Vanilla"}
          </span>
          <span class="chip muted">
            <Icon name="clock" size={13} />
            {profile.lastPlayed ? t("profile.lastPlayed", { when: relativeTime(profile.lastPlayed) }) : t("profile.neverPlayed")}
          </span>
        </div>
      </div>

      <div class="actions">
        <button class="icon-btn" title={t("profile.openFolder")} aria-label={t("profile.openFolder")} onclick={() => attempt(() => api.openProfileFolder(profile.id))}>
          <Icon name="folder" />
        </button>
        <button class="icon-btn" title={t("profile.edit")} aria-label={t("profile.edit")} onclick={() => (app.dialog = { mode: "edit", id: profile.id })}>
          <Icon name="pencil" />
        </button>

        {#if app.launch.phase === "running"}
          <button class="play running" onclick={stopGame}>
            <span class="pulse"></span>
            <span class="label-run">{t("profile.running")}</span>
            <span class="label-stop"><Icon name="stop" size={14} />{t("profile.stop")}</span>
          </button>
        {:else if app.launch.phase === "preparing"}
          <div class="play preparing" role="progressbar" aria-valuenow={percent} aria-valuemin={0} aria-valuemax={100}>
            <div class="fill" style:width="{percent}%"></div>
            <span class="label">
              <Icon name="loader" size={15} class="spin" />
              {t(`stage.${app.launch.stage}` as Key)}
              <b>{percent}%</b>
            </span>
          </div>
        {:else}
          <button class="play" onclick={launch}><Icon name="play" size={16} />{t("profile.play")}</button>
        {/if}
      </div>
    </header>

    <div class="tabs" role="tablist">
      {#each tabs as tab (tab.id)}
        <button role="tab" aria-selected={app.tab === tab.id} class:on={app.tab === tab.id} onclick={() => (app.tab = tab.id)}>
          {t(tab.label)}
          {#if tab.count}<span class="count">{tab.count}</span>{/if}
        </button>
      {/each}
      <span class="spacer"></span>
      {#if app.tab === "mods" && profile.loader === "fabric"}
        <button class="add-mod" onclick={() => (app.modBrowserOpen = true)}><Icon name="plus" size={16} />{t("mods.add")}</button>
      {/if}
    </div>

    <div class="body">
      {#if app.tab === "mods"}
        <ModsTab {profile} />
      {:else if app.tab === "worlds"}
        <WorldsTab {profile} />
      {:else}
        {#key profile.id}<SettingsTab {profile} />{/key}
      {/if}
    </div>

    {#if app.logs.length && !app.consoleOpen}
      <button class="console-toggle" onclick={() => (app.consoleOpen = true)}><Icon name="terminal" size={15} />{t("profile.console")}</button>
    {/if}
  {:else if app.ready}
    <div class="empty">
      <span class="empty-icon"><Icon name="rocket" size={30} /></span>
      <h2>{t("profile.empty")}</h2>
      <p>{t("profile.emptyBody")}</p>
      <button class="btn primary" onclick={() => (app.dialog = { mode: "create" })}><Icon name="plus" size={16} />{t("nav.newProfile")}</button>
    </div>
  {/if}
</section>

<style>
  .profile {
    position: relative;
    height: 100%;
    display: flex;
    flex-direction: column;
    padding: 46px 36px 0;
    overflow: hidden;
  }

  header {
    position: relative;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 16px 18px;
  }

  .title {
    flex: 1 1 240px;
    min-width: 0;
  }

  h1 {
    font-size: 28px;
    font-weight: 650;
    letter-spacing: -0.02em;
    line-height: 1.15;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 8px;
  }

  .chip {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 24px;
    padding: 0 10px;
    border-radius: 999px;
    background: var(--surface);
    border: 1px solid var(--border);
    color: var(--text-2);
    font-size: 12px;
    white-space: nowrap;
  }

  .chip.fabric {
    background: var(--accent-soft);
    border-color: rgb(127 119 221 / 0.3);
    color: #cecbf6;
  }

  .chip.muted {
    color: var(--text-3);
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .icon-btn {
    width: 44px;
    height: 44px;
    display: grid;
    place-items: center;
    border-radius: 12px;
    background: var(--surface);
    border: 1px solid var(--border);
    color: var(--text-2);
    transition: background 0.15s, color 0.15s;
  }

  .icon-btn:hover {
    background: var(--surface-hover);
    color: var(--text);
  }

  .play {
    position: relative;
    height: 48px;
    min-width: 150px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 9px;
    padding: 0 26px;
    border-radius: 13px;
    background: var(--accent);
    color: #fff;
    font-size: 15px;
    font-weight: 600;
    overflow: hidden;
    box-shadow: 0 10px 28px rgb(127 119 221 / 0.35);
    transition: background 0.15s, transform 0.15s var(--ease);
  }

  button.play:hover {
    background: var(--accent-hover);
    transform: translateY(-1px);
  }

  .preparing {
    min-width: 220px;
    background: var(--accent-soft);
    border: 1px solid rgb(127 119 221 / 0.4);
    box-shadow: none;
    font-size: 13px;
    font-weight: 500;
  }

  .fill {
    position: absolute;
    inset: 0 auto 0 0;
    background: rgb(127 119 221 / 0.45);
    transition: width 0.25s var(--ease);
  }

  .label {
    position: relative;
    display: flex;
    align-items: center;
    gap: 7px;
  }

  .label b {
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }

  .running {
    background: rgb(93 202 165 / 0.14);
    border: 1px solid rgb(93 202 165 / 0.35);
    color: var(--success);
    box-shadow: none;
    font-size: 13px;
    font-weight: 500;
  }

  button.running:hover {
    background: var(--danger-soft);
    border-color: rgb(240 149 149 / 0.4);
    color: var(--danger);
    transform: none;
  }

  .label-stop,
  .running:hover .label-run,
  .running:hover .pulse {
    display: none;
  }

  .running:hover .label-stop {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  .pulse {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--success);
    animation: pulse 1.6s ease-in-out infinite;
  }

  @keyframes pulse {
    50% {
      opacity: 0.35;
    }
  }

  .tabs {
    position: relative;
    display: flex;
    align-items: center;
    gap: 4px;
    margin-top: 26px;
    padding-bottom: 10px;
    border-bottom: 1px solid var(--border);
  }

  .tabs [role="tab"] {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 34px;
    padding: 0 14px;
    border-radius: 9px;
    color: var(--text-2);
    font-weight: 500;
  }

  .tabs [role="tab"]:hover {
    color: var(--text);
  }

  .tabs [role="tab"].on {
    background: var(--surface-hover);
    color: var(--text);
  }

  .count {
    min-width: 20px;
    padding: 0 6px;
    border-radius: 99px;
    background: var(--surface-hover);
    color: var(--text-3);
    font-size: 11px;
    line-height: 18px;
    text-align: center;
  }

  .spacer {
    flex: 1;
  }

  .add-mod {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 34px;
    padding: 0 14px;
    border-radius: 10px;
    background: var(--accent-soft);
    border: 1px solid rgb(127 119 221 / 0.35);
    color: #cecbf6;
    font-weight: 500;
  }

  .add-mod:hover {
    background: rgb(127 119 221 / 0.26);
  }

  .body {
    position: relative;
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    margin: 0 -10px;
    padding: 14px 10px 40px;
  }

  .console-toggle {
    position: absolute;
    right: 24px;
    bottom: 20px;
    display: flex;
    align-items: center;
    gap: 7px;
    height: 32px;
    padding: 0 12px;
    border-radius: 10px;
    background: var(--bg-raised);
    border: 1px solid var(--border);
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
    color: var(--text-3);
  }

  .empty-icon {
    width: 64px;
    height: 64px;
    display: grid;
    place-items: center;
    border-radius: 18px;
    background: var(--accent-soft);
    color: #cecbf6;
  }

  .empty h2 {
    color: var(--text);
    font-size: 20px;
    font-weight: 600;
  }

  .empty p {
    margin-bottom: 8px;
  }
</style>
