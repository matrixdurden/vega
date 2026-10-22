<script lang="ts">
  // Twinkling stars on a canvas. Redraws ~20 times a second, which is plenty
  // for a slow twinkle and keeps the launcher close to idle on the GPU.
  let { paused = false }: { paused?: boolean } = $props();

  let canvas: HTMLCanvasElement;

  type Star = { x: number; y: number; r: number; alpha: number; speed: number; phase: number; tinted: boolean };

  $effect(() => {
    const ctx = canvas.getContext("2d")!;
    const still = matchMedia("(prefers-reduced-motion: reduce)").matches;
    let stars: Star[] = [];
    let width = 0;
    let height = 0;
    let frame = 0;
    let last = 0;

    const draw = (time: number) => {
      ctx.clearRect(0, 0, width, height);
      for (const star of stars) {
        const alpha = star.alpha * (0.65 + 0.35 * Math.sin(time * star.speed + star.phase));
        ctx.fillStyle = star.tinted ? `rgb(175 169 236 / ${alpha})` : `rgb(255 255 255 / ${alpha})`;
        ctx.beginPath();
        ctx.arc(star.x, star.y, star.r, 0, Math.PI * 2);
        ctx.fill();
      }
    };

    const resize = () => {
      const dpr = Math.min(devicePixelRatio || 1, 2);
      width = canvas.clientWidth;
      height = canvas.clientHeight;
      canvas.width = width * dpr;
      canvas.height = height * dpr;
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      stars = Array.from({ length: Math.round((width * height) / 4500) }, () => ({
        x: Math.random() * width,
        y: Math.random() * height,
        r: Math.random() * 1.1 + 0.25,
        alpha: Math.random() * 0.6 + 0.2,
        speed: Math.random() * 0.0016 + 0.0004,
        phase: Math.random() * Math.PI * 2,
        tinted: Math.random() < 0.15,
      }));
      draw(performance.now());
    };

    const loop = (time: number) => {
      frame = requestAnimationFrame(loop);
      if (paused || time - last < 50) return;
      last = time;
      draw(time);
    };

    const observer = new ResizeObserver(resize);
    observer.observe(canvas);
    if (!still) frame = requestAnimationFrame(loop);

    return () => {
      cancelAnimationFrame(frame);
      observer.disconnect();
    };
  });
</script>

<canvas bind:this={canvas}></canvas>

<style>
  canvas {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
  }
</style>
