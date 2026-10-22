<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { isTauri } from "../api";
  import Icon from "../components/Icon.svelte";
  import MemorySlider from "../components/MemorySlider.svelte";
  import { t } from "../i18n.svelte";
  import { app, updateSettings } from "../state.svelte";

  const USERNAME = /^\w{3,16}$/;
  const REPO = "https://github.com/matrixdurden/vega";

  let username = $state(app.settings?.username ?? "");
  const usernameValid = $derived(USERNAME.test(username));

  function onUsername() {
    if (usernameValid) updateSettings({ username });
  }

  const openRepo = () => (isTauri ? openUrl(REPO) : window.open(REPO, "_blank"));
</script>

{#if app.settings}
  <section class="settings">
    <h1>{t("settings.title")}</h1>

    <div class="card">
      <div class="row">
        <div class="text">
          <span class="label"><Icon name="user" size={16} />{t("settings.username")}</span>
          <span class="hint" class:invalid={!usernameValid}>{usernameValid ? t("settings.usernameHint") : t("settings.usernameInvalid")}</span>
        </div>
        <input class="input" class:bad={!usernameValid} bind:value={username} oninput={onUsername} maxlength="16" spellcheck="false" />
      </div>

      <div class="row">
        <div class="text">
          <span class="label"><Icon name="memory" size={16} />{t("settings.memory")}</span>
          <span class="hint">{t("settings.memoryHint")}</span>
        </div>
        <MemorySlider value={app.settings.memoryMb} onchange={(memoryMb) => updateSettings({ memoryMb })} />
      </div>

      <div class="row">
        <div class="text">
          <span class="label"><Icon name="eye" size={16} />{t("settings.hide")}</span>
          <span class="hint">{t("settings.hideHint")}</span>
        </div>
        <button
          class="switch"
          role="switch"
          aria-checked={app.settings.hideOnLaunch}
          aria-label={t("settings.hide")}
          onclick={() => updateSettings({ hideOnLaunch: !app.settings!.hideOnLaunch })}
        ></button>
      </div>

      <div class="row">
        <div class="text">
          <span class="label"><Icon name="globe" size={16} />{t("settings.language")}</span>
        </div>
        <div class="segmented">
          <button class:on={(app.settings.language ?? "") !== "en"} onclick={() => updateSettings({ language: "tr" })}>Türkçe</button>
          <button class:on={app.settings.language === "en"} onclick={() => updateSettings({ language: "en" })}>English</button>
        </div>
      </div>
    </div>

    <p class="about">
      Vega 0.2.0 · {t("settings.about")}
      <button class="link" onclick={openRepo}><Icon name="github" size={14} />GitHub</button>
    </p>
  </section>
{/if}

<style>
  .settings {
    height: 100%;
    overflow-y: auto;
    padding: 46px 36px 40px;
  }

  h1 {
    margin-bottom: 20px;
    font-size: 28px;
    font-weight: 650;
    letter-spacing: -0.02em;
  }

  .card {
    max-width: 760px;
    border-radius: 16px;
    background: rgb(18 16 42 / 0.6);
    border: 1px solid var(--border);
    backdrop-filter: blur(10px);
  }

  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 24px;
    padding: 18px 20px;
  }

  .row + .row {
    border-top: 1px solid var(--border);
  }

  .text {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .label {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 500;
  }

  .label :global(svg) {
    color: var(--text-3);
  }

  .hint {
    font-size: 12px;
    color: var(--text-3);
  }

  .hint.invalid {
    color: var(--danger);
  }

  .input {
    width: 220px;
  }

  .input.bad {
    border-color: rgb(240 149 149 / 0.6);
  }

  .switch {
    position: relative;
    width: 44px;
    height: 26px;
    flex-shrink: 0;
    border-radius: 99px;
    background: rgb(255 255 255 / 0.12);
    transition: background 0.2s;
  }

  .switch::after {
    content: "";
    position: absolute;
    top: 3px;
    left: 3px;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: #fff;
    transition: transform 0.2s var(--ease);
  }

  .switch[aria-checked="true"] {
    background: var(--accent);
  }

  .switch[aria-checked="true"]::after {
    transform: translateX(18px);
  }

  .segmented {
    display: flex;
    gap: 4px;
    padding: 4px;
    border-radius: var(--radius-sm);
    background: var(--surface);
    border: 1px solid var(--border);
  }

  .segmented button {
    height: 30px;
    padding: 0 14px;
    border-radius: 7px;
    color: var(--text-2);
    font-weight: 500;
  }

  .segmented button.on {
    background: var(--accent);
    color: #fff;
  }

  .about {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 16px;
    font-size: 12px;
    color: var(--text-3);
  }

  .link {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    color: var(--text-2);
  }

  .link:hover {
    color: var(--text);
  }
</style>
