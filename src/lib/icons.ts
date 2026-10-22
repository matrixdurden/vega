// Outline icons in the style of Tabler Icons (MIT), on a 24px grid.

const circle = (cx: number, cy: number, r: number) => `M${cx - r} ${cy}a${r} ${r} 0 1 0 ${2 * r} 0a${r} ${r} 0 1 0 ${-2 * r} 0`;

export const icons = {
  home: ["M5 12H3l9-9 9 9h-2", "M5 12v7a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-7", "M9 21v-6a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2v6"],
  puzzle: [
    "M4 7h3a1 1 0 0 0 1-1V5a2 2 0 0 1 4 0v1a1 1 0 0 0 1 1h3a1 1 0 0 1 1 1v3a1 1 0 0 0 1 1h1a2 2 0 0 1 0 4h-1a1 1 0 0 0-1 1v3a1 1 0 0 1-1 1h-3a1 1 0 0 1-1-1v-1a2 2 0 0 0-4 0v1a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1v-3a1 1 0 0 1 1-1h1a2 2 0 0 0 0-4H4a1 1 0 0 1-1-1V8a1 1 0 0 1 1-1",
  ],
  sliders: ["M4 6h8", "M16 6h4", "M4 12h2", "M10 12h10", "M4 18h11", "M19 18h1", circle(14, 6, 2), circle(8, 12, 2), circle(17, 18, 2)],
  play: ["M7 4v16l13-8z"],
  stop: ["M7 5h10a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2z"],
  plus: ["M12 5v14", "M5 12h14"],
  search: [circle(10, 10, 7), "M21 21l-6-6"],
  chevron: ["M6 9l6 6 6-6"],
  folder: ["M5 4h4l3 3h7a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2"],
  trash: ["M4 7h16", "M10 11v6", "M14 11v6", "M5 7l1 12a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2l1-12", "M9 7V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v3"],
  x: ["M18 6L6 18", "M6 6l12 12"],
  minus: ["M5 12h14"],
  square: ["M7 6h10a1 1 0 0 1 1 1v10a1 1 0 0 1-1 1H7a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1z"],
  check: ["M5 12l5 5L20 7"],
  download: ["M4 17v2a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2", "M7 11l5 5 5-5", "M12 4v12"],
  terminal: ["M5 7l5 5-5 5", "M12 19h7"],
  loader: ["M12 3a9 9 0 1 0 9 9"],
  alert: [circle(12, 12, 9), "M12 8v4", "M12 16h.01"],
  user: [circle(12, 7, 4), "M6 21v-2a4 4 0 0 1 4-4h4a4 4 0 0 1 4 4v2"],
  globe: [circle(12, 12, 9), "M3.6 9h16.8", "M3.6 15h16.8", "M11.5 3a17 17 0 0 0 0 18", "M12.5 3a17 17 0 0 1 0 18"],
  memory: ["M6 5h12a1 1 0 0 1 1 1v12a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1V6a1 1 0 0 1 1-1z", "M9 9h6v6H9z", "M3 10h2", "M3 14h2", "M10 3v2", "M14 3v2", "M21 10h-2", "M21 14h-2", "M14 21v-2", "M10 21v-2"],
  eye: ["M10 12a2 2 0 1 0 4 0a2 2 0 1 0-4 0", "M21 12c-2.4 4-5.4 6-9 6s-6.6-2-9-6c2.4-4 5.4-6 9-6s6.6 2 9 6"],
  external: ["M12 6H6a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-6", "M11 13l9-9", "M15 4h5v5"],
  github: [
    "M9 19c-4.3 1.4-4.3-2.5-6-3m12 5v-3.5c0-1 .1-1.4-.5-2 2.8-.3 5.5-1.4 5.5-6a4.6 4.6 0 0 0-1.3-3.2 4.2 4.2 0 0 0-.1-3.2s-1.1-.3-3.5 1.3a12.3 12.3 0 0 0-6.2 0C6.5 2.8 5.4 3.1 5.4 3.1a4.2 4.2 0 0 0-.1 3.2A4.6 4.6 0 0 0 4 9.5c0 4.6 2.7 5.7 5.5 6-.6.6-.6 1.2-.5 2V21",
  ],
} satisfies Record<string, string[]>;

export type IconName = keyof typeof icons;

/** Icons drawn as solid shapes rather than outlines. */
export const filled: ReadonlySet<IconName> = new Set(["play", "stop"]);
