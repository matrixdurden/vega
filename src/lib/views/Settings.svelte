<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { isTauri } from "../api";
  import Icon from "../components/Icon.svelte";
  import MemorySlider from "../components/MemorySlider.svelte";
  import { t } from "../i18n.svelte";
  import { app, updateSettings } from "../state.svelte";

  const REPO = "https://github.com/matrixdurden/vega";
  const openRepo = () => (isTauri ? openUrl(REPO) : window.open(REPO, "_blank"));
</script>

{#if app.settings}
  <section class="settings">
    <h1>{t("settings.title")}</h1>

    <div class="card">
      <div class="row">
        <div class="text">
          <span class="label">{t("settings.memory")}</span>
          <span class="hint">{t("settings.memoryHint")}</span>
        </div>
        <MemorySlider value={app.settings.memoryMb} onchange={(memoryMb) => updateSettings({ memoryMb })} />
      </div>

      <div class="row">
        <div class="text">
          <span class="label">{t("settings.hide")}</span>
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
        <span class="label">{t("settings.language")}</span>
        <div class="segmented">
          <button class:on={(app.settings.language ?? "") !== "en"} onclick={() => updateSettings({ language: "tr" })}>Türkçe</button>
          <button class:on={app.settings.language === "en"} onclick={() => updateSettings({ language: "en" })}>English</button>
        </div>
      </div>
    </div>

    <p class="about">
      Vega 0.5.0 · {t("settings.about")}
      <button class="link" onclick={openRepo}><Icon name="github" size={14} />GitHub</button>
    </p>
  </section>
{/if}

<style>
  .settings {
    height: 100%;
    overflow-y: auto;
    padding: 56px clamp(20px, 3vw, 48px) 40px;
  }

  h1 {
    margin-bottom: 18px;
    font-size: 28px;
  }

  .card {
    border-radius: 8px;
    background: var(--panel);
    border: 1px solid var(--border);
  }

  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 24px;
    padding: 16px 18px;
  }

  .row + .row {
    border-top: 1px solid var(--border);
  }

  .text {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .label {
    font-weight: 600;
  }

  .hint {
    font-size: 12px;
    color: var(--text-3);
  }

  .segmented {
    width: 220px;
  }

  .about {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 14px;
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
