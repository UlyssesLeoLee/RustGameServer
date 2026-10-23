/**
 * app.ts — App Shell (spec `html_design_system.layout_contract`)
 *
 * 顶部: 项目/Revision/搜索/维度导航
 * 左侧: 目录      中央: 画布      右侧: 检查器      底部: 4D/5D 时间轴
 * Prototype 为独立 Tab, 不是第六维 (spec `placement.prototype`)
 */

import type { ArchitectureModel, Dimension, Evidence, GraphNode } from './types';
import { buildIndex, type GraphIndex } from './graph';
import { TOKENS, RING_STATES, DOMAIN_COLOR } from './tokens';

export interface DimensionContext {
  app: App;
  index: GraphIndex;
  /** 当前维度内容挂载点 */
  host: HTMLElement;
  /** 当前维度工具条 */
  toolbar: HTMLElement;
  /** 当前维度侧栏 */
  sidebar: HTMLElement;
  /** 检查器 */
  inspect(target: unknown): void;
  /** 状态栏 */
  status(text: string): void;
  /** 切换维度 */
  go(dim: Dimension): void;
}

export interface DimensionModule {
  id: Dimension;
  title: string;
  /** 该维度是否需要底部时间轴 */
  usesTimeline?: boolean;
  mount(ctx: DimensionContext): void | (() => void);
}

export class App {
  readonly model: ArchitectureModel;
  readonly evidence: Map<string, Evidence>;
  readonly index: GraphIndex;
  readonly registry = new Map<Dimension, DimensionModule>();
  current: Dimension = '0D';
  selection: { kind: 'node' | 'edge' | 'port' | 'none'; id: string | null } = { kind: 'none', id: null };
  private cleanup: (() => void) | null = null;

  constructor(model: ArchitectureModel, evidence: Evidence[]) {
    this.model = model;
    this.evidence = new Map(evidence.map((e) => [e.id, e]));
    this.index = buildIndex(model);
  }

  register(mod: DimensionModule) { this.registry.set(mod.id, mod); }

  /* ── DOM 句柄 ─────────────────────────────────────────────── */
  get el() {
    return {
      app: document.getElementById('app')!,
      panel: document.getElementById('dimension-panel')!,
      toolbar: document.getElementById('canvas-toolbar')!,
      sidebar: document.getElementById('sidebar-body')!,
      sidebarTitle: document.getElementById('sidebar-title')!,
      inspector: document.getElementById('inspector-body')!,
      timeline: document.getElementById('timeline')!,
      status: document.getElementById('status-text')!,
      statusRight: document.getElementById('status-right')!,
      search: document.getElementById('global-search-input') as HTMLInputElement,
      legend: document.getElementById('legend')!,
    };
  }

  /* ── 状态栏 ───────────────────────────────────────────────── */
  status(text: string) {
    const el = this.el.status;
    if (el) el.textContent = text;
  }

  /* ── 顶栏: 真实来源信息 (revision / branch / subject) ───────── */
  renderSourceInfo() {
    const m = this.model.meta;
    const rev = this.el.app.querySelector('#revision-text');
    const branch = this.el.app.querySelector('#branch-text');
    const subject = this.el.app.querySelector('#subject-label');
    if (rev) rev.textContent = m.shortRevision;
    if (branch) branch.textContent = m.branch;
    if (subject) subject.textContent = m.subject;
    const chip = this.el.app.querySelector('#revision-chip');
    if (chip) {
      chip.setAttribute('title',
        `モデル採取 Revision: ${m.revision}\nBranch: ${m.branch}\n採取日時: ${m.capturedAt}\n生成: ${m.generator}`);
    }
  }

