import type { IconName } from "./icons";

type ProfileIcon = { icon: IconName; bg: string; fg: string };

/** The icons a profile can wear, keyed by what is stored in profiles.json. */
export const profileIcons = {
  cube: { icon: "cube", bg: "#2c2c3a", fg: "#d3d1c7" },
  grass: { icon: "tree", bg: "#173404", fg: "#97c459" },
  sword: { icon: "sword", bg: "#501313", fg: "#f09595" },
  gear: { icon: "gear", bg: "#26215c", fg: "#afa9ec" },
  brush: { icon: "brush", bg: "#412402", fg: "#fac775" },
  diamond: { icon: "diamond", bg: "#04342c", fg: "#5dcaa5" },
  flame: { icon: "flame", bg: "#4a1b0c", fg: "#f0997b" },
  rocket: { icon: "rocket", bg: "#042c53", fg: "#85b7eb" },
  skull: { icon: "skull", bg: "#2c2c2a", fg: "#b4b2a9" },
  castle: { icon: "castle", bg: "#4b1528", fg: "#ed93b1" },
  pick: { icon: "pick", bg: "#3a3a36", fg: "#d3d1c7" },
  moon: { icon: "moon", bg: "#1e1a4a", fg: "#cecbf6" },
} satisfies Record<string, ProfileIcon>;

export type ProfileIconKey = keyof typeof profileIcons;

export const profileIconKeys = Object.keys(profileIcons) as ProfileIconKey[];

export function profileIcon(key: string): ProfileIcon {
  return profileIcons[key as ProfileIconKey] ?? profileIcons.cube;
}
