<script lang="ts">
  // A still night scene: square stars, a square moon and blocky hills.
  // It is drawn once as SVG and never animates, so it costs nothing at idle.
  const W = 1200;
  const H = 720;
  const BLOCK = 24;

  let seed = 11;
  const random = () => (seed = (seed * 16807) % 2147483647) / 2147483647;

  const stars = Array.from({ length: 90 }, () => ({
    x: Math.round(random() * W),
    y: Math.round(random() * H * 0.62),
    s: random() < 0.2 ? 4 : 2,
    o: (0.25 + random() * 0.6).toFixed(2),
  }));

  // Column heights in blocks for two hill layers, walking up and down.
  function hills(base: number, spread: number) {
    let h = base;
    return Array.from({ length: W / BLOCK + 1 }, () => {
      h = Math.max(base - spread, Math.min(base + spread, h + Math.round(random() * 2 - 1)));
      return h;
    });
  }
  const far = hills(8, 3);
  const near = hills(5, 2);
</script>

<svg class="backdrop" viewBox="0 0 {W} {H}" preserveAspectRatio="xMidYMax slice" aria-hidden="true" shape-rendering="crispEdges">
  <defs>
    <linearGradient id="sky" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#0a0918" />
      <stop offset="1" stop-color="#1a1640" />
    </linearGradient>
  </defs>
  <rect width={W} height={H} fill="url(#sky)" />
  {#each stars as star, i (i)}
    <rect x={star.x} y={star.y} width={star.s} height={star.s} fill="#fff" opacity={star.o} />
  {/each}

  <!-- Rising behind the far hills, below anything the UI draws on top. -->
  <g transform="translate(880 395)">
    <rect width="72" height="72" fill="#e8ecf2" />
    <rect x="12" y="12" width="12" height="12" fill="#c7cfdb" />
    <rect x="42" y="30" width="18" height="12" fill="#c7cfdb" />
    <rect x="18" y="48" width="12" height="12" fill="#c7cfdb" />
    <rect x="-6" y="-6" width="84" height="84" fill="none" stroke="#e8ecf2" stroke-opacity="0.12" stroke-width="6" />
  </g>

  {#each far as h, i (i)}
    <rect x={i * BLOCK} y={H - h * BLOCK} width={BLOCK} height={h * BLOCK} fill="#231e4e" />
  {/each}
  {#each near as h, i (i)}
    <rect x={i * BLOCK} y={H - h * BLOCK} width={BLOCK} height={h * BLOCK} fill="#15112d" />
    <rect x={i * BLOCK} y={H - h * BLOCK} width={BLOCK} height="8" fill="#2c5a2a" />
  {/each}
</svg>

<style>
  .backdrop {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
  }
</style>
