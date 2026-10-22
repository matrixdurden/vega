<script lang="ts">
  import Icon from "../components/Icon.svelte";
  import Planet from "../components/Planet.svelte";
  import ProfileMenu from "../components/ProfileMenu.svelte";
  import { t, type Key } from "../i18n.svelte";
  import { app, launch, selectedProfile, stopGame } from "../state.svelte";

  const MAX_CHIPS = 5;

  const profile = $derived(selectedProfile());
  const picked = $derived(app.mods.filter((mod) => !mod.dependency));
  const percent = $derived(app.launch.total ? Math.round((app.launch.done / app.launch.total) * 100) : 0);
</script>

<section class="home">
  <Planet />

  <div class="hero">
    <p class="greet">{t("home.welcome")}</p>
    <h1>{app.settings?.username ?? ""}</h1>

    {#if !profile && app.ready}
      <p class="greet"><Icon name="loader" size={16} class="spin" /> {t("home.firstRun")}</p>
    {/if}

    <div class="actions">
      {#if app.launch.phase === "running"}
        <button class="play running" onclick={stopGame}>
          <span class="pulse"></span>
          {t("home.running")}
          <span class="stop"><Icon name="stop" size={14} /> {t("home.stop")}</span>
        </button>
      {:else if app.launch.phase === "preparing"}
        <div class="play preparing" role="progressbar" aria-valuenow={percent} aria-valuemin={0} aria-valuemax={100}>
          <div class="fill" style:width="{percent}%"></div>
          <span class="label">
            <Icon name="loader" size={16} class="spin" />
            {t(`stage.${app.launch.stage}` as Key)}
            <b>{percent}%</b>
          </span>
        </div>
      {:else}
        <button class="play" onclick={launch} disabled={!profile}>
          <Icon name="play" size={18} />
          {t("home.play")}
        </button>
      {/if}
      <ProfileMenu />
    </div>

    <div class="mods">
      <span class="caption">{t("home.mods")}</span>
      {#if profile?.loader === "vanilla"}
        <span class="chip muted">{t("home.vanillaNote")}</span>
      {:else}
        {#each picked.slice(0, MAX_CHIPS) as mod (mod.projectId)}
          <span class="chip">
            {#if mod.iconUrl}<img src={mod.iconUrl} alt="" />{/if}
            {mod.title}
          </span>
        {:else}
          <span class="chip muted">{t("home.noMods")}</span>
        {/each}
        {#if picked.length > MAX_CHIPS}
          <span class="chip muted">+{picked.length - MAX_CHIPS}</span>
        {/if}
        <button class="chip add" onclick={() => (app.view = "mods")}><Icon name="plus" size={14} />{t("home.addMod")}</button>
      {/if}
    </div>
  </div>

  {#if app.logs.length && !app.consoleOpen}
    <button class="console-toggle" onclick={() => (app.consoleOpen = true)}>
      <Icon name="terminal" size={16} />
      {t("home.console")}
    </button>
  {/if}
</section>

<style>
  .home {
    position: relative;
    height: 100%;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    padding: 40px 44px 44px;
    overflow: hidden;
  }

  .hero {
    position: relative;
    z-index: 1;
  }

  .greet {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--text-2);
    font-size: 15px;
  }

  h1 {
    margin: 2px 0 26px;
    font-size: 46px;
    font-weight: 650;
    letter-spacing: -0.02em;
    line-height: 1.1;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 12px;
  }

  .play {
    position: relative;
    height: 52px;
    min-width: 170px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    padding: 0 30px;
    border-radius: var(--radius);
    background: var(--accent);
    color: #fff;
    font-size: 16px;
    font-weight: 600;
    overflow: hidden;
    box-shadow: 0 10px 30px rgb(127 119 221 / 0.35);
    transition: background 0.15s, transform 0.15s var(--ease), box-shadow 0.15s;
  }

  button.play:hover:not(:disabled) {
    background: var(--accent-hover);
    transform: translateY(-1px);
    box-shadow: 0 14px 36px rgb(127 119 221 / 0.45);
  }

  button.play:active:not(:disabled) {
    background: var(--accent-press);
    transform: none;
  }

  .preparing {
    min-width: 260px;
    background: var(--accent-soft);
    border: 1px solid rgb(127 119 221 / 0.4);
    box-shadow: none;
    font-size: 14px;
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
    gap: 8px;
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
    font-size: 14px;
    font-weight: 500;
  }

  button.running:hover:not(:disabled) {
    background: var(--danger-soft);
    border-color: rgb(240 149 149 / 0.4);
    color: var(--danger);
    box-shadow: none;
  }

  .running .stop {
    display: none;
    align-items: center;
    gap: 6px;
  }

  .running:hover .stop {
    display: inline-flex;
  }

  .running:hover .pulse {
    display: none;
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

  .mods {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    margin-top: 30px;
  }

  .caption {
    margin-right: 4px;
    font-size: 12px;
    color: var(--text-3);
  }

  .chip {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    height: 32px;
    padding: 0 12px;
    border-radius: 999px;
    background: var(--surface);
    border: 1px solid var(--border);
    color: #cecbf6;
    font-size: 12px;
    backdrop-filter: blur(8px);
  }

  .chip img {
    width: 18px;
    height: 18px;
    border-radius: 5px;
  }

  .chip.muted {
    color: var(--text-3);
  }

  .chip.add {
    border-style: dashed;
    color: var(--text-2);
    transition: background 0.15s, color 0.15s;
  }

  .chip.add:hover {
    background: var(--surface-hover);
    color: var(--text);
  }

  .console-toggle {
    position: absolute;
    right: 24px;
    bottom: 24px;
    z-index: 2;
    display: flex;
    align-items: center;
    gap: 8px;
    height: 34px;
    padding: 0 12px;
    border-radius: 10px;
    background: var(--surface);
    border: 1px solid var(--border);
    color: var(--text-2);
    font-size: 12px;
  }

  .console-toggle:hover {
    background: var(--surface-hover);
    color: var(--text);
  }
</style>
