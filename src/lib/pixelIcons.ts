// Original 16x16 pixel art for profile icons, in the spirit of Minecraft
// items. Each row is 16 characters; "." is transparent and every other
// character is a colour from the icon's palette.

type PixelIcon = { palette: Record<string, string>; rows: string[] };

export const pixelIcons = {
  grass: {
    palette: { g: "#7cb342", G: "#5b8c32", d: "#8b5a2b", D: "#6b4423", s: "#a87a46" },
    rows: [
      "gggGgggggGgggggG",
      "gGgggGggGgggGggg",
      "GgGgGggGgGGgggGg",
      "dGddGgdGdddGdGGd",
      "dddddGdddddddDdd",
      "dDdddddddsdddddd",
      "ddddsdddDddddDdd",
      "dddddddddddsdddd",
      "dDddddDddddddddd",
      "ddddddddddDddsdd",
      "ddsddddddddddddd",
      "dddddDddddsddDdd",
      "dddddddddddddddd",
      "dDddsdddDddddddd",
      "ddddddddddddDdsd",
      "dddDdddddddddddd",
    ],
  },
  diamond: {
    palette: { k: "#0e4f4f", c: "#4ee6d8", w: "#d8fffb", b: "#1fa394" },
    rows: [
      "",
      "",
      "....kkkkkkkk....",
      "...kwwccccbbk...",
      "..kwwcccccbbbk..",
      ".kwwccccccbbbbk.",
      ".kccccccccbbbbk.",
      "..kcccccccbbbk..",
      "...kccccccbbk...",
      "....kcccccbk....",
      ".....kcccbk.....",
      "......kcbk......",
      ".......kk.......",
    ],
  },
  sword: {
    palette: { k: "#1a1a1a", w: "#eef3f6", c: "#9aa7b0", y: "#d4a72c", h: "#5b3a1a" },
    rows: [
      ".............kcw",
      "............kcwk",
      "...........kcwk.",
      "..........kcwk..",
      ".........kcwk...",
      "........kcwk....",
      ".......kcwk.....",
      "....y.kcwk......",
      ".....ycwk.......",
      ".....hy.........",
      "....h..y........",
      "...h............",
      "..h.............",
      ".y..............",
    ],
  },
  tnt: {
    palette: { r: "#d32f2f", R: "#8e1b1b", w: "#eeeeee", k: "#222222" },
    rows: [
      "rrRrrrRrrrrRrrrR",
      "rRrrrRrrrRrrrRrr",
      "RrrrRrrrRrrrrRrr",
      "rrrRrrrRrrrRrrrr",
      "rrRrrrRrrrRrrrRr",
      "wwwwwwwwwwwwwwww",
      "wwkkkwkwkwkkkwww",
      "wwwkwwkkkwwkwwww",
      "wwwkwwkkkwwkwwww",
      "wwwkwwkwkwwkwwww",
      "wwwwwwwwwwwwwwww",
      "rrRrrrRrrrRrrrRr",
      "rRrrrRrrrRrrrRrr",
      "RrrrRrrrRrrrrRrr",
      "rrrRrrrRrrrRrrrr",
      "rrRrrrRrrrrRrrrR",
    ],
  },
  heart: {
    palette: { k: "#4a0e0e", r: "#e53935", R: "#b71c1c", w: "#ffcdd2" },
    rows: [
      "",
      "..kkkk....kkkk..",
      ".krrrrk..krrrrk.",
      "krwwrrrkkrrrrrrk",
      "krwrrrrrrrrrrrrk",
      "krrrrrrrrrrrrrRk",
      "krrrrrrrrrrrrrRk",
      ".krrrrrrrrrrrRk.",
      "..krrrrrrrrrRk..",
      "...krrrrrrrRk...",
      "....krrrrrRk....",
      ".....krrrRk.....",
      "......krRk......",
      ".......kk.......",
    ],
  },
  apple: {
    palette: { k: "#3b0d0d", r: "#d32f2f", R: "#9a1c1c", w: "#ff8a80", s: "#4e342e", l: "#43a047" },
    rows: [
      "........s.......",
      ".......s.ll.....",
      ".......sllll....",
      "...kkkksskkkk...",
      "..krrrrrrrrrrk..",
      ".krwrrrrrrrrrRk.",
      ".krwwrrrrrrrrRk.",
      ".krwrrrrrrrrrRk.",
      ".krrrrrrrrrrrRk.",
      ".krrrrrrrrrrrRk.",
      ".krrrrrrrrrrRRk.",
      "..krrrrrrrrRRk..",
      "..krrrrrrrRRRk..",
      "...krrRkkrRRk...",
      "....kkk..kkk....",
    ],
  },
  potion: {
    palette: { k: "#1b1b2f", b: "#8d6e63", g: "#cfe8ff", w: "#f3e5ff", p: "#b05cd6", P: "#7b2fa0" },
    rows: [
      "",
      "......kkkk......",
      "......kbbk......",
      "......kkkk......",
      "......kggk......",
      "......kggk......",
      "....kkggggkk....",
      "...kgwppppppk...",
      "..kgwppppppppk..",
      "..kgpppppppPPk..",
      "..kpppppppppPk..",
      "..kppppppppPPk..",
      "...kpppppPPPk...",
      "....kkkkkkkk....",
    ],
  },
  chest: {
    palette: { k: "#2e1a08", b: "#b8732e", B: "#8a5420", d: "#5a3510", y: "#f4c542" },
    rows: [
      "",
      ".kkkkkkkkkkkkkk.",
      ".kbbbbbbbbbbbbk.",
      ".kbBbbbbbbbbBbk.",
      ".kbbbbbbbbbbbbk.",
      ".kbbbbbbbbbbbbk.",
      ".kddddkyykddddk.",
      ".kbbbbkyykbbbbk.",
      ".kbbbbbkkbbbbbk.",
      ".kbBbbbbbbbbBbk.",
      ".kbbbbbbbbbbbbk.",
      ".kbbbbbbbbbbbbk.",
      ".kBbbbbbbbbbbBk.",
      ".kkkkkkkkkkkkkk.",
    ],
  },
  flame: {
    palette: { o: "#e65100", y: "#ffb300", w: "#fff59d" },
    rows: [
      "",
      ".......o........",
      "......oo........",
      "......oyo.......",
      ".....oyyo...o...",
      ".....oyyoo..oo..",
      "....oyyyyooooyo.",
      "...oyyyyyyyyyyo.",
      "...oyywyyyyyyyo.",
      "..oyywwyyyyywyo.",
      "..oyywwwyyywwyo.",
      "..oyyywwwwwwyyo.",
      "...oyyywwwwyyo..",
      "....ooyyyyyyoo..",
      "......oooooo....",
    ],
  },
  pearl: {
    palette: { k: "#0b2e2a", g: "#1f6f63", G: "#2fa08c", l: "#8ff0d8", d: "#134a42" },
    rows: [
      "",
      ".....kkkkkk.....",
      "...kkggggggkk...",
      "..kgGGggggggdk..",
      "..kGlGgggggddk..",
      ".kgGGggggggggdk.",
      ".kgggggggggggdk.",
      ".kgggggggggggdk.",
      ".kgggggggggggdk.",
      ".kggggggggggddk.",
      ".kgggggggggdddk.",
      "..kgggggggdddk..",
      "..kkggggdddkk...",
      "....kkkkkkk.....",
    ],
  },
  gold: {
    palette: { k: "#5a3d00", y: "#f5c542", Y: "#c99a1e", w: "#fff3b0" },
    rows: [
      "",
      "",
      "",
      "",
      "",
      "....kkkkkkkk....",
      "...kwwyyyyyyk...",
      "..kwyyyyyyyyYk..",
      ".kyyyyyyyyyYYYk.",
      ".kyyyyyyyyyYYYk.",
      ".kkkkkkkkkkkkkk.",
    ],
  },
  moon: {
    palette: { k: "#37474f", w: "#eceff1", g: "#b0bec5" },
    rows: [
      "",
      "..kkkkkkkkkkkk..",
      "..kwwwwwwwwwwk..",
      "..kwwgwwwwwwwk..",
      "..kwwwwwwggwwk..",
      "..kwgwwwwwggwk..",
      "..kwwwwwwwwwwk..",
      "..kwwwggwwwwwk..",
      "..kwwwggwwwgwk..",
      "..kwwwwwwwwwwk..",
      "..kwgwwwwwwwwk..",
      "..kwwwwwgwwwwk..",
      "..kwwwwwwwwwwk..",
      "..kkkkkkkkkkkk..",
    ],
  },
} satisfies Record<string, PixelIcon>;

