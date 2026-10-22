<script lang="ts">
  import Console from "./lib/components/Console.svelte";
  import ModBrowser from "./lib/components/ModBrowser.svelte";
  import ProfileDialog from "./lib/components/ProfileDialog.svelte";
  import Sidebar from "./lib/components/Sidebar.svelte";
  import Starfield from "./lib/components/Starfield.svelte";
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
  <Starfield paused={app.launch.phase === "running"} />
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

  {#if app.dialog}
    {#key app.dialog}
      <ProfileDialog mode={app.dialog.mode} id={app.dialog.mode === "edit" ? app.dialog.id : undefined} />
    {/key}
  {/if}
  <Toast />
</div>

<style>
  .app {
    position: relative;
    height: 100%;
    display: flex;
    background:
      radial-gradient(1200px 600px at 85% 0%, rgb(83 74 183 / 0.22), transparent 60%),
      radial-gradient(900px 500px at 0% 100%, rgb(24 95 165 / 0.14), transparent 60%),
      var(--bg);
    overflow: hidden;
  }

  main {
    position: relative;
    flex: 1;
    min-width: 0;
  }
</style>