  /* ── 维度切换 ─────────────────────────────────────────────── */
  go(dim: Dimension) {
    if (this.current === dim && this.cleanup) return;
    this.cleanup?.();
    this.cleanup = null;

    this.current = dim;
    this.el.app.setAttribute('data-dimension', dim);
    for (const t of document.querySelectorAll<HTMLButtonElement>('.dim-tab')) {
      t.setAttribute('aria-selected', String(t.dataset.dim === dim));
    }

    const mod = this.registry.get(dim);
    if (!mod) {
      this.el.panel.innerHTML = `<div class="panel"><h3>${dim} 未实现</h3>
        <p class="empty-hint">该维度模块尚未装载。状态: NOT_TESTED。</p></div>`;
      this.el.sidebar.innerHTML = '';
      this.el.toolbar.innerHTML = '';
      this.el.timeline.hidden = true;
      return;
    }

    this.el.sidebarTitle.textContent = mod.title;
    this.el.panel.innerHTML = '';
    this.el.sidebar.innerHTML = '';
    this.el.toolbar.innerHTML = '';
    this.el.timeline.hidden = !mod.usesTimeline;

    const ctx: DimensionContext = {
      app: this,
      index: this.index,
      host: this.el.panel,
      toolbar: this.el.toolbar,
      sidebar: this.el.sidebar,
      inspect: (t) => this.inspect(t),
      status: (t) => { this.el.status.textContent = t; },
      go: (d) => this.go(d),
    };
    const ret = mod.mount(ctx);
    if (typeof ret === 'function') this.cleanup = ret;
  }

  /* ── 检查器 ───────────────────────────────────────────────── */
  inspect(target: unknown) {
    const el = this.el.inspector;
    if (!target || typeof target !== 'object') {
      this.selection = { kind: 'none', id: null };
      el.innerHTML = '<p class="empty-hint">未选中对象。</p>';
      return;
    }
    const t = target as { __kind?: string; id?: string };
    if (t.__kind === 'node' && t.id) {
      const n = this.index.nodes.get(t.id);
      if (n) { this.selection = { kind: 'node', id: n.id }; el.innerHTML = this.renderNode(n); return; }
    }
    if (t.__kind === 'edge' && t.id) {
      const e = this.index.edges.get(t.id) ?? this.index.designedEdges.get(t.id);
      if (e) { this.selection = { kind: 'edge', id: e.id }; el.innerHTML = this.renderEdge(e); return; }
    }
    if (t.__kind === 'port' && t.id) {
      const p = this.index.ports.get(t.id);
      if (p) { this.selection = { kind: 'port', id: p.id }; el.innerHTML = this.renderPort(p); return; }
    }
    el.innerHTML = '<p class="empty-hint">未选中对象。</p>';
  }

  private evidenceList(ids: string[]): string {
    if (!ids.length) return '<p class="empty-hint">无证据记录。</p>';
    const rows = ids.map((id) => {
      const e = this.evidence.get(id);
      if (!e) return `<tr><td><code>${id}</code></td><td>—</td><td>—</td></tr>`;
      return `<tr>
        <td><code>${e.id}</code></td>
        <td>${e.kind}</td>
        <td><code title="${esc(e.fileLine)}">${esc(e.fileLine)}</code><br>
            <span class="empty-hint">rev ${esc(e.revision)}</span></td>
      </tr>
      ${e.snippet ? `<tr><td></td><td colspan="2"><pre style="margin:2px 0 6px;padding:6px;background:var(--raised);border-radius:4px;overflow-x:auto;font-size:11px">${esc(e.snippet)}</pre></td></tr>` : ''}`;
    }).join('');
    return `<table class="data"><thead><tr><th>ID</th><th>Kind</th><th>fileLine (rev)</th></tr></thead><tbody>${rows}</tbody></table>`;
  }

  private renderNode(n: GraphNode): string {
    const props = Object.entries(n.properties)
      .filter(([k]) => !k.startsWith('child') && k !== 'columns')
      .map(([k, v]) => `<dt>${esc(k)}</dt><dd>${esc(fmt(v))}</dd>`).join('');
    return `
      <div class="panel">
        <h3>${esc(n.name)} <span class="badge badge--${n.verification}">${n.verification}</span></h3>
        <dl class="kv">
          <dt>ID</dt><dd><code>${esc(n.id)}</code></dd>
          <dt>kind</dt><dd>${esc(n.kind)}</dd>
          <dt>qualified</dt><dd><code>${esc(n.qualifiedName)}</code></dd>
          <dt>role</dt><dd>${esc(n.role)}</dd>
          <dt>domain</dt><dd><span class="legend-swatch" style="display:inline-block;background:${DOMAIN_COLOR[n.domainId] ?? DOMAIN_COLOR.Unknown}"></span> ${esc(n.domainId)}</dd>
          ${props}
        </dl>
      </div>
      <div class="panel">
        <h3>证据 (${n.evidenceIds.length})</h3>
        <p class="empty-hint">每条含 fileLine 与 revision, 可独立复核。</p>
        ${this.evidenceList(n.evidenceIds)}
      </div>`;
  }

