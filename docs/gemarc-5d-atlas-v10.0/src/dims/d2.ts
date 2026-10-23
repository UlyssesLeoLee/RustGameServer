/**
 * d2.ts — 2D 无限画布 + 独立专业工程图审查单元 (spec `immutable_dimensions.2D`)
 *
 * spec 2D:
 *   definition: 全设计元素无限画布 + 独立专业工程图审查单元
 *   canvas: Pan/Zoom, Minimap, Search, Layer, Filter, Focus, Group, Collapse,
 *           MultiSelect, Undo/Redo, Node/Edge/Port 独立选择, 关系与需求追踪
 *   diagrams: 14 种 (UML_Class / Component / Package / Sequence / State /
 *             Activity / Deployment / UseCase / ER / DFD / BPMN / Security /
 *             Network / DataFlow) — 各自独立渲染器, 见 src/diagrams/**
 *   detachable_review: 每种图可独立展开/浮窗/移动/缩放/对比
 *   interoperability: 版本化 Graph/Workflow JSON Schema + 适配器子集与语义损失
 *                     报告 + AI 受控 Patch
 *
 * 实现约束:
 *   - SVG 手工生成, 无第三方图库; 事件委托 (不逐节点挂监听器)
 *   - 零网络请求: 无 fetch / 无动态 import / 无 eval / 无 CDN
 *   - 新样式全部在本文件内以局部 <style> 注入, 不改 src/styles.css
 *   - **维度链**: 1D の Prompt スコープを受け取り (何を描くかを決め)、
 *     選択を公開して 3D に渡す (2D → 3D)
 *   - **デモツアー**: 認知負荷を下げる導線 (src/dims/demo2d.ts)
 */

import type { DimensionModule, DimensionContext } from '../core/app';
import { esc } from '../core/app';
import { bus } from '../core/bus';
import { startDemoTour } from './demo2d';
import type { GraphNode, GraphEdge, GraphPort, CaseRecord, CausalRule, Requirement } from '../core/types';
import { DIAGRAMS, rendererFor } from '../diagrams/registry';
import type { DiagramContext, DiagramSelection, DiagramType } from '../diagrams/types';

/* ── 常量 ──────────────────────────────────────────────────────────── */

const NODE_W: Record<string, number> = { Service: 136, Api: 130, Table: 140 };
const NODE_H: Record<string, number> = { Service: 62, Api: 46, Table: 40 };
const PORT_SIZE = 7;
const DOMAIN_ORDER = ['Core', 'Interaction', 'Data', 'Security', 'Infrastructure', 'External', 'Operations', 'Unknown'];
const GRAPH_SCHEMA = 'rgs.graph/1.0.0';
const WORKFLOW_SCHEMA = 'rgs.workflow/1.0.0';
const PATCH_SCHEMA = 'rgs.graph-patch/1.0.0';
const UNDO_LIMIT = 60;   // spec 要求 ≥ 20 步

const LAYERS: { key: string; label: string; hint: string }[] = [
  { key: 'service', label: '服务 Service', hint: '31 个 crate' },
  { key: 'api', label: '接口 Api', hint: '33 个 proto service' },
  { key: 'table', label: '表 Table', hint: '96 张真实表' },
  { key: 'edge', label: '边 Implemented', hint: '20 条真实调用' },
  { key: 'edgeInferred', label: '边 Inferred', hint: '1 条仅注释提及' },
  { key: 'port', label: '端口 Port', hint: '710 个 rpc' },
];

/* ── 状态类型 ──────────────────────────────────────────────────────── */

interface Group { id: string; name: string; nodeIds: string[]; }

interface Snapshot {
  positions: Record<string, { x: number; y: number }>;
  groups: Group[];
  collapsed: string[];
  hidden: string[];
}

interface Pos { x: number; y: number; }

const D2_CSS = `
.d2-wrap{position:absolute;inset:0;display:flex;min-height:0}
.d2-stage{position:relative;flex:1 1 auto;min-width:0;overflow:hidden;background:var(--bg)}
.d2-svg{position:absolute;inset:0;width:100%;height:100%;display:block;cursor:grab;touch-action:none}
.d2-svg.is-panning{cursor:grabbing}
.d2-node{cursor:pointer}
.d2-node:focus{outline:none}
.d2-node:focus-visible rect.d2-box{stroke:var(--focus);stroke-width:2.5}
.d2-node[aria-pressed="true"] rect.d2-box{stroke:var(--focus);stroke-width:2.5}
.d2-node.is-dim{opacity:.16}
.d2-node.is-hit rect.d2-box{stroke:var(--warn);stroke-width:2.2}
.d2-edge-hit{fill:none;stroke:transparent;stroke-width:16;cursor:pointer}
.d2-edge-hit:focus{outline:none}
.d2-edge.is-sel path.d2-wire{stroke:var(--focus);stroke-width:2.6}
.d2-port{cursor:pointer;stroke:var(--bg);stroke-width:1}
.d2-port:focus{outline:none}
.d2-port:focus-visible,.d2-port[aria-pressed="true"]{stroke:var(--focus);stroke-width:2}
.d2-minimap{position:absolute;right:10px;bottom:10px;width:190px;height:130px;background:color-mix(in srgb,var(--surface) 88%,transparent);
  border:1px solid var(--edge);border-radius:8px;overflow:hidden;z-index:5}
.d2-minimap svg{width:100%;height:100%;display:block}
.d2-legend{position:absolute;left:10px;bottom:10px;display:flex;flex-direction:column;gap:3px;padding:7px 9px;
  background:color-mix(in srgb,var(--surface) 88%,transparent);border:1px solid var(--edge);border-radius:8px;
  font-size:10px;color:var(--text-2);z-index:5;max-width:260px}
.d2-legend b{color:var(--text);font-weight:600}
.d2-sect{margin-bottom:8px;border:1px solid var(--edge);border-radius:8px;background:var(--surface)}
.d2-sect>summary{cursor:pointer;padding:6px 8px;font-size:11px;font-weight:600;color:var(--text-2);
  list-style:none;display:flex;justify-content:space-between;align-items:center}
.d2-sect>summary::-webkit-details-marker{display:none}
.d2-sect>summary::before{content:"▸";margin-right:6px;display:inline-block;transition:transform var(--t-micro) var(--ease)}
.d2-sect[open]>summary::before{transform:rotate(90deg)}
.d2-sect-body{padding:0 8px 8px;font-size:11px}
.d2-layer{display:flex;align-items:center;gap:6px;padding:2px 0;cursor:pointer}
.d2-layer input{margin:0}
.d2-layer span.hint{margin-left:auto;color:var(--text-2);font-size:9px}
.d2-dg-btn{display:block;width:100%;text-align:left;margin:2px 0;padding:4px 6px;font-size:11px;background:var(--raised);
  border:1px solid var(--edge);border-radius:6px;color:var(--text-2);cursor:pointer}
.d2-dg-btn:hover{color:var(--text)}
.d2-dg-btn[aria-current="true"]{border-color:var(--focus);color:var(--text)}
.d2-hitlist{max-height:150px;overflow:auto;margin:0;padding:0;list-style:none}
.d2-hitlist li{padding:2px 0}
.d2-hitlist button{width:100%;text-align:left;background:none;border:none;color:var(--text-2);font-size:11px;cursor:pointer;padding:1px 2px;border-radius:4px}
.d2-hitlist button:hover{background:var(--raised);color:var(--text)}
.d2-float-layer{position:fixed;inset:0;z-index:900;pointer-events:none}
.d2-fw{position:absolute;pointer-events:auto;background:var(--surface);border:1px solid var(--edge);border-radius:var(--r-md);
  box-shadow:var(--shadow-3);display:flex;flex-direction:column;overflow:hidden}
.d2-fw-head{display:flex;align-items:center;gap:6px;padding:5px 8px;background:var(--raised);border-bottom:1px solid var(--edge);
  cursor:grab;font-size:12px;user-select:none}
.d2-fw-head strong{font-weight:600}
.d2-fw-head .sp{flex:1 1 auto}
.d2-fw-btn{height:22px;min-width:22px;padding:0 6px;font-size:11px;background:transparent;border:1px solid var(--edge);
  border-radius:4px;color:var(--text-2);cursor:pointer}
.d2-fw-btn:hover{color:var(--text);background:var(--surface)}
.d2-fw-body{flex:1 1 auto;overflow:auto;background:var(--bg)}
.d2-fw-body .d2-fw-inner{transform-origin:top left}
.d2-fw-note{padding:6px 8px;font-size:10px;color:var(--text-2);border-top:1px solid var(--edge);background:var(--surface);max-height:96px;overflow:auto}
@media (prefers-reduced-motion: reduce){.d2-fw,.d2-fw-body{transition:none!important}}
`;

