<script lang="ts">
  // The banner behind a profile's header: a still pixel night with square
  // stars, a square moon and blocky hills. Drawn once, never animated.
  const W = 1200;
  const H = 200;
  const BLOCK = 20;

  let seed = 11;
  const random = () => (seed = (seed * 16807) % 2147483647) / 2147483647;

  const stars = Array.from({ length: 70 }, () => ({
    x: Math.round(random() * W),
    y: Math.round(random() * H * 0.7),
    s: random() < 0.2 ? 4 : 2,
    o: (0.3 + random() * 0.6).toFixed(2),
  }));

  // Column heights in blocks, walking up and down.
  function hills(base: number, spread: number) {
    let h = base;
    return Array.from({ length: W / BLOCK + 1 }, () => {
      h = Math.max(base - spread, Math.min(base + spread, h + Math.round(random() * 2 - 1)));
      return h;
    });
  }
  const far = hills(4, 2);
  const near = hills(2, 1);
</script>

<div class="banner" aria-hidden="true">
  <svg viewBox="0 0 {W} {H}" preserveAspectRatio="xMaxYMax slice" shape-rendering="crispEdges">
    <defs>
      <linearGradient id="sky" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0" stop-color="#101026" />
        <stop offset="1" stop-color="#262150" />
      </linearGradient>
    </defs>
    <rect width={W} height={H} fill="url(#sky)" />
    {#each stars as star, i (i)}
      <rect x={star.x} y={star.y} width={star.s} height={star.s} fill="#fff" opacity={star.o} />
    {/each}
    <g transform="translate(700 12)">
      <rect width="48" height="48" fill="#e8ecf2" />
      <rect x="8" y="8" width="8" height="8" fill="#c7cfdb" />
      <rect x="28" y="20" width="12" height="8" fill="#c7cfdb" />
      <rect x="12" y="32" width="8" height="8" fill="#c7cfdb" />
    </g>
    {#each far as h, i (i)}
      <rect x={i * BLOCK} y={H - h * BLOCK} width={BLOCK} height={h * BLOCK} fill="#2e2864" />
    {/each}
    {#each near as h, i (i)}
      <rect x={i * BLOCK} y={H - h * BLOCK} width={BLOCK} height={h * BLOCK} fill="#3d2c22" />
      <rect x={i * BLOCK} y={H - h * BLOCK} width={BLOCK} height="6" fill="#4a8a32" />
    {/each}
  </svg>
  <div class="fade"></div>
</div>

<style>
  .banner {
    position: absolute;
    inset: 0 0 auto 0;
    height: 168px;
    overflow: hidden;
    pointer-events: none;
  }

  svg {
    width: 100%;
    height: 100%;
    display: block;
  }

  /* Darken toward the content so the header text always reads clearly. */
  .fade {
    position: absolute;
    inset: 0;
    background: linear-gradient(to bottom, rgb(23 23 27 / 0.15), rgb(23 23 27 / 0.55) 55%, var(--bg));
  }
</style>
