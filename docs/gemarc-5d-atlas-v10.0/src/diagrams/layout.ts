/**
 * diagrams/layout.ts — 确定性布局 (无第三方图库)
 *
 * 所有布局都是纯函数 + 固定排序 → 同一模型永远得到同一张图, 便于审查对比。
 */

export interface Box {
  x: number;
  y: number;
  w: number;
  h: number;
}

export interface PlaceOpts {
  cols: number;
  w: number;
  h: number;
  gapX?: number;
  gapY?: number;
  x0?: number;
  y0?: number;
}

/** 网格排布 */
export function grid<T>(items: T[], o: PlaceOpts): Map<T, Box> {
  const gapX = o.gapX ?? 40;
  const gapY = o.gapY ?? 30;
  const x0 = o.x0 ?? 0;
  const y0 = o.y0 ?? 0;
  const cols = Math.max(1, o.cols);
  const m = new Map<T, Box>();
  items.forEach((it, i) => {
    const c = i % cols;
    const r = Math.floor(i / cols);
    m.set(it, { x: x0 + c * (o.w + gapX), y: y0 + r * (o.h + gapY), w: o.w, h: o.h });
  });
  return m;
}

/** 单列纵向排布 */
export function column<T>(items: T[], o: { w: number; h: number; gapY?: number; x0?: number; y0?: number }): Map<T, Box> {
  const gapY = o.gapY ?? 24;
  const x0 = o.x0 ?? 0;
  const y0 = o.y0 ?? 0;
  const m = new Map<T, Box>();
  items.forEach((it, i) => {
    m.set(it, { x: x0, y: y0 + i * (o.h + gapY), w: o.w, h: o.h });
  });
  return m;
}

/** 按分组键排布 (每组一列, 组内纵排) */
export function columnsBy<T, K extends string>(
  items: T[],
  key: (t: T) => K,
  o: { w: number; h: number; gapX?: number; gapY?: number; x0?: number; y0?: number; order?: K[] },
): Map<T, Box> {
  const gapX = o.gapX ?? 70;
  const gapY = o.gapY ?? 24;
  const x0 = o.x0 ?? 0;
  const y0 = o.y0 ?? 0;
  const groups = new Map<K, T[]>();
  for (const it of items) {
    const k = key(it);
    if (!groups.has(k)) groups.set(k, []);
    groups.get(k)!.push(it);
  }
  let keys = [...groups.keys()];
  if (o.order) {
    keys.sort((a, b) => {
      const ia = o.order!.indexOf(a);
      const ib = o.order!.indexOf(b);
      return (ia < 0 ? 999 : ia) - (ib < 0 ? 999 : ib);
    });
  }
  const m = new Map<T, Box>();
  keys.forEach((k, ci) => {
    groups.get(k)!.forEach((it, ri) => {
      m.set(it, {
        x: x0 + ci * (o.w + gapX),
        y: y0 + ri * (o.h + gapY),
        w: o.w,
        h: o.h,
      });
    });
  });
  return m;
}

/** 取排布后的包围盒 (含 padding) */
export function bounds(boxes: Box[], pad = 20): { w: number; h: number } {
  if (!boxes.length) return { w: 200, h: 100 };
  let maxX = 0;
  let maxY = 0;
  for (const b of boxes) {
    maxX = Math.max(maxX, b.x + b.w);
    maxY = Math.max(maxY, b.y + b.h);
  }
  return { w: maxX + pad, h: maxY + pad };
}

/** 居中偏移 (把一排盒子对齐到画布中心) */
export function centerOffset(boxes: Box[], canvasW: number): number {
  if (!boxes.length) return 0;
  let maxX = 0;
  for (const b of boxes) maxX = Math.max(maxX, b.x + b.w);
  return Math.max(0, (canvasW - maxX) / 2);
}
