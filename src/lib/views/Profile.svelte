<script lang="ts">
  import Backdrop from "../components/Backdrop.svelte";
  import Icon from "../components/Icon.svelte";
  import PlayButton from "../components/PlayButton.svelte";
  import ProfileIcon from "../components/ProfileIcon.svelte";
  import { t } from "../i18n.svelte";
  import { app, selectedProfile } from "../state.svelte";
  import ModsTab from "./profile/ModsTab.svelte";
  import SettingsTab from "./profile/SettingsTab.svelte";

  const profile = $derived(selectedProfile());
  // Vanilla has nothing to mod, so it only shows its settings.
  const modded = $derived(profile?.loader === "fabric");
  const tab = $derived(modded ? app.tab : "settings");
</script>

<section class="profile">
  <Backdrop />

  {#if profile}
    <header>
      <ProfileIcon icon={profile.icon} size={64} />
      <div class="title">
        <h1>{profile.name}</h1>
        <span>{profile.gameVersion} · {modded ? "Fabric" : "Vanilla"}</span>
      </div>
      <PlayButton {profile} />
    </header>

    {#if modded}
      <div class="tabs" role="tablist">
        <button role="tab" aria-selected={tab === "mods"} class:on={tab === "mods"} onclick={() => (app.tab = "mods")}>
          {t("tab.mods")}
          {#if app.mods.length}<span class="count">{app.mods.length}</span>{/if}
        </button>
        <button role="tab" aria-selected={tab === "settings"} class:on={tab === "settings"} onclick={() => (app.tab = "settings")}>
          {t("tab.settings")}
        </button>
      </div>
    {:else}
      <div class="spacer"></div>
    {/if}

    <div class="body">
      {#if tab === "mods"}
        <ModsTab {profile} />
      {:else}
        {#key profile.id}<SettingsTab {profile} />{/key}
      {/if}
    </div>
  {:else if app.ready}
    <div class="empty">
      <h2>{t("profile.empty")}</h2>
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
    padding: 64px clamp(20px, 3vw, 48px) 0;
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

  .spacer {
    height: 24px;
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

  .empty {
    position: relative;
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 14px;
  }

  .empty h2 {
    font-size: 22px;
  }
</style>
