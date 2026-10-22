<script lang="ts">
  import { profileIcon } from "../profileIcons";
  import Icon from "./Icon.svelte";

  let { icon, size = 36 }: { icon: string; size?: number } = $props();

  const look = $derived(profileIcon(icon));
</script>

<span
  class="tile"
  style:width="{size}px"
  style:height="{size}px"
  style:border-radius="{Math.round(size * 0.28)}px"
  style:background={look.bg}
  style:color={look.fg}
>
  <Icon name={look.icon} size={Math.round(size * 0.52)} />
</span>

<style>
  .tile {
    display: grid;
    place-items: center;
    flex-shrink: 0;
    box-shadow: inset 0 0 0 1px rgb(255 255 255 / 0.06);
  }
</style>
