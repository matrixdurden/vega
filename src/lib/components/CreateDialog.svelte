<script lang="ts">
  import { api, on, type ClientInfo, type Loader } from "../api";
  import { t } from "../i18n.svelte";
  import { pixelIconKeys } from "../pixelIcons";
  import { addProfile, app, attempt } from "../state.svelte";
  import Icon from "./Icon.svelte";
  import Modal from "./Modal.svelte";
  import ProfileFields from "./ProfileFields.svelte";
  import ProfileIcon from "./ProfileIcon.svelte";

  const CLIENT_ICONS: Record<string, string> = { meteor: "sword", "fabulously-optimized": "diamond" };

  let mode = $state<"blank" | "client">("blank");
  let busy = $state(false);

  // Blank profile
  let icon = $state(pixelIconKeys[app.profiles.length % pixelIconKeys.length]);
  let name = $state("");
  let loader = $state<Loader>("fabric");
  let version = $state("");

  // Client profile
  let clients = $state<ClientInfo[] | null>(null);
  let clientId = $state("");
  let clientVersion = $state("");
  let progress = $state(0);
  const client = $derived(clients?.find((c) => c.id === clientId));

  $effect(() => {
    if (mode !== "client" || clients) return;
    attempt(() => api.listClients()).then((list) => {
      clients = list ?? [];
      if (!clientId && clients[0]) pick(clients[0]);
    });
  });

  $effect(() => {
    const off = on("install-progress", (e) => (progress = e.total ? Math.round((e.done / e.total) * 100) : 0));
    return () => void off.then((unlisten) => unlisten());
  });

  function pick(c: ClientInfo) {
    clientId = c.id;
    clientVersion = c.versions[0] ?? "";
  }

  const close = () => !busy && (app.createOpen = false);

  async function create() {
    if (busy) return;
    busy = true;
    progress = 0;
    const profile = await attempt(() =>
      mode === "blank"
        ? api.createProfile({ name: name.trim() || `${loader === "fabric" ? "Fabric" : "Vanilla"} ${version}`, icon, gameVersion: version, loader, memoryMb: null })
        : api.createClientProfile(clientId, clientVersion, CLIENT_ICONS[clientId] ?? "sword"),
    );
    busy = false;
    if (profile) {
      app.createOpen = false;
      await addProfile(profile);
    }
  }
</script>

<Modal title={t("create.title")} width={560} onclose={close}>
  <div class="segmented">
    <button class:on={mode === "blank"} onclick={() => (mode = "blank")} disabled={busy}>{t("create.blank")}</button>
    <button class:on={mode === "client"} onclick={() => (mode = "client")} disabled={busy}>{t("create.client")}</button>
  </div>

  {#if mode === "blank"}
    <ProfileFields bind:icon bind:name bind:loader bind:version placeholder={`${loader === "fabric" ? "Fabric" : "Vanilla"} ${version}`} />
  {:else if !clients}
    <div class="loading"><Icon name="loader" class="spin" /></div>
  {:else}
    <div class="clients">
      {#each clients as c (c.id)}
        <button class="client" class:on={c.id === clientId} disabled={!c.versions.length || busy} onclick={() => pick(c)}>
          <ProfileIcon icon={CLIENT_ICONS[c.id] ?? "sword"} size={40} />
          <span class="text">
            <b>{c.name}</b>
            <span class="tag" class:warn={c.kind === "utility"}>
              {c.kind === "utility" ? t("create.utility") : t("create.performance")}
            </span>
          </span>
          {#if !c.versions.length}<span class="muted">{t("create.noVersions")}</span>{/if}
        </button>
      {/each}
    </div>
    {#if client}
      <label class="field">
        <span class="label">{t("form.version")}</span>
        <select class="input" bind:value={clientVersion} disabled={busy}>
          {#each client.versions as v (v)}<option value={v}>{v}</option>{/each}
        </select>
      </label>
      {#if client.kind === "utility"}
        <p class="warning"><Icon name="alert" size={16} />{t("create.utilityWarning")}</p>
      {/if}
    {/if}
  {/if}

  <div class="actions">
    {#if busy && mode === "client"}
      <div class="bar"><div style:width="{progress}%"></div></div>
    {/if}
    <button class="btn" onclick={close} disabled={busy}>{t("create.cancel")}</button>
    <button class="btn green" onclick={create} disabled={busy || (mode === "blank" ? !version : !clientVersion)}>
      {#if busy}<Icon name="loader" size={16} class="spin" />{/if}
      {mode === "blank" ? t("create.button") : busy ? t("create.installing") : t("create.install")}
    </button>
  </div>
</Modal>

<style>
  .loading {
    display: grid;
    place-items: center;
    padding: 30px;
    color: var(--text-3);
  }

  .clients {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .client {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 10px;
    border-radius: 6px;
    background: #120f26;
    border: 2px solid transparent;
    text-align: left;
  }

  .client:hover:not(:disabled) {
    border-color: var(--border-strong);
  }

  .client.on {
    border-color: var(--accent);
  }

  .text {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
  }

  .text b {
    font-family: var(--pixel);
    font-size: 15px;
  }

  .tag {
    padding: 0 6px;
    border-radius: 3px;
    background: rgb(111 211 107 / 0.15);
    color: var(--success);
    font-size: 11px;
  }

  .tag.warn {
    background: rgb(250 199 117 / 0.15);
    color: var(--warning);
  }

  .muted {
    font-size: 12px;
    color: var(--text-3);
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .label {
    font-size: 12px;
    color: var(--text-3);
  }

  .warning {
    display: flex;
    gap: 8px;
    padding: 10px 12px;
    border-radius: 6px;
    background: rgb(250 199 117 / 0.08);
    color: var(--warning);
    font-size: 12.5px;
  }

  .warning :global(svg) {
    flex-shrink: 0;
    margin-top: 2px;
  }

  .actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
  }

  .bar {
    flex: 1;
    height: 8px;
    border-radius: 2px;
    background: #120f26;
    overflow: hidden;
  }

  .bar div {
    height: 100%;
    background: var(--green);
    transition: width 0.2s;
  }
</style>
