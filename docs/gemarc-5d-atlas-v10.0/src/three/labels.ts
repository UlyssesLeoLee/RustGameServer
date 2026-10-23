/**
 * labels.ts — 自实现屏幕标签层 (spec `3D.adaptive_labels` / `labels`)
 *
 * spec 要求:
 *   · Node 与 Edge 均需有稳定名字和多视角屏幕标签
 *   · 面向相机、遮挡避让、LOD、文本截断/悬停完整名, **不能旋转到倒置**
 *   · Edge 独立名称与关系类型, 沿 Edge 方向排布但文字不得倒置
 *   · 模式 [Smart, All, RelatedOnly]; 优先级 Selected > DirectlyRelated > Context > Unrelated
 *   · 调整缩放/角度时无布局跳动
 *
 * 实现选择: **自实现 DOM 标签层** (不用 CSS2DRenderer, 也不做 sprite)。
 *   理由: DOM 标签天然是屏幕对齐的 —— 无论相机怎么转, 文字永远正立,
 *   因此"不能旋转到倒置"由构造方式保证, 而不是靠运行时纠正。
 *   Edge 标签是唯一需要旋转的: 它在 **屏幕空间** 旋转 (不是世界空间),
 *   并对角度做 ±90° 夹取 → 文字永远不会倒置。
 *
 * 遮挡避让: 每帧按 (优先级, 深度) 排序后依次放置, 与已放置标签矩形相交者隐藏。
 * 无布局跳动: 标签不参与 3D 布局, 位置每帧由投影计算, 不会引起重排抖动。
 */

import * as THREE from 'three';

export type LabelMode = 'Smart' | 'All' | 'RelatedOnly';
export type LabelPriority = 'Selected' | 'DirectlyRelated' | 'Context' | 'Unrelated';

export interface LabelTarget {
  id: string;
  kind: 'node' | 'edge' | 'room' | 'port';
  /** 完整名 (悬停可见) */
  text: string;
  /** 次要信息 (如关系类型 / 指标) */
  sub?: string;
  /** Three 系世界坐标 */
  world: THREE.Vector3;
  /** 近似世界半径 (用于 LOD 与截断预算) */
  radius: number;
  priority: LabelPriority;
  /** Edge 专用: 屏幕空间方向 (未归一化), 缺省则水平排布 */
  edgeDir?: { x: number; y: number } | null;
}

interface LabelEntry {
  el: HTMLDivElement;
  tip: HTMLDivElement;
  target: LabelTarget;
  lastText: string;
}

const PRIORITY_RANK: Record<LabelPriority, number> = {
  Selected: 0,
  DirectlyRelated: 1,
  Context: 2,
  Unrelated: 3,
};

export class LabelLayer {
  private container: HTMLDivElement;
  private tooltip: HTMLDivElement;
  private entries = new Map<string, LabelEntry>();
  private ctx2d: CanvasRenderingContext2D | null = null;
  private fontSize = 11;
  private onSelect: ((kind: LabelTarget['kind'], id: string) => void) | null = null;

  constructor(host: HTMLElement, onSelect?: (kind: LabelTarget['kind'], id: string) => void) {
    this.onSelect = onSelect ?? null;
    this.container = document.createElement('div');
    this.container.className = 'd3-labels';
    host.appendChild(this.container);

    this.tooltip = document.createElement('div');
    this.tooltip.className = 'd3-tip';
    this.tooltip.hidden = true;
    host.appendChild(this.tooltip);

    const c = document.createElement('canvas');
    this.ctx2d = c.getContext('2d');
  }

  /** 文本量测 (精确, 避免"看起来差不多"的截断) */
  private measure(text: string): number {
    if (!this.ctx2d) return text.length * this.fontSize * 0.55;
    this.ctx2d.font = `${this.fontSize}px ui-monospace, "Cascadia Mono", Consolas, monospace`;
    return this.ctx2d.measureText(text).width;
  }

