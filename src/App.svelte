<script lang="ts">
  import AccountsDialog from "./lib/components/AccountsDialog.svelte";
  import Backdrop from "./lib/components/Backdrop.svelte";
  import Console from "./lib/components/Console.svelte";
  import CreateDialog from "./lib/components/CreateDialog.svelte";
  import ModBrowser from "./lib/components/ModBrowser.svelte";
  import Sidebar from "./lib/components/Sidebar.svelte";
  import TitleBar from "./lib/components/TitleBar.svelte";
  import Toast from "./lib/components/Toast.svelte";
  import { locale } from "./lib/i18n.svelte";
  import { app, init, selectedProfile } from "./lib/state.svelte";
  import Profile from "./lib/views/Profile.svelte";
  import Settings from "./lib/views/Settings.svelte";

  init();

  const profile = $derived(selectedProfile());

  $effect(() => {
    document.documentElement.lang = locale.lang;
  });
</script>

<div class="app">
  <Backdrop />
  <TitleBar />
  <Sidebar />

  <main>
    {#if app.view === "settings"}
      <Settings />
    {:else}
      <Profile />
    {/if}
    {#if app.modBrowserOpen && profile && app.view === "profile"}
      <ModBrowser {profile} />
    {/if}
    {#if app.consoleOpen}
      <Console />
    {/if}
  </main>

  {#if app.createOpen}<CreateDialog />{/if}
  {#if app.accountsOpen}<AccountsDialog />{/if}
  <Toast />
</div>

<style>
  .app {
    position: relative;
    height: 100%;
    display: flex;
    background: var(--bg);
    overflow: hidden;
  }

  main {
    position: relative;
    flex: 1;
    min-width: 0;
  }
</style>