  private renderEdge(e: { id: string; type: string; directed: boolean; verification: string; properties: Record<string, unknown>; evidenceIds: string[]; sourceId: string | null; targetId: string | null; guardId: string | null }): string {
    const props = Object.entries(e.properties).map(([k, v]) => `<dt>${esc(k)}</dt><dd>${esc(fmt(v))}</dd>`).join('');
    return `
      <div class="panel">
        <h3>${esc(e.id)} <span class="badge badge--${e.verification}">${e.verification}</span></h3>
        <dl class="kv">
          <dt>type</dt><dd>${esc(e.type)}</dd>
          <dt>directed</dt><dd>${e.directed}</dd>
          <dt>source</dt><dd><code>${esc(e.sourceId ?? '—')}</code></dd>
          <dt>target</dt><dd><code>${esc(e.targetId ?? '—')}</code></dd>
          <dt>guard</dt><dd><code>${esc(e.guardId ?? '—')}</code></dd>
          ${props}
        </dl>
      </div>
      <div class="panel">
        <h3>证据 (${e.evidenceIds.length})</h3>
        ${this.evidenceList(e.evidenceIds)}
      </div>`;
  }

  private renderPort(p: { id: string; name: string; crate: string; requestType: string; responseType: string; type: string; evidenceIds: string[] }): string {
    return `
      <div class="panel">
        <h3>${esc(p.name)} <span class="badge badge--Designed">Designed</span></h3>
        <dl class="kv">
          <dt>ID</dt><dd><code>${esc(p.id)}</code></dd>
          <dt>crate</dt><dd>${esc(p.crate)}</dd>
          <dt>request</dt><dd><code>${esc(p.requestType)}</code></dd>
          <dt>response</dt><dd><code>${esc(p.responseType)}</code></dd>
          <dt>type</dt><dd>${esc(p.type)}</dd>
        </dl>
        <p class="empty-hint">⚠️ 端口由 proto 接口定义推导。按 implementation_gate, 接口定义<strong>不能</strong>证明业务行为已实现。</p>
      </div>
      <div class="panel"><h3>证据 (${p.evidenceIds.length})</h3>${this.evidenceList(p.evidenceIds)}</div>`;
  }

  /* ── 图例 ─────────────────────────────────────────────────── */
  renderLegend() {
    const items: [string, string][] = [
      ...Object.entries(DOMAIN_COLOR).map(([k, v]) => [k, v] as [string, string]),
      ...RING_STATES.map((s) => [s, ''] as [string, string]),
    ];
    this.el.legend.innerHTML = items.map(([name, color]) => `
      <span class="legend-item">
        <span class="legend-swatch" style="background:${color || 'var(--edge)'}"></span>${esc(name)}
      </span>`).join('');
  }

  /* ── 全局搜索 ─────────────────────────────────────────────── */
  search(q: string): { nodes: GraphNode[]; edges: string[] } {
    const s = q.trim().toLowerCase();
    if (!s) return { nodes: [], edges: [] };
    const nodes = [...this.index.nodes.values()].filter((n) =>
      n.name.toLowerCase().includes(s) || n.qualifiedName.toLowerCase().includes(s) || n.role.toLowerCase().includes(s));
    const edges: string[] = [];
    for (const e of this.index.edges.values()) {
      const hay = JSON.stringify(e.properties).toLowerCase();
      if (hay.includes(s)) edges.push(e.id);
    }
    return { nodes, edges };
  }
}

/* ── 工具 ─────────────────────────────────────────────────────── */
export function esc(s: unknown): string {
  return String(s ?? '').replace(/[&<>"']/g, (c) =>
    ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]!));
}

export function fmt(v: unknown): string {
  if (v === null || v === undefined) return '—';
  if (Array.isArray(v)) return v.length ? v.map((x) => fmt(x)).join(', ') : '(空)';
  if (typeof v === 'object') return JSON.stringify(v);
  return String(v);
}

export { TOKENS };
