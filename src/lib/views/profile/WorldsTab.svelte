<script lang="ts">
  import { api, type Profile } from "../../api";
  import Icon from "../../components/Icon.svelte";
  import { relativeTime, t } from "../../i18n.svelte";
  import { app, attempt } from "../../state.svelte";

  let { profile }: { profile: Profile } = $props();

  const openSaves = () => attempt(() => api.openProfileFolder(profile.id, "saves"));
</script>

{#if app.worlds.length === 0}
  <div class="empty">
    <Icon name="map" size={30} />
    <h2>{t("worlds.none")}</h2>
    <p>{t("worlds.noneBody")}</p>
  </div>
{:else}
  <div class="head">
    <button class="btn" onclick={openSaves}><Icon name="folder" size={16} />{t("worlds.openFolder")}</button>
  </div>
  <div class="list">
    {#each app.worlds as world (world.name)}
      <article class="row">
        <span class="glyph"><Icon name="map" size={18} /></span>
        <div class="info">
          <b>{world.name}</b>
          {#if world.lastPlayed}<span class="meta">{t("profile.lastPlayed", { when: relativeTime(world.lastPlayed) })}</span>{/if}
        </div>
      </article>
    {/each}
  </div>
{/if}

<style>
  .head {
    display: flex;
    justify-content: flex-end;
    margin-bottom: 10px;
  }

  .list {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: 8px;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px;
    border-radius: 12px;
    background: rgb(18 16 42 / 0.55);
    border: 1px solid var(--border);
    backdrop-filter: blur(8px);
  }

  .glyph {
    width: 38px;
    height: 38px;
    display: grid;
    place-items: center;
    border-radius: 10px;
    background: #173404;
    color: #97c459;
  }

  .info {
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .info b {
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .meta {
    font-size: 12px;
    color: var(--text-3);
  }

  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 56px 20px;
    text-align: center;
    color: var(--text-3);
  }

  .empty h2 {
    color: var(--text);
    font-size: 17px;
    font-weight: 600;
  }
</style>
