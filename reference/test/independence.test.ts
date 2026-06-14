import { test } from "node:test";
import assert from "node:assert/strict";
import { makeSeed } from "../src/hash.ts";
import { baseRarity } from "../src/rarity.ts";
import { colorMod } from "../src/appearance.ts";

const enc = (s: string) => new TextEncoder().encode(s);

function pearson(xs: number[], ys: number[]): number {
  const n = xs.length;
  const mx = xs.reduce((a, b) => a + b, 0) / n;
  const my = ys.reduce((a, b) => a + b, 0) / n;
  let sxy = 0, sxx = 0, syy = 0;
  for (let i = 0; i < n; i++) {
    const dx = xs[i] - mx, dy = ys[i] - my;
    sxy += dx * dy; sxx += dx * dx; syy += dy * dy;
  }
  return sxy / Math.sqrt(sxx * syy);
}

// 「色違いはレア度と独立」を統計的に確認（docs/01 1.5 / 必須要件）。
test("baseRarity and color are uncorrelated (independent streams)", () => {
  const N = 40_000;
  const rar: number[] = [], hue: number[] = [], sat: number[] = [];
  for (let i = 0; i < N; i++) {
    const seed = makeSeed(enc("ind#" + i));
    rar.push(baseRarity(seed));
    const c = colorMod(seed);
    hue.push(c.hShift);
    sat.push(c.sMul);
  }
  const rHue = pearson(rar, hue);
  const rSat = pearson(rar, sat);
  assert.ok(Math.abs(rHue) < 0.02, `corr(rarity,hShift)=${rHue.toFixed(4)} not ~0`);
  assert.ok(Math.abs(rSat) < 0.02, `corr(rarity,sMul)=${rSat.toFixed(4)} not ~0`);
});