  /** 按像素预算截断, 返回截断后的文本 (超长加省略号) */
  private truncate(text: string, maxWidth: number): string {
    if (this.measure(text) <= maxWidth) return text;
    let lo = 1;
    let hi = text.length;
    while (lo < hi) {
      const mid = (lo + hi + 1) >> 1;
      if (this.measure(text.slice(0, mid) + '…') <= maxWidth) lo = mid;
      else hi = mid - 1;
    }
    return text.slice(0, lo) + '…';
  }

  private ensure(id: string): LabelEntry {
    let e = this.entries.get(id);
    if (e) return e;
    const el = document.createElement('div');
    el.className = 'd3-label';
    el.setAttribute('data-kind', 'node');
    el.addEventListener('mouseenter', () => this.showTip(e));
    el.addEventListener('mouseleave', () => this.hideTip());
    el.addEventListener('click', (ev) => {
      ev.stopPropagation();
      if (this.onSelect) this.onSelect(e.target.kind, e.target.id);
    });
    this.container.appendChild(el);
    e = { el, tip: el, target: { id, kind: 'node', text: '', world: new THREE.Vector3(), radius: 1, priority: 'Unrelated' }, lastText: '' };
    this.entries.set(id, e);
    return e;
  }

  /**
   * 公開ラベルの登録 API。
   *
   * これが無いと `update()` が常に空の entries を走査し、
   * **ラベルが 1 つも描画されない** (对抗審査で発見: 160 ノード全是無文字)。
   * ensure() が private で呼び出し口も無かったため、層が静かに空のままだった。
   *
   * 呼び出し側は 3D シーン構築のたびに全ターゲットを渡す。
   * 消えたターゲットは DOM から除去する (リーク防止)。
   */
  setTargets(targets: LabelTarget[]): void {
    const keep = new Set<string>();
    for (const t of targets) {
      keep.add(t.id);
      const e = this.ensure(t.id);
      e.target = t;
      e.el.setAttribute('data-kind', t.kind);
    }
    for (const [id, e] of [...this.entries]) {
      if (keep.has(id)) continue;
      e.el.remove();
      this.entries.delete(id);
    }
  }

  /** 現在登録されているラベル数 (診断・テスト用) */
  get count(): number { return this.entries.size; }

  private showTip(e: LabelEntry): void {
    const t = e.target;
    this.tooltip.innerHTML = `<strong>${escapeHtml(t.text)}</strong>${t.sub ? `<br><span>${escapeHtml(t.sub)}</span>` : ''}`;
    this.tooltip.hidden = false;
    const r = e.el.getBoundingClientRect();
    const host = this.container.getBoundingClientRect();
    this.tooltip.style.left = `${r.left - host.left + r.width / 2}px`;
    this.tooltip.style.top = `${r.top - host.top - 6}px`;
    this.tooltip.classList.add('d3-tip--below');
  }

  private hideTip(): void {
    this.tooltip.hidden = true;
  }

