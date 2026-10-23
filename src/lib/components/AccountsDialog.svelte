<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { api, isTauri, type DeviceCode } from "../api";
  import { t } from "../i18n.svelte";
  import { app, attempt } from "../state.svelte";
  import Icon from "./Icon.svelte";
  import Modal from "./Modal.svelte";

  const USERNAME = /^\w{3,16}$/;

  // list → choose a kind → fill in that kind
  let step = $state<"list" | "choose" | "offline" | "microsoft">(app.accounts.length ? "list" : "choose");
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
    if (await update(() => api.addOfflineAccount(username.trim()))) {
      username = "";
      step = "list";
    }
  }

  async function signIn() {
    step = "microsoft";
    code = (await attempt(() => api.startMicrosoftLogin())) ?? null;
    if (!code) return void (step = "choose");
    if (await update(() => api.finishMicrosoftLogin())) step = "list";
    code = null;
  }

  async function copyAndOpen() {
    if (!code) return;
    await navigator.clipboard.writeText(code.userCode).catch(() => {});
    if (isTauri) openUrl(code.verificationUri);
    else window.open(code.verificationUri, "_blank");
  }

  function back() {
    if (code) api.cancelMicrosoftLogin();
    step = app.accounts.length ? "list" : "choose";
  }

  function close() {
    if (code) api.cancelMicrosoftLogin();
    app.accountsOpen = false;
  }
</script>

<Modal title={t("accounts.title")} width={440} onclose={close}>
  {#if step === "list"}
    <div class="list">
      {#each app.accounts as account (account.id)}
        <div class="row" class:active={account.active}>
          <span class="avatar" class:ms={account.kind === "microsoft"}>{account.username.slice(0, 1).toUpperCase()}</span>
          <span class="text">
            <b class="mc">{account.username}</b>
            <span class="hint">{account.kind === "microsoft" ? t("accounts.microsoft") : t("accounts.offline")}</span>
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
    <button class="btn green wide" onclick={() => (step = "choose")}><Icon name="plus" size={16} />{t("accounts.add")}</button>
  {:else if step === "choose"}
    <p class="hint">{app.accounts.length ? t("accounts.chooseType") : t("accounts.none")}</p>
    <div class="kinds">
      <button class="kind" onclick={() => (step = "offline")}>
        <span class="avatar">S</span>
        <span class="text">
          <b class="mc">{t("accounts.offline")}</b>
          <span class="hint">{t("accounts.offlineDesc")}</span>
        </span>
      </button>
      <button class="kind" disabled={!microsoftReady} onclick={signIn}>
        <svg viewBox="0 0 16 16" width="30" height="30" aria-hidden="true">
          <rect width="7" height="7" fill="#f25022" /><rect x="9" width="7" height="7" fill="#7fba00" />
          <rect y="9" width="7" height="7" fill="#00a4ef" /><rect x="9" y="9" width="7" height="7" fill="#ffb900" />
        </svg>
        <span class="text">
          <b class="mc">{t("accounts.microsoft")}</b>
          <span class="hint">{microsoftReady ? t("accounts.microsoftDesc") : t("accounts.microsoftOff")}</span>
        </span>
      </button>
    </div>
    {#if app.accounts.length}<button class="btn" onclick={back}>{t("accounts.back")}</button>{/if}
  {:else if step === "offline"}
    <form class="offline" onsubmit={addOffline}>
      <label class="field">
        <span class="label">{t("accounts.username")}</span>
        <!-- svelte-ignore a11y_autofocus -->
        <input class="input" bind:value={username} maxlength="16" spellcheck="false" autofocus />
        {#if username && !valid}<span class="error">{t("accounts.usernameInvalid")}</span>{/if}
      </label>
      <div class="actions">
        <button type="button" class="btn" onclick={back}>{t("accounts.back")}</button>
        <button class="btn green" disabled={!valid}>{t("accounts.addButton")}</button>
      </div>
    </form>
  {:else}
    <div class="code">
      {#if code}
        <span class="hint">{t("accounts.codeHint")}</span>
        <b class="mc">{code.userCode}</b>
        <button class="btn green" onclick={copyAndOpen}><Icon name="external" size={16} />{t("accounts.copyOpen")}</button>
      {/if}
      <span class="hint waiting"><Icon name="loader" size={14} class="spin" />{t("accounts.waiting")}</span>
      <button class="btn" onclick={back}>{t("accounts.cancel")}</button>
    </div>
  {/if}
</Modal>

<style>
  .list,
  .kinds {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .row,
  .kind {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    border-radius: 4px;
    background: var(--panel-2);
    border: 1px solid var(--border);
    text-align: left;
  }

  .row.active {
    border-color: var(--green);
  }

  .kind:hover:not(:disabled) {
    border-color: var(--border-strong);
    background: #32323a;
  }

  .avatar {
    width: 32px;
    height: 32px;
    display: grid;
    place-items: center;
    flex-shrink: 0;
    border-radius: 4px;
    background: #5a3b1f;
    color: #ffd59a;
    font-family: var(--mc);
    font-weight: 700;
  }

  .avatar.ms {
    background: #1d4d23;
    color: var(--green-text);
  }

  .text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .text b {
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: 14px;
  }

  .badge {
    padding: 2px 8px;
    border-radius: 3px;
    background: var(--select);
    color: var(--green-text);
    font-family: var(--mc);
    font-size: 12px;
  }

  .btn.small {
    height: 28px;
    padding: 0 10px;
    font-size: 12px;
  }

  .btn.wide {
    width: 100%;
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
    flex-direction: column;
    gap: 14px;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .error {
    font-size: 12px;
    color: var(--danger);
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  .code {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 8px 0;
  }

  .code b {
    font-size: 30px;
    letter-spacing: 0.1em;
    user-select: text;
  }

  .waiting {
    display: flex;
    align-items: center;
    gap: 6px;
  }
</style>
