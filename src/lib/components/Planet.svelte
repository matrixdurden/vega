<script lang="ts">
  // Decorative ringed planet for the home screen; the ring's front half is
  // drawn after the body so the planet sits inside it.
</script>

<svg class="planet" viewBox="0 0 440 440" aria-hidden="true">
  <defs>
    <radialGradient id="planet-body" cx="34%" cy="28%" r="80%">
      <stop offset="0" stop-color="#a49df2" />
      <stop offset="0.5" stop-color="#5b52c4" />
      <stop offset="1" stop-color="#1c1852" />
    </radialGradient>
    <linearGradient id="planet-ring" x1="0" x2="1">
      <stop offset="0" stop-color="#cecbf6" stop-opacity="0.1" />
      <stop offset="0.5" stop-color="#cecbf6" stop-opacity="0.75" />
      <stop offset="1" stop-color="#cecbf6" stop-opacity="0.1" />
    </linearGradient>
    <clipPath id="ring-front">
      <rect x="0" y="220" width="440" height="220" />
    </clipPath>
  </defs>
  <g transform="rotate(-16 220 220)">
    <ellipse cx="220" cy="220" rx="200" ry="42" fill="none" stroke="url(#planet-ring)" stroke-width="7" />
  </g>
  <circle cx="220" cy="220" r="118" fill="url(#planet-body)" />
  <circle cx="180" cy="170" r="22" fill="#fff" opacity="0.05" />
  <circle cx="262" cy="262" r="14" fill="#fff" opacity="0.04" />
  <circle cx="246" cy="150" r="8" fill="#fff" opacity="0.05" />
  <g transform="rotate(-16 220 220)">
    <ellipse cx="220" cy="220" rx="200" ry="42" fill="none" stroke="url(#planet-ring)" stroke-width="7" clip-path="url(#ring-front)" />
  </g>
</svg>

<style>
  .planet {
    position: absolute;
    right: -120px;
    bottom: -170px;
    width: 440px;
    height: 440px;
    opacity: 0.55;
    pointer-events: none;
    filter: drop-shadow(0 0 70px rgb(127 119 221 / 0.35));
    animation: float 9s ease-in-out infinite;
  }

  @keyframes float {
    50% {
      transform: translateY(10px);
    }
  }
</style>