export type PixelIconKey = keyof typeof pixelIcons;

export const pixelIconKeys = Object.keys(pixelIcons) as PixelIconKey[];

type Run = { x: number; y: number; w: number; fill: string };

/** Horizontal runs of one colour, so an icon is a few dozen rects, not 256.
 *  The drawn pixels are centred in the 16x16 grid, whatever the rows say. */
export function pixelRuns(key: string): Run[] {
  const icon: PixelIcon = pixelIcons[key as PixelIconKey] ?? pixelIcons.grass;
  const runs: Run[] = [];
  icon.rows.forEach((row, y) => {
    let x = 0;
    while (x < row.length) {
      const char = row[x];
      let end = x + 1;
      while (end < row.length && row[end] === char) end++;
      const fill = icon.palette[char];
      if (fill) runs.push({ x, y, w: end - x, fill });
      x = end;
    }
  });
  if (!runs.length) return runs;
  const left = Math.min(...runs.map((r) => r.x));
  const right = Math.max(...runs.map((r) => r.x + r.w));
  const top = Math.min(...runs.map((r) => r.y));
  const bottom = Math.max(...runs.map((r) => r.y + 1));
  const dx = Math.floor((16 - (right - left)) / 2) - left;
  const dy = Math.floor((16 - (bottom - top)) / 2) - top;
  return runs.map((r) => ({ ...r, x: r.x + dx, y: r.y + dy }));
}

const urls = new Map<string, string>();

/** A 16x16 PNG of the icon. Shown with `image-rendering: pixelated`, the
 *  browser scales it with hard edges at any zoom, unlike scaled SVG rects. */
export function pixelIconUrl(key: string): string {
  let url = urls.get(key);
  if (!url) {
    const canvas = document.createElement("canvas");
    canvas.width = canvas.height = 16;
    const ctx = canvas.getContext("2d")!;
    for (const run of pixelRuns(key)) {
      ctx.fillStyle = run.fill;
      ctx.fillRect(run.x, run.y, run.w, 1);
    }
    url = canvas.toDataURL("image/png");
    urls.set(key, url);
  }
  return url;
}
