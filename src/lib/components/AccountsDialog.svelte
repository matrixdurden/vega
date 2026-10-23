<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { api, isTauri, type DeviceCode } from "../api";
  import { t } from "../i18n.svelte";
  import { app, attempt } from "../state.svelte";
  import Icon from "./Icon.svelte";
  import Modal from "./Modal.svelte";

  const USERNAME = /^\w{3,16}$/;

  let username = $state("");
  let microsoftReady = $state(false);
  let code = $state<DeviceCode | null>(null);
  const valid = $derived(USERNAME.test(username.trim()));

  $effect(() => {
    api.microsoftAvailable().then((ready) => (microsoftReady = ready)).catch(() => {});
  });

  async function update(action: () => ReturnType<typeof api.listAccounts>) {
    const accounts = await attempt(action);
    if (accounts) app.accounts = accounts;
    return accounts;
  }

  async function addOffline(event: SubmitEvent) {
    event.preventDefault();
    if (!valid) return;
    if (await update(() => api.addOfflineAccount(username.trim()))) username = "";
  }

  async function signIn() {
    code = (await attempt(() => api.startMicrosoftLogin())) ?? null;
    if (!code) return;
    await update(() => api.finishMicrosoftLogin());
    code = null;
  }

  async function copyAndOpen() {
    if (!code) return;
    await navigator.clipboard.writeText(code.userCode).catch(() => {});
    if (isTauri) openUrl(code.verificationUri);
    else window.open(code.verificationUri, "_blank");
  }

  function close() {
    if (code) api.cancelMicrosoftLogin();
    app.accountsOpen = false;
  }
</script>

<Modal title={t("accounts.title")} width={460} onclose={close}>
  {#if app.accounts.length}
    <div class="list">
      {#each app.accounts as account (account.id)}
        <div class="row" class:active={account.active}>
          <span class="avatar" class:ms={account.kind === "microsoft"}>{account.username.slice(0, 1).toUpperCase()}</span>
          <span class="text">
            <b>{account.username}</b>
            <span class="meta">{account.kind === "microsoft" ? t("accounts.microsoft") : t("accounts.offline")}</span>
          </span>
          {#if account.active}
            <span class="badge">{t("accounts.active")}</span>
          {:else}
            <button class="btn small" onclick={() => update(() => api.setActiveAccount(account.id))}>{t("accounts.use")}</button>
          {/if}
          <button class="remove" aria-label={t("accounts.remove")} title={t("accounts.remove")} onclick={() => update(() => api.removeAccount(account.id))}>
            <Icon name="trash" size={16} />
          </button>
        </div>
      {/each}
    </div>
  {:else}
    <p class="hint">{t("accounts.none")}</p>
  {/if}

  <form class="offline" onsubmit={addOffline}>
    <input class="input" bind:value={username} placeholder={t("accounts.offlinePlaceholder")} maxlength="16" spellcheck="false" />
    <button class="btn" disabled={!valid}><Icon name="plus" size={16} />{t("accounts.addOffline")}</button>
  </form>
  {#if username && !valid}<span class="error">{t("accounts.usernameInvalid")}</span>{/if}

  {#if code}
    <div class="code">
      <span class="hint">{t("accounts.codeHint")}</span>
      <b class="pixel">{code.userCode}</b>
      <div class="code-actions">
        <button class="btn green" onclick={copyAndOpen}><Icon name="external" size={16} />{t("accounts.copyOpen")}</button>
        <button class="btn" onclick={() => api.cancelMicrosoftLogin()}>{t("accounts.cancel")}</button>
      </div>
      <span class="waiting"><Icon name="loader" size={14} class="spin" />{t("accounts.waiting")}</span>
    </div>
  {:else}
    <button class="btn microsoft" disabled={!microsoftReady} onclick={signIn}>
      <svg viewBox="0 0 16 16" width="14" height="14" aria-hidden="true">
        <rect width="7" height="7" fill="#f25022" /><rect x="9" width="7" height="7" fill="#7fba00" />
        <rect y="9" width="7" height="7" fill="#00a4ef" /><rect x="9" y="9" width="7" height="7" fill="#ffb900" />
      </svg>
      {t("accounts.addMicrosoft")}
    </button>
    {#if !microsoftReady}<p class="hint">{t("accounts.microsoftOff")}</p>{/if}
  {/if}
</Modal>

<style>
  .list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    border-radius: 6px;
    background: #120f26;
    border: 2px solid transparent;
  }

  .row.active {
    border-color: rgb(111 211 107 / 0.4);
  }

  .avatar {
    width: 32px;
    height: 32px;
    display: grid;
    place-items: center;
    border-radius: 4px;
    background: #3a2a1a;
    color: #e8b06a;
    font-family: var(--pixel);
  }

  .avatar.ms {
    background: #0d3a24;
    color: var(--success);
  }

  .text {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .text b {
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .meta {
    font-size: 12px;
    color: var(--text-3);
  }

  .badge {
    padding: 2px 8px;
    border-radius: 3px;
    background: rgb(111 211 107 / 0.15);
    color: var(--success);
    font-size: 12px;
    font-weight: 600;
  }

  .btn.small {
    height: 28px;
    padding: 0 10px;
    font-size: 12px;
  }

  .remove {
    width: 30px;
    height: 30px;
    display: grid;
    place-items: center;
    border-radius: 4px;
    color: var(--text-3);
  }

  .remove:hover {
    background: var(--danger-soft);
    color: var(--danger);
  }

  .offline {
    display: flex;
    gap: 8px;
  }

  .offline .btn {
    flex-shrink: 0;
    height: 38px;
  }

  .error {
    margin-top: -8px;
    font-size: 12px;
    color: var(--danger);
  }

  .microsoft {
    height: 40px;
  }

  .hint {
    font-size: 12.5px;
    color: var(--text-3);
  }

  .code {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    padding: 16px;
    border-radius: 6px;
    background: #120f26;
  }

  .code b {
    font-size: 30px;
    letter-spacing: 0.12em;
    user-select: text;
  }

  .code-actions {
    display: flex;
    gap: 8px;
  }

  .waiting {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--text-3);
  }
</style>