  /**
   * 每帧更新: 投影 → 排序 → 截断 → 遮挡避让 → 定位。
   * camera 必须是已 updateMatrixWorld 的相机。
   */
  update(
    camera: THREE.Camera,
    width: number,
    height: number,
    mode: LabelMode,
    visibleIds: Set<string>,
  ): void {
    const v = new THREE.Vector3();
    const camPos = new THREE.Vector3();
    camera.getWorldPosition(camPos);
    const fovRad = (camera as THREE.PerspectiveCamera).fov
      ? ((camera as THREE.PerspectiveCamera).fov * Math.PI) / 180
      : Math.PI / 4;
    const pxPerUnit = height / (2 * Math.tan(fovRad / 2));

    const ordered: { entry: LabelEntry; sx: number; sy: number; depth: number; px: number; priority: number }[] = [];

    for (const [id, entry] of this.entries) {
      const t = entry.target;
      if (!visibleIds.has(id)) { entry.el.style.display = 'none'; continue; }
      v.copy(t.world).project(camera);
      if (v.z > 1 || v.z < -1) { entry.el.style.display = 'none'; continue; }
      const sx = (v.x * 0.5 + 0.5) * width;
      const sy = (-v.y * 0.5 + 0.5) * height;
      if (sx < -80 || sx > width + 80 || sy < -40 || sy > height + 40) {
        entry.el.style.display = 'none';
        continue;
      }
      const depth = camPos.distanceTo(t.world);
      const px = Math.max(4, t.radius * 2 * (pxPerUnit / Math.max(0.001, depth)));
      ordered.push({ entry, sx, sy, depth, px, priority: PRIORITY_RANK[t.priority] });
    }

    ordered.sort((a, b) => (a.priority - b.priority) || (a.depth - b.depth));

    const placed: { x1: number; y1: number; x2: number; y2: number }[] = [];

    /* Smart モードの予算:
     * 「投影サイズが小さいものだけ隠す」規則だけだと、遠景カメラでは
     * **全部が隠れて 1 つも出ない** (実測: visible=0)。
     * そのため、Unrelated のうち**カメラに最も近い N 個**は必ず残す。 */
    const SMART_BUDGET = 24;
    let smartKept = 0;

    for (const o of ordered) {
      const t = o.entry.target;
      // 模式过滤
      if (mode === 'RelatedOnly' && t.priority === 'Unrelated') { o.entry.el.style.display = 'none'; continue; }
      if (mode === 'Smart' && t.priority === 'Unrelated') {
        if (o.px < 26 && smartKept >= SMART_BUDGET) { o.entry.el.style.display = 'none'; continue; }
        smartKept += 1;
      }

      // 截断预算: 屏幕上的宝石越大, 可给的字符越多 (但不小于 8)
      const budget = Math.max(64, Math.min(240, o.px * 4.2 + 48));
      const text = this.truncate(t.text, budget);
      if (text !== o.entry.lastText) {
        o.entry.el.textContent = text;
        o.entry.lastText = text;
        o.entry.el.title = t.text;
      }
      o.entry.el.setAttribute('data-kind', t.kind);
      o.entry.el.setAttribute('data-prio', t.priority);

      // 遮挡避让: 与已放置 (更高优先级 / 更近) 的标签相交 → 隐藏
      const w = this.measure(text) + 8;
      const h = 15;
      const x1 = o.sx - w / 2;
      const y1 = o.sy - h - 6;
      const rect = { x1, y1, x2: x1 + w, y2: y1 + h };
      let clash = false;
      for (const p of placed) {
        if (rect.x1 < p.x2 && rect.x2 > p.x1 && rect.y1 < p.y2 && rect.y2 > p.y1) { clash = true; break; }
      }
      if (clash && t.priority === 'Unrelated') { o.entry.el.style.display = 'none'; continue; }
      placed.push(rect);

      o.entry.el.style.display = 'block';
      if (t.kind === 'edge' && t.edgeDir) {
        // 屏幕空间旋转 + ±90° 夹取 → 文字永不倒置
        let ang = (Math.atan2(t.edgeDir.y, t.edgeDir.x) * 180) / Math.PI;
        if (ang > 90) ang -= 180;
        if (ang < -90) ang += 180;
        o.entry.el.classList.add('d3-label--edge');
        o.entry.el.style.transform = `translate(-50%,-50%) translate(${o.sx.toFixed(1)}px,${o.sy.toFixed(1)}px) rotate(${ang.toFixed(1)}deg)`;
      } else {
        o.entry.el.classList.remove('d3-label--edge');
        o.entry.el.style.transform = `translate(-50%,-100%) translate(${o.sx.toFixed(1)}px,${(o.sy - 6).toFixed(1)}px)`;
      }
      o.entry.el.style.opacity = t.priority === 'Unrelated' ? '0.62' : '1';
    }
  }

  setFontSize(px: number): void {
    this.fontSize = px;
  }

  dispose(): void {
    for (const e of this.entries.values()) e.el.remove();
    this.entries.clear();
    this.container.remove();
    this.tooltip.remove();
  }
}

function escapeHtml(s: string): string {
  return String(s).replace(/[&<>"']/g, (c) =>
    ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]!));
}
