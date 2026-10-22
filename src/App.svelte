<script lang="ts">
  import Console from "./lib/components/Console.svelte";
  import NewProfileDialog from "./lib/components/NewProfileDialog.svelte";
  import Sidebar from "./lib/components/Sidebar.svelte";
  import Starfield from "./lib/components/Starfield.svelte";
  import TitleBar from "./lib/components/TitleBar.svelte";
  import Toast from "./lib/components/Toast.svelte";
  import { locale } from "./lib/i18n.svelte";
  import { app, init } from "./lib/state.svelte";
  import Home from "./lib/views/Home.svelte";
  import Mods from "./lib/views/Mods.svelte";
  import Settings from "./lib/views/Settings.svelte";

  init();

  $effect(() => {
    document.documentElement.lang = locale.lang;
  });
</script>

<div class="app">
  <Starfield paused={app.launch.phase === "running"} />
  <TitleBar />
  <Sidebar />

  <main>
    {#if app.view === "home"}
      <Home />
    {:else if app.view === "mods"}
      <Mods />
    {:else}
      <Settings />
    {/if}
    {#if app.consoleOpen}
      <Console />
    {/if}
  </main>

  {#if app.newProfileOpen}
    <NewProfileDialog />
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
