/**
 * diagrams/svg.ts — SVG 手工生成原语 (spec `conflict_rules`: "SVG 负责专业 2D 符号")
 *
 * 无第三方图库。全部颜色走 CSS 变量, 明暗主题自动跟随。
 * 不含 fetch / eval / 动态 import / 远程资源引用。
 */

let seq = 0;

/** 生成页面内唯一 id (marker / clipPath 用), 避免多张 SVG 冲突 */
export function uid(prefix = 'd2'): string {
  seq += 1;
  return `${prefix}${seq.toString(36)}`;
}

export type Attrs = Record<string, string | number | undefined | null>;

/** HTML/SVG 属性转义 (实体用拼接构造, 避免源码内出现裸实体串) */
const ENT: Record<string, string> = {
  '&': '&' + 'amp;',
  '<': '&' + 'lt;',
  '>': '&' + 'gt;',
  '"': '&' + 'quot;',
  "'": '&' + '#39;',
};

export function esc(s: unknown): string {
  return String(s ?? '').replace(/[&<>"']/g, (c) => ENT[c] ?? c);
}

/** 属性串 */
export function a(o: Attrs): string {
  const parts: string[] = [];
  for (const [k, v] of Object.entries(o)) {
    if (v === undefined || v === null || v === false) continue;
    parts.push(`${k}="${esc(v)}"`);
  }
  return parts.length ? ' ' + parts.join(' ') : '';
}

/** 元素 */
export function el(tag: string, o: Attrs, inner = ''): string {
  return `<${tag}${a(o)}>${inner}</${tag}>`;
}

/** 文本 */
export function text(x: number, y: number, s: unknown, o: Attrs = {}): string {
  return el('text', { x, y, fill: 'var(--text)', 'font-size': 11, ...o }, esc(s));
}

/** 多行文本 (tspans) */
export function textLines(
  x: number, y: number, lines: string[], o: Attrs = {}, lh = 12,
): string {
  const inner = lines
    .map((ln, i) => `<tspan x="${x}" dy="${i === 0 ? 0 : lh}">${esc(ln)}</tspan>`)
    .join('');
  return el('text', { x, y, fill: 'var(--text)', 'font-size': 11, ...o }, inner);
}

/** 贪心换行 (ASCII 优先空格断行, 超长无空格则硬断) */
export function wrapText(s: unknown, max: number, maxLines = 3): string[] {
  const src = String(s ?? '');
  if (!src) return [];
  const lines: string[] = [];
  let cur = '';
  for (const ch of src) {
    cur += ch;
    if (cur.length >= max) {
      const sp = cur.lastIndexOf(' ');
      if (sp >= Math.floor(max * 0.55)) {
        lines.push(cur.slice(0, sp));
        cur = cur.slice(sp + 1);
      } else {
        lines.push(cur);
        cur = '';
      }
    }
  }
  if (cur) lines.push(cur);
  if (lines.length > maxLines) {
    const kept = lines.slice(0, maxLines);
    kept[maxLines - 1] = kept[maxLines - 1].slice(0, Math.max(1, max - 1)) + '…';
    return kept;
  }
  return lines;
}

/** 截断 */
export function trunc(s: unknown, n: number): string {
  const t = String(s ?? '');
  return t.length > n ? t.slice(0, Math.max(1, n - 1)) + '…' : t;
}

/** 箭头 marker 定义 (实心 = 依赖/调用, 空心 = 关联/实现, 三角 = 泛化) */
export function arrowDefs(id: string): string {
  return `<defs>
  <marker id="${id}-f" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
    <path d="M0 0 L10 5 L0 10 z" fill="var(--text-2)"/></marker>
  <marker id="${id}-o" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="8" markerHeight="8" orient="auto-start-reverse">
    <path d="M0 0 L10 5 L0 10" fill="none" stroke="var(--text-2)" stroke-width="1.3"/></marker>
  <marker id="${id}-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="8" markerHeight="8" orient="auto-start-reverse">
    <path d="M0 0 L10 5 L0 10" fill="none" stroke="var(--warn)" stroke-width="1.6"/></marker>
  <marker id="${id}-t" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="9" markerHeight="9" orient="auto-start-reverse">
    <path d="M0 0 L10 5 L0 10 z" fill="var(--text)"/></marker>
  <marker id="${id}-diamond" viewBox="0 0 12 12" refX="10" refY="6" markerWidth="10" markerHeight="10" orient="auto-start-reverse">
    <path d="M0 6 L6 0 L12 6 L6 12 z" fill="none" stroke="var(--text-2)" stroke-width="1.2"/></marker>
</defs>`;
}

/** SVG 根 (不写 xmlns — HTML 解析上下文自动进 SVG 命名空间) */
export function svgRoot(w: number, h: number, label: string, inner: string): string {
  return `<svg viewBox="0 0 ${w} ${h}" width="${w}" height="${h}" role="img" aria-label="${esc(label)}" `
    + `style="max-width:100%;height:auto;display:block;font-family:var(--font)">${inner}</svg>`;
}

/** 正交折线路径 (边走线, 不用贝塞尔以便对齐工程图风格) */
export function orthPath(x1: number, y1: number, x2: number, y2: number): string {
  const midX = (x1 + x2) / 2;
  return `M ${x1} ${y1} L ${midX} ${y1} L ${midX} ${y2} L ${x2} ${y2}`;
}

/** 圆角矩形路径 (用于状态/活动/参与者等) */
export function roundRect(x: number, y: number, w: number, h: number, r: number): string {
  const rr = Math.min(r, w / 2, h / 2);
  return `M ${x + rr} ${y} H ${x + w - rr} A ${rr} ${rr} 0 0 1 ${x + w} ${y + rr} `
    + `V ${y + h - rr} A ${rr} ${rr} 0 0 1 ${x + w - rr} ${y + h} H ${x + rr} `
    + `A ${rr} ${rr} 0 0 1 ${x} ${y + h - rr} V ${y + rr} A ${rr} ${rr} 0 0 1 ${x + rr} ${y} Z`;
}

/** 组合框 (UML alt/opt/loop/par 角标) */
export function frameCorner(x: number, y: number, w: number, h: number, label: string): string {
  return `<path d="${roundRect(x, y, w, h, 4)}" fill="none" stroke="var(--info)" stroke-width="1.2" stroke-dasharray="4 3"/>`
    + `<rect x="${x}" y="${y}" width="${Math.min(64, w)}" height="14" fill="var(--surface)" stroke="var(--info)" stroke-width="1"/>`
    + `<text x="${x + 5}" y="${y + 10}" font-size="10" fill="var(--info)" font-weight="600">${esc(label)}</text>`;
}

/** 域名 → CSS 变量 (主题自适应) */
export function domainVar(domainId: string): string {
  return `var(--dom-${domainId || 'Unknown'})`;
}
