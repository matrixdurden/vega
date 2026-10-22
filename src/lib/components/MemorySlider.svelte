<script lang="ts">
  // Memory in MB, shown and stepped in half gigabytes from 1 to 16 GB.
  let { value, onchange }: { value: number; onchange: (mb: number) => void } = $props();

  const gb = $derived(value / 1024);
</script>

<div class="memory">
  <input
    type="range"
    min="1"
    max="16"
    step="0.5"
    value={gb}
    style:--fill="{((gb - 1) / 15) * 100}%"
    oninput={(e) => onchange(Math.round(Number(e.currentTarget.value) * 1024))}
  />
  <b>{gb} GB</b>
</div>

<style>
  .memory {
    display: flex;
    align-items: center;
    gap: 14px;
  }

  b {
    width: 52px;
    text-align: right;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }

  input {
    width: 200px;
    height: 6px;
    appearance: none;
    border-radius: 99px;
    background: linear-gradient(to right, var(--accent) var(--fill), rgb(255 255 255 / 0.1) var(--fill));
    cursor: pointer;
  }

  input::-webkit-slider-thumb {
    appearance: none;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: #fff;
    border: 4px solid var(--accent);
  }
</style>