export const d2: DimensionModule = {
  id: '2D',
  title: '2D 无限画布',

  mount(ctx: DimensionContext) {
    const { app, index, host, toolbar, sidebar } = ctx;

    /* ── 局部样式 (不改 src/styles.css) ───────────────────────── */
    const style = document.createElement('style');
    style.id = 'd2-style';
    style.textContent = D2_CSS;
    document.head.appendChild(style);

    /* ── 容器 (监听器只挂在本容器上, 切换维度时整体移除) ─────── */
    const wrap = document.createElement('div');
    wrap.className = 'd2-wrap';
    const stage = document.createElement('div');
    stage.className = 'd2-stage';
    wrap.appendChild(stage);
    host.appendChild(wrap);

    /* ── 状态 ─────────────────────────────────────────────────── */
    const state = {
      view: { x: 40, y: 30, k: 0.7 },
      bandBox: null as { x: number; y: number; w: number; h: number } | null,
      selNodes: new Set<string>(),
      selEdges: new Set<string>(),
      selPorts: new Set<string>(),
      focus: null as string | null,
      search: '',
      domainFilter: '' as string,
      positions: new Map<string, Pos>(),
      groups: [] as Group[],
      collapsed: new Set<string>(),
      hidden: new Set<string>(),
  /** 1D Prompt 范围のみ表示する (1D → 2D の链) */
  promptOnly: false,
      undo: [] as { label: string; snap: Snapshot }[],
      redo: [] as { label: string; snap: Snapshot }[],
    };

    /* ── 辅助模型 (4D/5D/需求) ────────────────────────────────── */
    const rawModels = (window as unknown as { __ATLAS_MODELS?: Record<string, unknown> }).__ATLAS_MODELS ?? {};
    const aux = {
      cases: (rawModels.cases as CaseRecord[]) ?? [],
      rules: (rawModels.rules as CausalRule[]) ?? [],
      requirements: (rawModels.requirements as Requirement[]) ?? [],
    };
    const traceDoc = (rawModels.traceability as {
      nodeToRequirement?: { nodeId: string; requirementIds: string[] }[];
      caseToNode?: { caseId: string; nodeIds: string[] }[];
      requirements?: Requirement[];
    }) ?? {};
    const reqById = new Map<string, Requirement>();
    for (const r of (traceDoc.requirements ?? aux.requirements)) reqById.set(r.id, r);
    const caseById = new Map<string, CaseRecord>();
    for (const c of aux.cases) caseById.set(c.caseId, c);

    /* ── 初始布局 (确定性) ────────────────────────────────────── */
    const services = [...index.nodes.values()].filter((n) => n.kind === 'Service');
    const apis = [...index.nodes.values()].filter((n) => n.kind === 'Api');
    const tables = [...index.nodes.values()].filter((n) => n.kind === 'Table');
    const crateOf = (n: GraphNode) => String(n.properties.crate ?? '');
    const svcByCrate = new Map<string, GraphNode>();
    for (const s of services) svcByCrate.set(crateOf(s), s);
    const apiByCrate = new Map<string, GraphNode[]>();
    for (const a of apis) {
      const c = crateOf(a);
      if (!apiByCrate.has(c)) apiByCrate.set(c, []);
      apiByCrate.get(c)!.push(a);
    }
    const tblByCrate = new Map<string, GraphNode[]>();
    for (const t of tables) {
      const c = crateOf(t);
      if (!tblByCrate.has(c)) tblByCrate.set(c, []);
      tblByCrate.get(c)!.push(t);
    }
    const portsByCrate = new Map<string, GraphPort[]>();
    for (const p of index.ports.values()) {
      if (!portsByCrate.has(p.crate)) portsByCrate.set(p.crate, []);
      portsByCrate.get(p.crate)!.push(p);
    }

    function initialLayout() {
      state.positions.clear();
      const domains = DOMAIN_ORDER.filter((d) => services.some((s) => s.domainId === d));
      domains.forEach((d, di) => {
        const svcs = services.filter((s) => s.domainId === d).sort((a, b) => a.name.localeCompare(b.name));
        const svcX = 60 + di * 460;
        svcs.forEach((s, i) => state.positions.set(s.id, { x: svcX, y: 50 + i * 108 }));
        const dApis: GraphNode[] = [];
        const dTbls: GraphNode[] = [];
        for (const s of svcs) {
          dApis.push(...(apiByCrate.get(crateOf(s)) ?? []));
          dTbls.push(...(tblByCrate.get(crateOf(s)) ?? []));
        }
        dApis.sort((a, b) => a.name.localeCompare(b.name)).forEach((a, i) =>
          state.positions.set(a.id, { x: svcX + 168, y: 50 + i * 32 }));
        dTbls.sort((a, b) => a.name.localeCompare(b.name)).forEach((t, i) =>
          state.positions.set(t.id, { x: svcX + 320, y: 50 + i * 28 }));
      });
    }
    initialLayout();

    /* ── 快照 / 撤销重做 (真撤销: 完整状态回滚) ────────────────── */
    function snapshot(): Snapshot {
      return {
        positions: Object.fromEntries([...state.positions].map(([k, v]) => [k, { ...v }])),
        groups: state.groups.map((g) => ({ ...g, nodeIds: [...g.nodeIds] })),
        collapsed: [...state.collapsed],
        hidden: [...state.hidden],
      };
    }
    function restore(s: Snapshot) {
      state.positions = new Map(Object.entries(s.positions).map(([k, v]) => [k, { ...v }]));
      state.groups = s.groups.map((g) => ({ ...g, nodeIds: [...g.nodeIds] }));
      state.collapsed = new Set(s.collapsed);
      state.hidden = new Set(s.hidden);
    }
    function pushUndo(label: string, snap: Snapshot) {
      state.undo.push({ label, snap });
      if (state.undo.length > UNDO_LIMIT) state.undo.shift();
      state.redo.length = 0;
    }
    /** 必须在**变更之前**调用, 否则撤销会恢复成"变更后"的状态 */
    function commit(label: string) {
      pushUndo(label, snapshot());
    }
    function undo() {
      const e = state.undo.pop();
      if (!e) { ctx.status('无可撤销的操作'); return; }
      state.redo.push({ label: e.label, snap: snapshot() });
      restore(e.snap);
      render();
      ctx.status(`撤销: ${e.label} (剩余 ${state.undo.length} 步)`);
    }
    function redo() {
      const e = state.redo.pop();
      if (!e) { ctx.status('无可重做的操作'); return; }
      state.undo.push({ label: e.label, snap: snapshot() });
      restore(e.snap);
      render();
      ctx.status(`重做: ${e.label} (剩余 ${state.redo.length} 步)`);
    }

    /* ── 可见性 / 过滤 ────────────────────────────────────────── */
    const layerOn = (k: string) => !state.hidden.has(k);
    function visibleNode(n: GraphNode): boolean {
      if (!layerOn(n.kind === 'Service' ? 'service' : n.kind === 'Api' ? 'api' : 'table')) return false;
      if (state.domainFilter && n.domainId !== state.domainFilter) return false;
      // 1D Prompt の范围に限定する (1D → 2D の链)
      if (state.promptOnly && bus.snapshot.promptScopeNodeIds.size
          && !bus.snapshot.promptScopeNodeIds.has(n.id)) return false;
      return true;
    }
    function groupOf(nodeId: string): Group | undefined {
      return state.groups.find((g) => g.nodeIds.includes(nodeId));
    }
    /** 折叠后节点映射到组节点 */
    function effectiveNodeId(id: string): string {
      const g = groupOf(id);
      if (g && state.collapsed.has(g.id)) return `group:${g.id}`;
      return id;
    }

    /* ── 画布几何 ─────────────────────────────────────────────── */
    function boxOf(id: string): { x: number; y: number; w: number; h: number } | null {
      if (id.startsWith('group:')) {
        const gid = id.slice(6);
        const g = state.groups.find((x) => x.id === gid);
        if (!g) return null;
        const ps = g.nodeIds.map((n) => state.positions.get(n)).filter(Boolean) as Pos[];
        if (!ps.length) return null;
        const minX = Math.min(...ps.map((p) => p.x));
        const minY = Math.min(...ps.map((p) => p.y));
        const maxX = Math.max(...ps.map((p) => p.x + NODE_W[index.nodes.get(g.nodeIds[0])?.kind ?? 'Service']));
        const maxY = Math.max(...ps.map((p) => p.y + NODE_H[index.nodes.get(g.nodeIds[0])?.kind ?? 'Service']));
        return { x: minX, y: minY, w: Math.max(150, maxX - minX), h: Math.max(70, maxY - minY) };
      }
      const n = index.nodes.get(id);
      if (!n) return null;
      const p = state.positions.get(id);
      if (!p) return null;
      return { x: p.x, y: p.y, w: NODE_W[n.kind] ?? 130, h: NODE_H[n.kind] ?? 44 };
    }

    /* ── 渲染: 画布 SVG ───────────────────────────────────────── */
    function renderCanvas(): string {
      const v = state.view;
      const parts: string[] = [];
      parts.push(`<defs>
        <pattern id="d2-grid" width="32" height="32" patternUnits="userSpaceOnUse">
          <path d="M 32 0 L 0 0 0 32" fill="none" stroke="var(--edge)" stroke-width="0.5" opacity="0.45"/>
        </pattern>
        <marker id="d2-arr" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
          <path d="M0 0 L10 5 L0 10 z" fill="var(--text-2)"/></marker>
        <marker id="d2-arr-warn" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
          <path d="M0 0 L10 5 L0 10 z" fill="var(--warn)"/></marker>
      </defs>`);
      parts.push(`<rect x="-100000" y="-100000" width="200000" height="200000" fill="url(#d2-grid)"/>`);
      parts.push(`<g class="d2-viewport" transform="translate(${v.x} ${v.y}) scale(${v.k})">`);

      // 框选矩形 (Shift+拖拽)
      if (state.bandBox) {
        const b = state.bandBox;
        parts.push(`<rect id="d2-band" x="${b.x}" y="${b.y}" width="${b.w}" height="${b.h}" `
          + `fill="var(--focus)" fill-opacity="0.12" stroke="var(--focus)" stroke-width="1" stroke-dasharray="4 3"/>`);
      }

      // 分组框 (虚线, 在节点之下)
      for (const g of state.groups) {
        const ps = g.nodeIds.map((n) => state.positions.get(n)).filter(Boolean) as Pos[];
        if (!ps.length) continue;
        const minX = Math.min(...ps.map((p) => p.x)) - 12;
        const minY = Math.min(...ps.map((p) => p.y)) - 24;
        const maxX = Math.max(...ps.map((p) => p.x + 140)) + 12;
        const maxY = Math.max(...ps.map((p) => p.y + 64)) + 12;
        const isCol = state.collapsed.has(g.id);
        parts.push(`<rect x="${minX}" y="${minY}" width="${maxX - minX}" height="${maxY - minY}" rx="10" `
          + `fill="none" stroke="var(--focus)" stroke-width="1.2" stroke-dasharray="7 5" opacity="0.75"/>`);
        parts.push(`<text x="${minX + 6}" y="${minY - 6}" font-size="11" fill="var(--focus)">`
          + `«group» ${esc(g.name)}${isCol ? ' (折叠)' : ''} · ${g.nodeIds.length}</text>`);
      }

      // 边 (先画, 节点压在上面)
      const edgeList: { e: GraphEdge; inferred: boolean }[] = [];
      for (const e of index.edges.values()) edgeList.push({ e, inferred: false });
      if (layerOn('edgeInferred')) for (const e of index.designedEdges.values()) edgeList.push({ e, inferred: true });

      for (const { e, inferred } of edgeList) {
        if (!layerOn('edge')) continue;
        const sId = effectiveNodeId(e.sourceId ?? '');
        const tId = effectiveNodeId(e.targetId ?? '');
        const sb = boxOf(sId);
        const tb = boxOf(tId);
        if (!sb || !tb) continue;
        const x1 = sb.x + sb.w;
        const y1 = sb.y + sb.h / 2;
        const x2 = tb.x;
        const y2 = tb.y + tb.h / 2;
        const midX = (x1 + x2) / 2;
        const d = `M ${x1} ${y1} L ${midX} ${y1} L ${midX} ${y2} L ${x2} ${y2}`;
        const sel = state.selEdges.has(e.id);
        const color = inferred ? 'var(--warn)' : 'var(--text-2)';
        parts.push(`<g class="d2-edge${sel ? ' is-sel' : ''}" data-edge-id="${esc(e.id)}">`);
        parts.push(`<path class="d2-wire" d="${d}" fill="none" stroke="${color}" stroke-width="${sel ? 2.4 : 1.2}" `
          + `${inferred ? 'stroke-dasharray="7 5"' : ''} marker-end="url(#${inferred ? 'd2-arr-warn' : 'd2-arr'})"/>`);
        parts.push(`<path class="d2-edge-hit" d="${d}" data-edge-id="${esc(e.id)}" tabindex="0" role="button" `
          + `aria-label="边 ${esc(e.id)} ${esc(e.properties.calleeService ?? '')} ${esc(e.verification)} ${esc(String(e.properties.fileLine ?? ''))}"/>`);
        parts.push(`<text x="${midX}" y="${(y1 + y2) / 2 - 6}" font-size="9" fill="var(--text-2)" text-anchor="middle">`
          + `${esc(String(e.properties.calleeService ?? ''))}</text>`);
        parts.push(`<text x="${midX}" y="${(y1 + y2) / 2 + 7}" font-size="8" fill="var(--text-2)" text-anchor="middle">`
          + `${esc(String(e.properties.fileLine ?? ''))}</text>`);
        parts.push('</g>');
      }

      // 节点
      const collapsedMember = new Set<string>();
      for (const g of state.groups) if (state.collapsed.has(g.id)) for (const n of g.nodeIds) collapsedMember.add(n);

      for (const n of index.nodes.values()) {
        if (collapsedMember.has(n.id)) continue;
        if (!visibleNode(n)) continue;
        const p = state.positions.get(n.id);
        if (!p) continue;
        const w = NODE_W[n.kind] ?? 130;
        const h = NODE_H[n.kind] ?? 44;
        const sel = state.selNodes.has(n.id);
        const dim = state.focus ? !isRelated(n.id) : false;
        const hit = state.search ? isMatch(n) : false;
        const domColor = `var(--dom-${n.domainId || 'Unknown'})`;
        parts.push(`<g class="d2-node${dim ? ' is-dim' : ''}${hit ? ' is-hit' : ''}" data-node-id="${esc(n.id)}" `
          + `tabindex="0" role="button" aria-pressed="${sel}" aria-label="${esc(n.name)} ${esc(n.kind)} ${esc(n.domainId)} ${esc(n.verification)}">`);
        parts.push(`<rect class="d2-box" x="${p.x}" y="${p.y}" width="${w}" height="${h}" rx="6" fill="var(--surface)" `
          + `stroke="${sel ? 'var(--focus)' : 'var(--edge)'}" stroke-width="${sel ? 2.4 : 1.1}"/>`);
        parts.push(`<rect x="${p.x}" y="${p.y}" width="4" height="${h}" rx="2" fill="${domColor}"/>`);
        parts.push(`<text x="${p.x + 12}" y="${p.y + 20}" font-size="11" fill="var(--text)" font-weight="600">${esc(n.name)}</text>`);
        parts.push(`<text x="${p.x + 12}" y="${p.y + 35}" font-size="9" fill="var(--text-2)">${esc(n.kind)} · ${esc(n.domainId)}</text>`);
        if (n.kind === 'Service') {
          parts.push(`<text x="${p.x + 12}" y="${p.y + 49}" font-size="8" fill="var(--text-2)">`
            + `${esc(String(n.properties.version ?? '—'))} · deployed=${n.properties.deployed === true}</text>`);
        } else if (n.kind === 'Table') {
          const cols = (n.properties.columns as unknown[])?.length ?? 0;
          parts.push(`<text x="${p.x + 12}" y="${p.y + 35}" font-size="9" fill="var(--text-2)">${cols} 列 · ${esc(n.domainId)}</text>`);
        }
        parts.push('</g>');
      }

      // 折叠的组节点
      for (const g of state.groups) {
        if (!state.collapsed.has(g.id)) continue;
        const b = boxOf(`group:${g.id}`);
        if (!b) continue;
        parts.push(`<g class="d2-node" data-node-id="group:${esc(g.id)}" tabindex="0" role="button" aria-pressed="false" `
          + `aria-label="折叠分组 ${esc(g.name)} ${g.nodeIds.length} 个成员">`);
        parts.push(`<path d="M ${b.x} ${b.y + 18} L ${b.x + 18} ${b.y} L ${b.x + b.w} ${b.y} L ${b.x + b.w} ${b.y + b.h} `
          + `L ${b.x + 18} ${b.y + b.h} Z" fill="var(--raised)" stroke="var(--focus)" stroke-width="1.6"/>`);
        parts.push(`<text x="${b.x + 28}" y="${b.y + 24}" font-size="11" fill="var(--text)" font-weight="600">${esc(g.name)}</text>`);
        parts.push(`<text x="${b.x + 28}" y="${b.y + 40}" font-size="9" fill="var(--text-2)">«group» ${g.nodeIds.length} 个成员 (点击展开)</text>`);
        parts.push('</g>');
      }

      // 端口 (独立可选对象)
      if (layerOn('port')) {
        for (const n of index.nodes.values()) {
          if (n.kind !== 'Service') continue;
          if (!visibleNode(n)) continue;
          const p = state.positions.get(n.id);
          if (!p) continue;
          const list = portsByCrate.get(crateOf(n)) ?? [];
          const expanded = state.selNodes.has(n.id) || state.selPorts.size > 0;
          const shown = expanded ? list : list.slice(0, 8);
          shown.forEach((port, i) => {
            const px = p.x + 6 + i * (PORT_SIZE + 2);
            const py = p.y + NODE_H.Service + 5;
            const sel = state.selPorts.has(port.id);
            parts.push(`<rect class="d2-port" data-port-id="${esc(port.id)}" x="${px}" y="${py}" `
              + `width="${PORT_SIZE}" height="${PORT_SIZE}" rx="1.5" fill="${sel ? 'var(--focus)' : 'var(--info)'}" `
              + `tabindex="0" role="button" aria-pressed="${sel}" `
              + `aria-label="端口 ${esc(port.name)} ${esc(port.requestType)} ${esc(port.responseType)}"/>`);
          });
          if (!expanded && list.length > shown.length) {
            parts.push(`<text x="${p.x + 6 + shown.length * (PORT_SIZE + 2)}" y="${p.y + NODE_H.Service + 12}" `
              + `font-size="8" fill="var(--text-2)">+${list.length - shown.length}</text>`);
          }
        }
      }

      parts.push('</g>');
      return parts.join('');
    }

    function isMatch(n: GraphNode): boolean {
      const q = state.search.trim().toLowerCase();
      if (!q) return false;
      return n.name.toLowerCase().includes(q)
        || n.qualifiedName.toLowerCase().includes(q)
        || String(n.properties.crate ?? '').toLowerCase().includes(q);
    }
    function isRelated(id: string): boolean {
      if (!state.focus) return true;
      if (id === state.focus) return true;
      for (const e of index.edges.values()) {
        if (e.sourceId === state.focus && e.targetId === id) return true;
        if (e.targetId === state.focus && e.sourceId === id) return true;
      }
      return false;
    }

    /* ── 渲染: 缩略图 ─────────────────────────────────────────── */
    function renderMinimap(): string {
      const all: Pos[] = [];
      for (const n of index.nodes.values()) {
        if (!visibleNode(n)) continue;
        const p = state.positions.get(n.id);
        if (p) all.push(p);
      }
      if (!all.length) return '<svg viewBox="0 0 190 130" aria-hidden="true"></svg>';
      const minX = Math.min(...all.map((p) => p.x)) - 40;
      const minY = Math.min(...all.map((p) => p.y)) - 40;
      const maxX = Math.max(...all.map((p) => p.x + 140)) + 40;
      const maxY = Math.max(...all.map((p) => p.y + 64)) + 40;
      const w = Math.max(1, maxX - minX);
      const h = Math.max(1, maxY - minY);
      const k = Math.min(190 / w, 130 / h);
      const ox = (190 - w * k) / 2;
      const oy = (130 - h * k) / 2;
      const parts: string[] = [`<svg viewBox="0 0 190 130" aria-hidden="true">`];
      for (const n of index.nodes.values()) {
        if (!visibleNode(n)) continue;
        const p = state.positions.get(n.id);
        if (!p) continue;
        const x = ox + (p.x - minX) * k;
        const y = oy + (p.y - minY) * k;
        parts.push(`<rect x="${x}" y="${y}" width="${Math.max(1.5, (NODE_W[n.kind] ?? 130) * k)}" `
          + `height="${Math.max(1.5, (NODE_H[n.kind] ?? 44) * k)}" fill="var(--dom-${n.domainId || 'Unknown'})" opacity="0.7"/>`);
      }
      const v = state.view;
      const vw = (stage.clientWidth || 800) / v.k;
      const vh = (stage.clientHeight || 600) / v.k;
      parts.push(`<rect x="${ox + (-v.x / v.k - minX) * k}" y="${oy + (-v.y / v.k - minY) * k}" `
        + `width="${vw * k}" height="${vh * k}" fill="none" stroke="var(--focus)" stroke-width="1.4"/>`);
      parts.push('</svg>');
      return parts.join('');
    }

    /* ── 渲染: 侧栏 ───────────────────────────────────────────── */
    function renderSidebar() {
      const q = state.search.trim().toLowerCase();
      const hits = q ? [...index.nodes.values()].filter(isMatch).slice(0, 40) : [];
      const domains = DOMAIN_ORDER.filter((d) => services.some((s) => s.domainId === d));
      const selCount = state.selNodes.size + state.selEdges.size + state.selPorts.size;

      sidebar.innerHTML = `
      <div class="d2-sect" open>
        <summary>上游: 1D Prompt 范围</summary>
        <div class="d2-sect-body" id="d2-prompt-scope">
          ${bus.snapshot.promptId
            ? `<div style="font-size:11px;color:var(--text-2);margin-bottom:5px">
                 <code>${esc(bus.snapshot.promptId)}</code><br>
                 ${esc(bus.snapshot.promptTitle ?? '')}<br>
                 范围 ${bus.snapshot.promptScopeNodeIds.size} 节点 / ${bus.snapshot.promptScopeEdgeIds.size} 边
               </div>
               <button class="d2-dg-btn" data-act="focus-prompt">聚焦 1D 范围</button>
               <button class="d2-dg-btn" data-act="only-prompt">仅显示 1D 范围</button>`
            : `<div style="font-size:11px;color:var(--text-2)">未选择 1D Prompt — 画布显示全部事实节点。<br>去 <strong>1D</strong> 选一条 Prompt, 这里会出现它的绘制范围。</div>`}
        </div>
      </div>

      <div class="d2-sect" open>
        <summary>画布操作</summary>
        <div class="d2-sect-body">
          <button class="d2-dg-btn" data-act="fit">适应全部 (0)</button>
          <button class="d2-dg-btn" data-act="focus">聚焦选中 (F)</button>
          <button class="d2-dg-btn" data-act="group">分组选中 (G)</button>
          <button class="d2-dg-btn" data-act="ungroup">取消分组</button>
          <button class="d2-dg-btn" data-act="collapse">折叠选中组</button>
          <button class="d2-dg-btn" data-act="expand">展开全部组</button>
          <button class="d2-dg-btn" data-act="clear">清空选择 (Esc)</button>
        </div>
      </div>

      <div class="d2-sect" open>
        <summary>搜索</summary>
        <div class="d2-sect-body">
          <input id="d2-search" type="search" placeholder="按名称 / crate 过滤…" value="${esc(state.search)}"
                 style="width:100%;height:26px;background:var(--raised);color:var(--text);border:1px solid var(--edge);
                        border-radius:6px;padding:0 6px;font-size:11px">
          ${hits.length ? `<ul class="d2-hitlist">${hits.map((n) => `
            <li><button data-goto="${esc(n.id)}">${esc(n.name)} <span style="color:var(--text-2)">${esc(n.kind)}</span></button></li>`).join('')}</ul>`
            : (q ? '<div class="empty-hint" style="padding:4px">无匹配节点</div>' : '')}
        </div>
      </div>

      <div class="d2-sect" open>
        <summary>图层</summary>
        <div class="d2-sect-body">
          ${LAYERS.map((l) => `
            <label class="d2-layer">
              <input type="checkbox" data-layer="${l.key}" ${layerOn(l.key) ? 'checked' : ''}>
              <span>${esc(l.label)}</span><span class="hint">${esc(l.hint)}</span>
            </label>`).join('')}
        </div>
      </div>

      <div class="d2-sect" open>
        <summary>过滤</summary>
        <div class="d2-sect-body">
          <select id="d2-domain" style="width:100%;height:26px;background:var(--raised);color:var(--text);
                  border:1px solid var(--edge);border-radius:6px;font-size:11px">
            <option value="">全部业务域 (${services.length} 服务)</option>
            ${domains.map((d) => `<option value="${d}" ${state.domainFilter === d ? 'selected' : ''}>${d}</option>`).join('')}
          </select>
        </div>
      </div>

      <div class="d2-sect">
        <summary>分组 (${state.groups.length})</summary>
        <div class="d2-sect-body">
          ${state.groups.length ? state.groups.map((g) => `
            <button class="d2-dg-btn" data-group="${esc(g.id)}" aria-current="${state.collapsed.has(g.id)}">
              «group» ${esc(g.name)} · ${g.nodeIds.length} 个${state.collapsed.has(g.id) ? ' (已折叠)' : ''}
            </button>`).join('') : '<div class="empty-hint" style="padding:2px">尚未分组 — 选中多个节点后按 G。</div>'}
        </div>
      </div>

      <div class="d2-sect" open>
        <summary>工程图 (${DIAGRAMS.length})</summary>
        <div class="d2-sect-body">
          ${DIAGRAMS.map((d) => `
            <button class="d2-dg-btn" data-diagram="${esc(d.type)}">«${esc(d.type)}» ${esc(d.label)}</button>`).join('')}
        </div>
      </div>

      <div class="d2-sect">
        <summary>选择</summary>
        <div class="d2-sect-body">
          <div style="color:var(--text-2);font-size:11px">
            节点 ${state.selNodes.size} · 边 ${state.selEdges.size} · 端口 ${state.selPorts.size}<br>
            撤销栈 ${state.undo.length} / ${UNDO_LIMIT} 步
          </div>
          ${state.selPorts.size ? `<ul class="d2-hitlist">${[...state.selPorts].map((pid) => {
            const p = index.ports.get(pid);
            return `<li><button data-goto-port="${esc(pid)}">${esc(p?.name ?? pid)}</button></li>`;
          }).join('')}</ul>` : ''}
        </div>
      </div>`;
    }

    /* ── 渲染: 工具条 ─────────────────────────────────────────── */
    function renderToolbar() {
      toolbar.innerHTML = `
        <button class="tool-btn" data-act="zoom-out" aria-label="缩小">−</button>
        <span class="tool-label" id="d2-zoom">${Math.round(state.view.k * 100)}%</span>
        <button class="tool-btn" data-act="zoom-in" aria-label="放大">+</button>
        <button class="tool-btn" data-act="fit">适应</button>
        <span class="tool-sep"></span>
        <button class="tool-btn" data-act="undo" ${state.undo.length ? '' : 'disabled'}>撤销 (${state.undo.length})</button>
        <button class="tool-btn" data-act="redo" ${state.redo.length ? '' : 'disabled'}>重做 (${state.redo.length})</button>
        <span class="tool-sep"></span>
        <button class="tool-btn" data-act="focus">聚焦</button>
        <button class="tool-btn" data-act="group">分组</button>
        <button class="tool-btn" data-act="collapse">折叠</button>
        <span class="tool-sep"></span>
        <button class="tool-btn" data-act="adapters">适配器报告</button>
        <button class="tool-btn" data-act="patch">受控 Patch</button>
        <button class="tool-btn" data-act="export-graph">导出 Graph JSON</button>
        <button class="tool-btn" data-act="export-workflow">导出 Workflow JSON</button>
        <span class="tool-sep"></span>
        <button class="tool-btn" data-act="demo" title="分步演示 2D 的用法 (降低认知负荷)">▶ 演示</button>`;
    }

    /* ── 全量渲染 ─────────────────────────────────────────────── */
    function render() {
      stage.innerHTML = `
        <svg class="d2-svg" id="d2-canvas" tabindex="0" role="application"
             aria-label="2D 无限画布 — 方向键平移, 加减号缩放, F 聚焦, G 分组, Ctrl+Z 撤销">
          ${renderCanvas()}
        </svg>
        <div class="d2-minimap" id="d2-minimap" aria-label="缩略图">${renderMinimap()}</div>
        <div class="d2-legend">
          <b>图例</b>
          <span>实线 = Implemented 真实调用点 · 黄色虚线 = Inferred 仅注释提及</span>
          <span>侧栏色条 = 业务域 · 蓝方块 = 端口 (点击独立选择)</span>
          <span>Shift+拖拽 = 框选 · 拖拽空白 = 平移 · 滚轮 = 以光标为锚点缩放</span>
        </div>`;
      renderToolbar();
      renderSidebar();
    }

    /* ── 检查器 + 关系与需求追踪 ──────────────────────────────── */
    function inspectNode(id: string) {
      const n = index.nodes.get(id);
      if (!n) return;
      ctx.inspect({ __kind: 'node', id });
      /* 2D → 3D の鎖: 選択したノードを公開する。3D はこれを受けてフォーカスする。 */
      bus.setSelection2d({ kind: 'node', id, origin: '2D' });
      const body = document.getElementById('inspector-body');
      if (!body) return;
      const reqs = (traceDoc.nodeToRequirement ?? []).filter((r) => r.nodeId === id);
      const cases = (traceDoc.caseToNode ?? []).filter((c) => c.nodeIds.includes(id));
      const outbox = (n.properties.childTableIds as string[]) ?? [];
      const box = document.createElement('div');
      box.className = 'panel';
      box.innerHTML = `
        <h3>关系与需求追踪 (2D)</h3>
        <dl class="kv">
          <dt>需求</dt><dd>${reqs.length ? reqs.map((r) => r.requirementIds.map((rid) => {
            const rq = reqById.get(rid);
            return `<div><code>${esc(rid)}</code> ${esc(rq?.intent ?? '')}</div>`;
          }).join('')) : '—'}</dd>
          <dt>用例</dt><dd>${cases.length ? cases.map((c) => {
            const cs = caseById.get(c.caseId);
            return `<div><code>${esc(c.caseId)}</code> ${esc(cs?.kind ?? '')} · ${esc(cs?.title ?? '')}</div>`;
          }).join('') : '—'}</dd>
          <dt>入边</dt><dd>${index.edges.size ? [...index.edges.values()].filter((e) => e.targetId === id).length : 0} 条实现 / `
            + `${[...index.designedEdges.values()].filter((e) => e.targetId === id).length} 条推断</dd>
          <dt>出边</dt><dd>${[...index.edges.values()].filter((e) => e.sourceId === id).length} 条实现 / `
            + `${[...index.designedEdges.values()].filter((e) => e.sourceId === id).length} 条推断</dd>
          <dt>子表</dt><dd>${outbox.length ? outbox.map((t) => `<div><code>${esc(t)}</code></div>`).join('') : '—'}</dd>
        </dl>`;
      body.appendChild(box);
    }
    function inspectEdge(id: string) {
      const e = index.edges.get(id) ?? index.designedEdges.get(id);
      if (!e) return;
      ctx.inspect({ __kind: 'edge', id });
      /* 2D → 3D の鎖: 辺の選択も公開する */
      bus.setSelection2d({ kind: 'edge', id, origin: '2D' });
      const body = document.getElementById('inspector-body');
      if (!body) return;
      const src = e.sourceId ? index.nodes.get(e.sourceId) : undefined;
      const dst = e.targetId ? index.nodes.get(e.targetId) : undefined;
      const box = document.createElement('div');
      box.className = 'panel';
      box.innerHTML = `
        <h3>边证据 (2D)</h3>
        <dl class="kv">
          <dt>fileLine</dt><dd><code>${esc(String(e.properties.fileLine ?? '—'))}</code></dd>
          <dt>verification</dt><dd><span class="badge badge--${esc(e.verification)}">${esc(e.verification)}</span></dd>
          <dt>source</dt><dd>${esc(src?.name ?? '—')} <code>${esc(e.sourceId ?? '—')}</code></dd>
          <dt>target</dt><dd>${esc(dst?.name ?? '—')} <code>${esc(e.targetId ?? '—')}</code></dd>
          <dt>usageForm</dt><dd>${esc(String(e.properties.usageForm ?? '—'))}</dd>
          <dt>note</dt><dd>${esc(String(e.properties.note ?? '—'))}</dd>
        </dl>
        <p class="empty-hint">${e.verification === 'Implemented'
          ? '实线 = 真实调用点, 可独立复核 fileLine。'
          : '虚线 = 仅注释/文档提及, 采集时未发现真实调用点。'}</p>`;
      body.appendChild(box);
    }
    function inspectPort(id: string) {
      ctx.inspect({ __kind: 'port', id });
      const body = document.getElementById('inspector-body');
      if (!body) return;
      const p = index.ports.get(id);
      if (!p) return;
      const box = document.createElement('div');
      box.className = 'panel';
      box.innerHTML = `
        <h3>端口归属 (2D)</h3>
        <dl class="kv">
          <dt>crate</dt><dd><code>${esc(p.crate)}</code></dd>
          <dt>apiId</dt><dd>${esc(p.apiId ?? 'null (采集模型中端口未绑定到具体 proto service)')}</dd>
          <dt>direction</dt><dd>${esc(p.direction)}</dd>
          <dt>type</dt><dd>${esc(p.type)}</dd>
        </dl>`;
      body.appendChild(box);
    }

    /* ── 浮窗 (可分离审查) ────────────────────────────────────── */
    const floatLayer = document.createElement('div');
    floatLayer.className = 'd2-float-layer';
    floatLayer.id = 'd2-float-layer';
    document.body.appendChild(floatLayer);
    let fwSeq = 0;

    function selectionForDiagrams(): DiagramSelection {
      return {
        nodeIds: [...state.selNodes],
        edgeIds: [...state.selEdges],
        focusId: state.focus ?? (state.selNodes.size === 1 ? [...state.selNodes][0] : null),
      };
    }
    function diagramContext(): DiagramContext {
      return {
        model: app.model,
        index,
        selection: selectionForDiagrams(),
        showInferred: layerOn('edgeInferred'),
        aux,
      };
    }

    function openDiagram(type: DiagramType) {
      const renderer = rendererFor(type);      // 严格按 diagramType 分派
      let result;
      try {
        result = renderer.render(diagramContext());
      } catch (err) {
        ctx.status(`图 ${type} 渲染失败: ${(err as Error).message}`);
        return;
      }
      fwSeq += 1;
      const id = `d2fw${fwSeq}`;
      const w = Math.min(1000, Math.max(560, result.width + 40));
      const h = Math.min(760, Math.max(380, result.height + 120));
      /* 図窓の初期位置は**左サイドバーを避けて**使う。
       * 以前 left=80 固定でサイドバー (256px) を覆い、サイドバーの
       * 「次の図種」ボタンが押せなかった (対抗審査の指摘)。 */
      const wide = window.innerWidth > 1023;
      const baseLeft = wide ? 272 : 16;
      const baseTop = wide ? 70 : 56;
      const left = baseLeft + ((fwSeq - 1) % 4) * 46;
      const top = baseTop + ((fwSeq - 1) % 4) * 40;
      const el = document.createElement('div');
      el.className = 'd2-fw';
      el.id = id;
      el.style.left = `${left}px`;
      el.style.top = `${top}px`;
      el.style.width = `${w}px`;
      el.style.height = `${h}px`;
      el.innerHTML = `
        <div class="d2-fw-head" data-drag="1">
          <strong>«${esc(type)}» ${esc(renderer.label)}</strong>
          <span class="sp"></span>
          <button class="d2-fw-btn" data-z="out" aria-label="缩小">−</button>
          <button class="d2-fw-btn" data-z="fit" aria-label="适应宽度">适应</button>
          <button class="d2-fw-btn" data-z="in" aria-label="放大">+</button>
          <button class="d2-fw-btn" data-close aria-label="关闭">✕</button>
        </div>
        <div class="d2-fw-body"><div class="d2-fw-inner" data-inner>${result.svg}</div></div>
        <div class="d2-fw-note">
          <b>${esc(result.summary)}</b><br>
          符号: ${esc(renderer.symbols.join(' / '))}<br>
          数据: ${esc(renderer.dataSource)}<br>
          ${result.notes.map((n) => `· ${esc(n)}`).join('<br>')}
        </div>`;
      floatLayer.appendChild(el);

      const inner = el.querySelector('[data-inner]') as HTMLElement;
      const body = el.querySelector('.d2-fw-body') as HTMLElement;
      let k = 1;
      const applyK = () => {
        inner.style.transform = `scale(${k})`;
        inner.style.width = `${result.width * k}px`;
        inner.style.height = `${result.height * k}px`;
      };
      applyK();
      const fit = () => {
        k = Math.min(1.6, Math.max(0.2, (body.clientWidth - 20) / Math.max(1, result.width)));
        applyK();
      };
      el.querySelector('[data-z="in"]')?.addEventListener('click', () => { k = Math.min(4, k * 1.25); applyK(); });
      el.querySelector('[data-z="out"]')?.addEventListener('click', () => { k = Math.max(0.1, k / 1.25); applyK(); });
      el.querySelector('[data-z="fit"]')?.addEventListener('click', fit);
      el.querySelector('[data-close]')?.addEventListener('click', () => el.remove());

      // 头部拖拽移动
      const head = el.querySelector('[data-drag]') as HTMLElement;
      head.addEventListener('pointerdown', (ev) => {
        if ((ev.target as HTMLElement).closest('button')) return;
        const sx = ev.clientX - el.offsetLeft;
        const sy = ev.clientY - el.offsetTop;
        const move = (e2: PointerEvent) => {
          el.style.left = `${Math.max(0, e2.clientX - sx)}px`;
          el.style.top = `${Math.max(0, e2.clientY - sy)}px`;
        };
        const up = () => {
          head.removeEventListener('pointermove', move);
          window.removeEventListener('pointerup', up);
        };
        head.addEventListener('pointermove', move);
        window.addEventListener('pointerup', up);
      });
      body.addEventListener('wheel', (ev) => {
        if (!ev.ctrlKey && !ev.altKey) return;
        ev.preventDefault();
        k = Math.min(4, Math.max(0.1, k * (ev.deltaY < 0 ? 1.1 : 0.9)));
        applyK();
      }, { passive: false });

      ctx.status(`已打开工程图 «${type}» (${result.summary}) — 可同时打开多个对比`);
      return el;
    }

    /* ── 适配器报告 (支持子集 + 语义损失) ─────────────────────── */
    function adapterReport(): string {
      const n = index.nodes.size;
      const e = index.edges.size + index.designedEdges.size;
      const tables = tableNodesCount();
      const pk = countPk();
      const fk = countFk();
      const ports = index.ports.size;
      const rules = aux.rules.length;
      const cases = aux.cases.length;
      return `
      <p class="empty-hint" style="padding:0 0 8px">
        本图集导出的 Graph JSON (<code>${GRAPH_SCHEMA}</code>) 相对各适配器的支持子集与语义损失。
        以下数字由当前模型实时统计, 不是写死的说明文字。
      </p>
      <table class="data">
        <thead><tr><th>适配器</th><th>支持子集</th><th>语义损失</th><th>保留策略</th></tr></thead>
        <tbody>
          <tr>
            <td><b>React Flow</b></td>
            <td>node/edge 图 (${n} 节点 / ${e} 边), 位置, 选择态, 分组可映射为 parentId</td>
            <td>丢失 UML compartment 语义, ER 基数/菱形, 时序生命线与激活条, BPMN 池/泳道/事件/网关, 信任边界, PK/FK 视觉, 端口类型, verification 的语义 (仅能存为自定义字段)</td>
            <td>边样式 (实/虚) + 标签 + evidenceIds 作为 custom data 保留; 语义损失在 adapter 报告中登记</td>
          </tr>
          <tr>
            <td><b>tldraw</b></td>
            <td>自由画布 + 任意 shape/arrow, 适合作为 2D 无限画布交换格式</td>
            <td>无类型化节点/边语义, 无基数/guard/生命线, 端口不可寻址; 形状需按 diagramType 再解释</td>
            <td>位置 + 分组 + 选择态可无损交换; 13 种 UML/ER/BPMN 语义全部降级为图形约定</td>
          </tr>
          <tr>
            <td><b>BPMN 2.0</b></td>
            <td>仅 BPMN 子集: 池/泳道 (${cases} 用例 / ${rules} 规则), 任务 (${e} 条调用边), 排他网关 (when guard), 开始/结束事件</td>
            <td>UML_Class/Component/Package/Sequence/State/Deployment/UseCase/ER/Security/Network/DataFlow/DFD 全部超出 BPMN 能力; 端口 (${ports}) 与 verification 区分无对应元素</td>
            <td>BPMN 导出只承诺任务/网关/事件/池四类; 其余 10 种图种标记为 lossy 并给出损失清单</td>
          </tr>
          <tr>
            <td><b>本图集原生</b></td>
            <td>全部 14 种图种 + ${tables} 张表 (${pk} 个 PK / ${fk} 个 FK) + ${ports} 个端口 + 真实 fileLine 证据</td>
            <td>无 — 这是保真源, 其他适配器均由它派生</td>
            <td>schemaVersion = <code>${GRAPH_SCHEMA}</code> / <code>${WORKFLOW_SCHEMA}</code>, 每次导出写入</td>
          </tr>
        </tbody>
      </table>
      <h4>版本化 Schema</h4>
      <dl class="kv">
        <dt>Graph</dt><dd><code>${GRAPH_SCHEMA}</code> — nodes / edges / ports / groups / layout / verification</dd>
        <dt>Workflow</dt><dd><code>${WORKFLOW_SCHEMA}</code> — processes / rules / cases / guards</dd>
        <dt>Patch</dt><dd><code>${PATCH_SCHEMA}</code> — AI 只能通过白名单 op 修改画布状态</dd>
      </dl>
      <h4>受控 Patch (AI 修改入口)</h4>
      <p class="empty-hint" style="padding:0 0 6px">
        AI 不直接写画布, 只能提交受控 Patch: op 白名单 + 目标 ID 必须存在 + 坐标必须为有限数。
        任一条不满足则整批拒绝, 不做部分应用。
      </p>
      <pre style="margin:0;padding:8px;background:var(--raised);border-radius:6px;font:11px/16px var(--mono);overflow-x:auto">{
  "schemaVersion": "${PATCH_SCHEMA}",
  "ops": [
    { "op": "setPosition", "nodeId": "N-136-svc-economy-service", "x": 100, "y": 200 },
    { "op": "group", "groupId": "g1", "name": "经济域", "nodeIds": ["N-136-svc-economy-service"] },
    { "op": "collapse", "groupId": "g1" },
    { "op": "setLayer", "layer": "Api", "visible": false }
  ]
}</pre>
      <p class="empty-hint" style="padding:6px 0 0">白名单 op: setPosition / group / ungroup / collapse / expand / setLayer。</p>`;
    }
    function tableNodesCount() { return [...index.nodes.values()].filter((n) => n.kind === 'Table').length; }
    function countPk() {
      let c = 0;
      for (const n of index.nodes.values()) {
        if (n.kind !== 'Table') continue;
        const cols = (n.properties.columns as { pk?: boolean }[]) ?? [];
        c += cols.filter((x) => x.pk).length;
      }
      return c;
    }
    function countFk() {
      let c = 0;
      for (const n of index.nodes.values()) {
        if (n.kind !== 'Table') continue;
        const cols = (n.properties.columns as { fk?: string | null }[]) ?? [];
        c += cols.filter((x) => !!x.fk).length;
      }
      return c;
    }

    function openModal(title: string, bodyHtml: string, withFoot = false) {
      const root = document.getElementById('modal-root')!;
      root.hidden = false;
      root.innerHTML = `
        <div class="modal" role="dialog" aria-modal="true" aria-label="${esc(title)}"
             style="max-width:min(1040px,94vw)">
          <div class="modal-head"><strong>${esc(title)}</strong>
            <button class="icon-btn" data-close aria-label="关闭">✕</button></div>
          <div class="modal-body">${bodyHtml}</div>
        </div>`;
      const close = () => { root.hidden = true; root.innerHTML = ''; };
      root.querySelector('[data-close]')?.addEventListener('click', close);
      return { root, close };
    }

    /* ── 受控 Patch 应用 ──────────────────────────────────────── */
    function applyPatch(raw: string): { ok: boolean; msg: string } {
      let doc: { schemaVersion?: string; ops?: unknown[] };
      try {
        doc = JSON.parse(raw);
      } catch (err) {
        return { ok: false, msg: `JSON 解析失败: ${(err as Error).message}` };
      }
      if (doc.schemaVersion !== PATCH_SCHEMA) {
        return { ok: false, msg: `schemaVersion 必须为 ${PATCH_SCHEMA}, 收到 ${String(doc.schemaVersion)}` };
      }
      const ops = Array.isArray(doc.ops) ? doc.ops : [];
      if (!ops.length) return { ok: false, msg: 'ops 为空' };

      // 先全量校验 (不满足即整批拒绝)
      for (const o of ops) {
        const op = o as Record<string, unknown>;
        const kind = String(op.op ?? '');
        if (!['setPosition', 'group', 'ungroup', 'collapse', 'expand', 'setLayer'].includes(kind)) {
          return { ok: false, msg: `未知 op "${kind}" (不在白名单内)` };
        }
        if (kind === 'setPosition') {
          if (!index.nodes.has(String(op.nodeId))) return { ok: false, msg: `setPosition: 节点不存在 ${String(op.nodeId)}` };
          if (!Number.isFinite(Number(op.x)) || !Number.isFinite(Number(op.y))) {
            return { ok: false, msg: 'setPosition: x / y 必须是有限数' };
          }
        }
        if (kind === 'group') {
          const ids = (op.nodeIds as string[]) ?? [];
          if (!ids.length) return { ok: false, msg: 'group: nodeIds 为空' };
          for (const nid of ids) if (!index.nodes.has(nid)) return { ok: false, msg: `group: 节点不存在 ${nid}` };
          if (!String(op.groupId ?? '')) return { ok: false, msg: 'group: 缺 groupId' };
        }
        if (['ungroup', 'collapse', 'expand'].includes(kind)) {
          const gid = String(op.groupId ?? '');
          if (!state.groups.some((g) => g.id === gid)) return { ok: false, msg: `${kind}: 分组不存在 ${gid}` };
        }
        if (kind === 'setLayer') {
          if (!LAYERS.some((l) => l.key === String(op.layer))) return { ok: false, msg: `setLayer: 图层不存在 ${String(op.layer)}` };
        }
      }

      // 校验通过 → 作为一次撤销步骤整体应用
      commit('受控 Patch');
      for (const o of ops) {
        const op = o as Record<string, unknown>;
        const kind = String(op.op);
        if (kind === 'setPosition') {
          state.positions.set(String(op.nodeId), { x: Number(op.x), y: Number(op.y) });
        } else if (kind === 'group') {
          const gid = String(op.groupId);
          state.groups.push({ id: gid, name: String(op.name ?? gid), nodeIds: [...(op.nodeIds as string[])] });
        } else if (kind === 'ungroup') {
          const gid = String(op.groupId);
          state.groups = state.groups.filter((g) => g.id !== gid);
          state.collapsed.delete(gid);
        } else if (kind === 'collapse') {
          state.collapsed.add(String(op.groupId));
        } else if (kind === 'expand') {
          state.collapsed.delete(String(op.groupId));
        } else if (kind === 'setLayer') {
          const key = String(op.layer);
          if (op.visible === false) state.hidden.add(key); else state.hidden.delete(key);
        }
      }
      render();
      return { ok: true, msg: `已应用 ${ops.length} 个 op (作为 1 步撤销)` };
    }

    /* ── 导出 ─────────────────────────────────────────────────── */
    function download(name: string, obj: unknown) {
      const blob = new Blob([JSON.stringify(obj, null, 2)], { type: 'application/json' });
      const a = document.createElement('a');
      a.href = URL.createObjectURL(blob);
      a.download = name;
      a.click();
      setTimeout(() => URL.revokeObjectURL(a.href), 1000);
      ctx.status(`已导出 ${name}`);
    }
    function exportGraph() {
      download('rgs-graph.json', {
        schemaVersion: GRAPH_SCHEMA,
        generatedAt: new Date().toISOString(),
        source: { revision: app.model.meta.shortRevision, branch: app.model.meta.branch },
        nodes: [...index.nodes.values()].map((n) => ({
          id: n.id, kind: n.kind, name: n.name, domainId: n.domainId, verification: n.verification,
          position: state.positions.get(n.id) ?? null, properties: n.properties, evidenceIds: n.evidenceIds,
        })),
        edges: [...index.edges.values(), ...index.designedEdges.values()].map((e) => ({
          id: e.id, sourceId: e.sourceId, targetId: e.targetId, type: e.type,
          verification: e.verification, properties: e.properties, evidenceIds: e.evidenceIds,
        })),
        ports: [...index.ports.values()].map((p) => ({
          id: p.id, name: p.name, crate: p.crate, requestType: p.requestType, responseType: p.responseType,
          direction: p.direction, type: p.type,
        })),
        groups: state.groups,
        layout: { view: state.view },
      });
    }
    function exportWorkflow() {
      download('rgs-workflow.json', {
        schemaVersion: WORKFLOW_SCHEMA,
        generatedAt: new Date().toISOString(),
        source: { revision: app.model.meta.shortRevision },
        processes: services.map((s) => ({
          id: s.id, name: s.name, domainId: s.domainId, crate: s.properties.crate,
          deployed: s.properties.deployed === true, verification: s.verification,
        })),
        edges: [...index.edges.values(), ...index.designedEdges.values()].map((e) => ({
          id: e.id, from: e.sourceId, to: e.targetId, verification: e.verification, fileLine: e.properties.fileLine,
        })),
        rules: aux.rules,
        cases: aux.cases,
        requirements: aux.requirements.map((r) => ({ id: r.id, intent: r.intent, status: r.status })),
      });
    }

    /* ── 交互: 事件委托 (画布) ────────────────────────────────── */
    function toWorld(ev: { clientX: number; clientY: number }) {
      const r = stage.getBoundingClientRect();
      return {
        x: (ev.clientX - r.left - state.view.x) / state.view.k,
        y: (ev.clientY - r.top - state.view.y) / state.view.k,
      };
    }

    let dragMode: 'none' | 'pan' | 'move' | 'band' = 'none';
    let dragStart = { x: 0, y: 0 };
    let moved = false;
    let pendingSnap: Snapshot | null = null;

    stage.addEventListener('pointerdown', (ev) => {
      const target = ev.target as HTMLElement;
      const svg = target.closest('svg') as SVGSVGElement | null;
      if (!svg) return;
      moved = false;
      if (ev.button !== 0) return;

      const nodeEl = target.closest('[data-node-id]') as HTMLElement | null;
      const edgeEl = target.closest('[data-edge-id]') as HTMLElement | null;
      const portEl = target.closest('[data-port-id]') as HTMLElement | null;

      if (portEl) {
        const pid = portEl.dataset.portId!;
        if (!ev.shiftKey) state.selPorts.clear();
        state.selPorts.add(pid);
        state.selNodes.clear();
        state.selEdges.clear();
        inspectPort(pid);
        render();
        return;
      }
      if (edgeEl) {
        const eid = edgeEl.dataset.edgeId!;
        if (!ev.shiftKey) { state.selEdges.clear(); state.selNodes.clear(); state.selPorts.clear(); }
        state.selEdges.add(eid);
        state.selNodes.clear();
        state.selPorts.clear();
        inspectEdge(eid);
        render();
        return;
      }
      if (nodeEl) {
        const nid = nodeEl.dataset.nodeId!;
        if (nid.startsWith('group:')) {
          const gid = nid.slice(6);
          commit('展开分组');
          state.collapsed.delete(gid);
          render();
          ctx.status(`已展开分组 ${gid}`);
          return;
        }
        if (ev.shiftKey) {
          if (state.selNodes.has(nid)) state.selNodes.delete(nid); else state.selNodes.add(nid);
        } else if (!state.selNodes.has(nid)) {
          state.selNodes.clear();
          state.selEdges.clear();
          state.selPorts.clear();
          state.selNodes.add(nid);
        }
        state.focus = state.selNodes.size === 1 ? nid : state.focus;
        inspectNode(nid);
        dragMode = 'move';
        dragStart = toWorld(ev);
        render();
        return;
      }
      // 空白
      if (ev.shiftKey) {
        dragMode = 'band';
        dragStart = toWorld(ev);
        state.bandBox = { x: dragStart.x, y: dragStart.y, w: 0, h: 0 };
      } else {
        dragMode = 'pan';
        dragStart = { x: ev.clientX, y: ev.clientY };
        stage.querySelector('.d2-svg')?.classList.add('is-panning');
      }
    });

    stage.addEventListener('pointermove', (ev) => {
      if (dragMode === 'none') return;
      moved = true;
      if (dragMode === 'pan') {
        state.view.x += ev.clientX - dragStart.x;
        state.view.y += ev.clientY - dragStart.y;
        dragStart = { x: ev.clientX, y: ev.clientY };
        render();
        return;
      }
      if (dragMode === 'move' && state.selNodes.size) {
        const w = toWorld(ev);
        const dx = w.x - dragStart.x;
        const dy = w.y - dragStart.y;
        for (const id of state.selNodes) {
          const p = state.positions.get(id);
          if (!p) continue;
          state.positions.set(id, { x: p.x + dx, y: p.y + dy });
        }
        dragStart = w;
        render();
        return;
      }
      if (dragMode === 'band' && state.bandBox !== null) {
        const w = toWorld(ev);
        state.bandBox = {
          x: Math.min(dragStart.x, w.x), y: Math.min(dragStart.y, w.y),
          w: Math.abs(w.x - dragStart.x), h: Math.abs(w.y - dragStart.y),
        };
        render();
      }
    });

    const endDrag = (ev: PointerEvent) => {
      if (dragMode === 'band' && state.bandBox) {
        const bandBox = state.bandBox;
        const ids: string[] = [];
        for (const n of index.nodes.values()) {
          if (!visibleNode(n)) continue;
          const p = state.positions.get(n.id);
          if (!p) continue;
          const w = NODE_W[n.kind] ?? 130;
          const h = NODE_H[n.kind] ?? 44;
          if (p.x >= bandBox.x && p.x + w <= bandBox.x + bandBox.w && p.y >= bandBox.y && p.y + h <= bandBox.y + bandBox.h) {
            ids.push(n.id);
          }
        }
        if (ids.length) {
          commit('框选');
          state.selNodes = new Set(ids);
          state.selEdges.clear();
          state.selPorts.clear();
          if (ids.length === 1) inspectNode(ids[0]);
          ctx.status(`框选 ${ids.length} 个节点`);
        }
        state.bandBox = null;
      } else if (dragMode === 'move' && moved) {
        commit('移动节点');
        ctx.status(`移动 ${state.selNodes.size} 个节点 (可撤销)`);
      }
      dragMode = 'none';
      stage.querySelector('.d2-svg')?.classList.remove('is-panning');
    };
    stage.addEventListener('pointerup', endDrag);
    stage.addEventListener('pointercancel', endDrag);

    stage.addEventListener('wheel', (ev) => {
      ev.preventDefault();
      const r = stage.getBoundingClientRect();
      const cx = ev.clientX - r.left;
      const cy = ev.clientY - r.top;
      const k2 = Math.max(0.08, Math.min(3.2, state.view.k * (ev.deltaY < 0 ? 1.12 : 1 / 1.12)));
      // 以光标为锚点: 保持光标下的世界坐标不变
      state.view.x = cx - ((cx - state.view.x) / state.view.k) * k2;
      state.view.y = cy - ((cy - state.view.y) / state.view.k) * k2;
      state.view.k = k2;
      render();
    }, { passive: false });

    // 键盘可达
    stage.addEventListener('keydown', (ev) => {
      const k = ev.key;
      if (ev.ctrlKey || ev.metaKey) {
        if (k === 'z' && !ev.shiftKey) { ev.preventDefault(); undo(); return; }
        if ((k === 'z' && ev.shiftKey) || k === 'y') { ev.preventDefault(); redo(); return; }
        return;
      }
      const pan = 40;
      if (k === 'ArrowLeft') { state.view.x += pan; render(); ev.preventDefault(); }
      else if (k === 'ArrowRight') { state.view.x -= pan; render(); ev.preventDefault(); }
      else if (k === 'ArrowUp') { state.view.y += pan; render(); ev.preventDefault(); }
      else if (k === 'ArrowDown') { state.view.y -= pan; render(); ev.preventDefault(); }
      else if (k === '+' || k === '=') { state.view.k = Math.min(3.2, state.view.k * 1.15); render(); ev.preventDefault(); }
      else if (k === '-' || k === '_') { state.view.k = Math.max(0.08, state.view.k / 1.15); render(); ev.preventDefault(); }
      else if (k === '0') { fitAll(); ev.preventDefault(); }
      else if (k === 'f' || k === 'F') { focusSelection(); ev.preventDefault(); }
      else if (k === 'g' || k === 'G') { groupSelection(); ev.preventDefault(); }
      else if (k === 'Escape') {
        state.selNodes.clear(); state.selEdges.clear(); state.selPorts.clear(); state.focus = null;
        render(); ev.preventDefault();
      }
    });
    stage.addEventListener('click', (ev) => {
      const t = ev.target as HTMLElement;
      const goto = t.closest('[data-goto]') as HTMLElement | null;
      if (goto) { centerOn(goto.dataset.goto!); return; }
      const gp = t.closest('[data-goto-port]') as HTMLElement | null;
      if (gp) {
        const pid = gp.dataset.gotoPort!;
        state.selPorts.clear(); state.selPorts.add(pid);
        const p = index.ports.get(pid);
        if (p) {
          const svc = services.find((s) => crateOf(s) === p.crate);
          if (svc) { state.selNodes.clear(); state.selNodes.add(svc.id); }
        }
        inspectPort(pid);
        render();
      }
    });

    /* ── 工具条 / 侧栏 事件 (委托) ───────────────────────────── */
    /* 工具条と侧栏は**外壳の常驻要素** (app.ts の go() は innerHTML を消すだけ)。
       そのため removeEventListener しないと 2D に入る度に登録が積み上がる
       (実測: 13 回 mount → 1 クリックで 13 回描画)。
       AbortController で一括解除できるようする。 */
    const shellListeners = new AbortController();
    const shellOpts = { signal: shellListeners.signal };
    /** 外殻常驻要素へのリスナ登録 (cleanup で一括解除できるようする) */
    const onShell = <K extends keyof HTMLElementEventMap>(
      el: HTMLElement, type: K, fn: (ev: HTMLElementEventMap[K]) => void,
    ) => { el.addEventListener(type, fn as EventListener, shellOpts); };
    /* ── 1D Prompt 范围 (1D → 2D) ─────────────────────────── */
    function focusPromptScope(): void {
      const scope = bus.snapshot.promptScopeNodeIds;
      if (!scope.size) { ctx.status('1D 未选择 Prompt, 无范围'); return; }
      state.selNodes = new Set(scope);
      state.focus = null;
      commit('聚焦 1D 范围');
      fitAll();
      ctx.status(`已聚焦 1D 范围 ${scope.size} 节点`);
    }
    function togglePromptOnly(): void {
      const scope = bus.snapshot.promptScopeNodeIds;
      if (!scope.size) { ctx.status('1D 未选择 Prompt, 无范围'); return; }
      state.promptOnly = !state.promptOnly;
      commit(state.promptOnly ? '仅显示 1D 范围' : '显示全部节点');
      fitAll();
      ctx.status(state.promptOnly ? `仅显示 1D 范围 ${scope.size} 节点` : '恢复显示全部节点');
    }

    /* ── デモツアー (認知負荷低減の導線) ─────────────────────── */
    let stopTour: (() => void) | null = null;
    function startTour(): void {
      stopTour?.();
      stopTour = startDemoTour(stage, {
        focusNode: (id) => {
          state.selNodes = new Set([id]);
          state.focus = id;
          inspectNode(id);
          centerOn(id);
          render();
        },
        openDiagram: (t) => openDiagram(t as DiagramType),
        setLayer: (k, v) => {
          if (v) state.hidden.delete(k); else state.hidden.add(k);
          render();
        },
        setSearch: (q) => { state.search = q; render(); },
        selectedNodeId: () => (state.selNodes.size === 1 ? [...state.selNodes][0] : null),
        stats: () => ({
          nodes: index.nodes.size,
          edges: index.edges.size + index.designedEdges.size,
          ports: index.ports.size,
          diagrams: DIAGRAMS.length,
        }),
      });
      ctx.status('演示开始 — 用「次へ」推进, Esc 退出');
    }

    onShell(toolbar, 'click', (ev) => {
      const btn = (ev.target as HTMLElement).closest('[data-act]') as HTMLElement | null;
      if (!btn) return;
      const act = btn.dataset.act!;
      if (act === 'zoom-in') { state.view.k = Math.min(3.2, state.view.k * 1.15); render(); }
      else if (act === 'zoom-out') { state.view.k = Math.max(0.08, state.view.k / 1.15); render(); }
      else if (act === 'fit') fitAll();
      else if (act === 'undo') undo();
      else if (act === 'redo') redo();
      else if (act === 'focus') focusSelection();
      else if (act === 'group') groupSelection();
      else if (act === 'collapse') collapseSelection();
      else if (act === 'adapters') {
        const m = openModal('适配器支持子集与语义损失报告', adapterReport());
        void m;
      } else if (act === 'patch') openPatchModal();
      else if (act === 'export-graph') exportGraph();
      else if (act === 'export-workflow') exportWorkflow();
      else if (act === 'demo') startTour();
    });

    onShell(sidebar, 'click', (ev) => {
      const t = ev.target as HTMLElement;
      const dg = t.closest('[data-diagram]') as HTMLElement | null;
      if (dg) { openDiagram(dg.dataset.diagram as DiagramType); return; }
      const grp = t.closest('[data-group]') as HTMLElement | null;
      if (grp) {
        const gid = grp.dataset.group!;
        commit('折叠/展开分组');
        if (state.collapsed.has(gid)) { state.collapsed.delete(gid); } else { state.collapsed.add(gid); }
        render();
        return;
      }
      const act = t.closest('[data-act]') as HTMLElement | null;
      if (!act) return;
      const a = act.dataset.act!;
      if (a === 'focus-prompt') focusPromptScope();
      else if (a === 'only-prompt') togglePromptOnly();
      else if (a === 'fit') fitAll();
      else if (a === 'focus') focusSelection();
      else if (a === 'group') groupSelection();
      else if (a === 'ungroup') ungroupSelection();
      else if (a === 'collapse') collapseSelection();
      else if (a === 'expand') { state.collapsed.clear(); commit('展开全部'); render(); }
      else if (a === 'clear') {
        state.selNodes.clear(); state.selEdges.clear(); state.selPorts.clear(); state.focus = null;
        render();
      }
    });
    onShell(sidebar, 'change', (ev) => {
      const t = ev.target as HTMLElement;
      if (t.matches('[data-layer]')) {
        const key = t.dataset.layer!;
        commit('图层开关');
        if ((t as HTMLInputElement).checked) state.hidden.delete(key); else state.hidden.add(key);
        render();
        return;
      }
      if (t.id === 'd2-domain') {
        state.domainFilter = (t as HTMLSelectElement).value;
        render();
      }
    });
    onShell(sidebar, 'input', (ev) => {
      const t = ev.target as HTMLInputElement;
      if (t.id === 'd2-search') {
        state.search = t.value;
        render();
        // 保持输入框焦点与光标
        const again = sidebar.querySelector('#d2-search') as HTMLInputElement | null;
        if (again) { again.focus(); again.setSelectionRange(t.value.length, t.value.length); }
      }
    });

    /* ── 动作实现 ─────────────────────────────────────────────── */
    function fitAll() {
      const ps = [...state.positions.values()];
      if (!ps.length) return;
      const minX = Math.min(...ps.map((p) => p.x)) - 60;
      const minY = Math.min(...ps.map((p) => p.y)) - 60;
      const maxX = Math.max(...ps.map((p) => p.x + 140)) + 60;
      const maxY = Math.max(...ps.map((p) => p.y + 70)) + 60;
      const w = stage.clientWidth || 900;
      const h = stage.clientHeight || 600;
      state.view.k = Math.max(0.08, Math.min(1.2, Math.min(w / (maxX - minX), h / (maxY - minY))));
      state.view.x = -minX * state.view.k + 20;
      state.view.y = -minY * state.view.k + 20;
      render();
      ctx.status(`适应全部 — 缩放 ${Math.round(state.view.k * 100)}%`);
    }
    function centerOn(nodeId: string) {
      const p = state.positions.get(nodeId);
      if (!p) return;
      state.view.x = stage.clientWidth / 2 - (p.x + 70) * state.view.k;
      state.view.y = stage.clientHeight / 2 - (p.y + 30) * state.view.k;
      state.selNodes.clear();
      state.selNodes.add(nodeId);
      state.focus = nodeId;
      inspectNode(nodeId);
      render();
    }
    function focusSelection() {
      const id = state.focus ?? (state.selNodes.size === 1 ? [...state.selNodes][0] : null);
      if (!id) { ctx.status('请先选中一个节点再聚焦'); return; }
      state.focus = id;
      centerOn(id);
      ctx.status(`聚焦 ${id} — 非邻接节点已淡化`);
    }
    function groupSelection() {
      if (state.selNodes.size < 2) { ctx.status('分组需要至少选中 2 个节点'); return; }
      const gid = `g${state.groups.length + 1}`;
      commit('分组');
      state.groups.push({ id: gid, name: `分组 ${state.groups.length + 1}`, nodeIds: [...state.selNodes] });
      render();
      ctx.status(`已分组 ${state.selNodes.size} 个节点 → ${gid}`);
    }
    function ungroupSelection() {
      if (!state.groups.length) { ctx.status('没有可取消的分组'); return; }
      const gid = state.groups[state.groups.length - 1].id;
      commit('取消分组');
      state.groups = state.groups.filter((g) => g.id !== gid);
      state.collapsed.delete(gid);
      render();
      ctx.status(`已取消分组 ${gid}`);
    }
    function collapseSelection() {
      if (!state.groups.length) { ctx.status('没有可折叠的分组'); return; }
      const gid = state.groups[state.groups.length - 1].id;
      commit('折叠分组');
      state.collapsed.add(gid);
      render();
      ctx.status(`已折叠 ${gid} (成员节点已收起为组节点)`);
    }

    function openPatchModal() {
      const sample = JSON.stringify({
        schemaVersion: PATCH_SCHEMA,
        ops: [{ op: 'setPosition', nodeId: services[0]?.id ?? '', x: 120, y: 160 }],
      }, null, 2);
      const m = openModal('受控 Patch (AI 修改入口)', `
        <p class="empty-hint" style="padding:0 0 8px">
          AI 只能通过白名单 op 修改画布状态。校验不通过则整批拒绝, 不做部分应用。
        </p>
        <textarea id="d2-patch" spellcheck="false" style="width:100%;min-height:220px;background:var(--raised);color:var(--text);
          border:1px solid var(--edge);border-radius:8px;padding:10px;font:12px/18px var(--mono);resize:vertical">${esc(sample)}</textarea>
        <div style="display:flex;gap:6px;margin-top:8px">
          <button class="tool-btn" data-apply>应用 Patch</button>
          <span id="d2-patch-msg" class="tool-label"></span>
        </div>`);
      m.root.querySelector('[data-apply]')?.addEventListener('click', () => {
        const ta = m.root.querySelector('#d2-patch') as HTMLTextAreaElement;
        const res = applyPatch(ta.value);
        const msg = m.root.querySelector('#d2-patch-msg');
        if (msg) {
          msg.textContent = res.ok ? `✓ ${res.msg}` : `✕ ${res.msg}`;
          msg.style.color = res.ok ? 'var(--success)' : 'var(--fail)';
        }
        ctx.status(res.ok ? res.msg : `受控 Patch 拒绝: ${res.msg}`);
      });
    }

    /* ── 启动 ─────────────────────────────────────────────────── */
    render();
    fitAll();
    ctx.status(`2D — ${index.nodes.size} 节点 / ${index.edges.size} 实现边 / ${index.designedEdges.size} 推断边 / ${index.ports.size} 端口 / ${DIAGRAMS.length} 种工程图`);

        const unsubBus = bus.subscribe((s) => {
      if (s.promptId) renderSidebar();
    });

return () => {
      /* 外殻常驻要素へのリスナも必ず解除する (累積防止) */
      shellListeners.abort();
      style.remove();
      floatLayer.remove();
      wrap.remove();
      unsubBus();
    };
  },
};
