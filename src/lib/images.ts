const SIZE = 96;

/** Squares and shrinks a picture into a small PNG data URL for a profile
 *  icon, so profiles.json stays small. Pixel art stays crisp when enlarged. */
export async function toIconDataUrl(source: Blob | string): Promise<string> {
  const blob = typeof source === "string" ? await (await fetch(source)).blob() : source;
  const bitmap = await createImageBitmap(blob);
  const side = Math.min(bitmap.width, bitmap.height);
  const canvas = document.createElement("canvas");
  canvas.width = canvas.height = SIZE;
  const ctx = canvas.getContext("2d")!;
  ctx.imageSmoothingEnabled = side > SIZE;
  ctx.drawImage(bitmap, (bitmap.width - side) / 2, (bitmap.height - side) / 2, side, side, 0, 0, SIZE, SIZE);
  return canvas.toDataURL("image/png");
}
