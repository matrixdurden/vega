<script lang="ts">
  import { filled, icons, type IconName } from "../icons";

  let { name, size = 18, class: className = "" }: { name: IconName; size?: number; class?: string } = $props();
</script>

<svg
  class={className}
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill={filled.has(name) ? "currentColor" : "none"}
  stroke="currentColor"
  stroke-width="1.8"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  {#each icons[name] as d (d)}
    <path {d} />
  {/each}
</svg>
