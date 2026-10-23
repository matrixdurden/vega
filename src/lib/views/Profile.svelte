<script lang="ts">
  import Icon from "../components/Icon.svelte";
  import ProfileIcon from "../components/ProfileIcon.svelte";
  import { t, type Key } from "../i18n.svelte";
  import { app, launch, selectedProfile, stopGame } from "../state.svelte";
  import ModsTab from "./profile/ModsTab.svelte";
  import SettingsTab from "./profile/SettingsTab.svelte";

  const profile = $derived(selectedProfile());
  const percent = $derived(app.launch.total ? Math.round((app.launch.done / app.launch.total) * 100) : 0);
  const subtitle = $derived(profile ? `${profile.loader === "fabric" ? "Fabric" : "Vanilla"} · ${profile.gameVersion}` : "");
</script>

<section class="profile">
  {#if profile}
    <header>
      <ProfileIcon icon={profile.icon} size={56} />
      <div class="title">
        <h1>{profile.name}</h1>
        <span>{subtitle}</span>
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
      <span class="spacer"></span>
      {#if app.tab === "mods" && profile.loader === "fabric"}
        <button class="btn add-mod" onclick={() => (app.modBrowserOpen = true)}><Icon name="plus" size={15} />{t("mods.add")}</button>
      {/if}
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
      <p>{t("profile.emptyBody")}</p>
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
    padding: 44px 32px 0;
  }

  header {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .title {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  h1 {
    font-size: 28px;
    line-height: 1.15;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .title span {
    color: var(--text-3);
    font-size: 13px;
  }

  .play {
    position: relative;
    height: 50px;
    min-width: 170px;
    padding: 0 28px;
    overflow: hidden;
    font-family: var(--pixel);
    font-size: 20px;
  }

  .preparing,
  .running {
    font-family: inherit;
    font-size: 13px;
  }

  .preparing {
    background: #120f26;
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
    background: #13301a;
    color: var(--success);
  }

  .running:hover {
    background: #3a1717 !important;
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
    display: flex;
    align-items: center;
    gap: 4px;
    margin-top: 22px;
    padding-bottom: 10px;
    border-bottom: 2px solid #1d1a38;
  }

  .tabs [role="tab"] {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 32px;
    padding: 0 12px;
    border-radius: 4px;
    color: var(--text-3);
    font-family: var(--pixel);
    font-size: 15px;
  }

  .tabs [role="tab"]:hover {
    color: var(--text);
  }

  .tabs [role="tab"].on {
    background: var(--surface-hover);
    color: var(--text);
  }

  .count {
    font-family: inherit;
    font-size: 12px;
    color: var(--text-3);
  }

  .spacer {
    flex: 1;
  }

  .add-mod {
    height: 32px;
    font-size: 13px;
  }

  .body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    margin: 0 -8px;
    padding: 14px 8px 40px;
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
    background: var(--bg-raised);
    border: 1px solid var(--border);
    color: var(--text-2);
    font-size: 12px;
  }

  .empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    color: var(--text-3);
    text-align: center;
  }

  .empty h2 {
    color: var(--text);
    font-size: 22px;
  }

  .empty p {
    margin-bottom: 8px;
  }
</style>
