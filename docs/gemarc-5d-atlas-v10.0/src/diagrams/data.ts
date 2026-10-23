/**
 * diagrams/data.ts — 真实模型数据提取 (渲染器的唯一数据入口)
 *
 * 全部函数只从 models/architecture.json 派生, 不引入任何虚构实体。
 * 派生规则 (可见性 / 基数 / 部署) 均在返回结构中显式携带, 由渲染器写进图注。
 */

import type { GraphNode, GraphEdge, GraphPort } from '../core/types';
import type { DiagramContext } from './types';

/* ── 节点分型 ─────────────────────────────────────────────────────── */

export function serviceNodes(ctx: DiagramContext): GraphNode[] {
  return [...ctx.index.nodes.values()].filter((n) => n.kind === 'Service');
}
export function apiNodes(ctx: DiagramContext): GraphNode[] {
  return [...ctx.index.nodes.values()].filter((n) => n.kind === 'Api');
}
export function tableNodes(ctx: DiagramContext): GraphNode[] {
  return [...ctx.index.nodes.values()].filter((n) => n.kind === 'Table');
}

export function crateOf(n: GraphNode): string {
  return String(n.properties.crate ?? '');
}

function groupBy<T>(items: T[], key: (t: T) => string): Map<string, T[]> {
  const m = new Map<string, T[]>();
  for (const it of items) {
    const k = key(it);
    if (!m.has(k)) m.set(k, []);
    m.get(k)!.push(it);
  }
  return m;
}

export function servicesByCrate(ctx: DiagramContext): Map<string, GraphNode[]> {
  return groupBy(serviceNodes(ctx), crateOf);
}
export function apisByCrate(ctx: DiagramContext): Map<string, GraphNode[]> {
  return groupBy(apiNodes(ctx), crateOf);
}
export function tablesByCrate(ctx: DiagramContext): Map<string, GraphNode[]> {
  return groupBy(tableNodes(ctx), crateOf);
}
/** 端口按 crate 聚合 (apiId 在采集模型中全为 null, 故只能按 crate 归属) */
export function portsByCrate(ctx: DiagramContext): Map<string, GraphPort[]> {
  return groupBy([...ctx.index.ports.values()], (p) => p.crate);
}

/* ── 边 ───────────────────────────────────────────────────────────── */

/** 实现边 (实线) + 推断边 (虚线, 按开关) */
export function allEdges(ctx: DiagramContext): GraphEdge[] {
  const out: GraphEdge[] = [...ctx.index.edges.values()];
  if (ctx.showInferred) out.push(...ctx.index.designedEdges.values());
  return out;
}
export function isDesigned(ctx: DiagramContext, e: GraphEdge): boolean {
  return ctx.index.designedEdges.has(e.id);
}
export function inbound(ctx: DiagramContext, id: string): GraphEdge[] {
  return allEdges(ctx).filter((e) => e.targetId === id);
}
export function outbound(ctx: DiagramContext, id: string): GraphEdge[] {
  return allEdges(ctx).filter((e) => e.sourceId === id);
}
export function edgeFileLine(e: GraphEdge): string {
  return String(e.properties.fileLine ?? '—');
}
export function edgeCalleeService(e: GraphEdge): string {
  return String(e.properties.calleeService ?? e.targetId ?? '');
}
export function edgeCallerCrate(e: GraphEdge): string {
  return String(e.properties.callerCrate ?? '');
}
export function edgeCalleeCrate(e: GraphEdge): string {
  return String(e.properties.calleeCrate ?? '');
}
export function edgeUsageForm(e: GraphEdge): string {
  return String(e.properties.usageForm ?? '');
}
export function edgeNote(e: GraphEdge): string {
  return String(e.properties.note ?? '');
}

/** 出现在真实调用图里的 crate 集合 (caller ∪ callee) */
export function cratesInCallGraph(ctx: DiagramContext): Set<string> {
  const s = new Set<string>();
  for (const e of allEdges(ctx)) {
    if (e.properties.callerCrate) s.add(String(e.properties.callerCrate));
    if (e.properties.calleeCrate) s.add(String(e.properties.calleeCrate));
  }
  return s;
}

/* ── 表 / 列 / 外键 ───────────────────────────────────────────────── */

export interface Column {
  name: string;
  type: string;
  pk: boolean;
  fk: string | null;
  notNull: boolean;
  unique: boolean;
}

export function columnsOf(table: GraphNode): Column[] {
  const raw = (table.properties.columns as unknown[]) ?? [];
  const out: Column[] = [];
  for (const c of raw) {
    const o = c as Partial<Column> & { name?: string };
    out.push({
      name: String(o.name ?? ''),
      type: String(o.type ?? ''),
      pk: o.pk === true,
      fk: o.fk === undefined ? null : (o.fk as string | null),
      notNull: o.notNull === true,
      unique: o.unique === true,
    });
  }
  return out;
}

/** 外键候选表 (按表名) */
export function fkCandidates(ctx: DiagramContext, ref: string): GraphNode[] {
  return tableNodes(ctx).filter((t) => t.name === ref);
}

/**
 * 外键目标解析: 同 crate 优先, 其次全局唯一, 其次歧义返回 null。
 * @returns 命中的目标表, 或 null (歧义 / 不存在)
 */
export function fkTarget(ctx: DiagramContext, table: GraphNode, ref: string): GraphNode | null {
  const cands = fkCandidates(ctx, ref);
  if (!cands.length) return null;
  const same = cands.filter((t) => crateOf(t) === crateOf(table));
  if (same.length === 1) return same[0];
  if (same.length > 1) return same[0];       // 同名同 crate 多份 (迁移重复) → 取首份, 图注声明
  return cands.length === 1 ? cands[0] : null;
}

/**
 * 基数推导 (crow's foot 记法):
 *   父端 (被引用表) = 1        — 每条子记录只指向 1 条父记录
 *   子端 (引用表)   = 0..* / 1..* / 0..1 / 1..1
 *     notNull && !unique → 1..*   (每父行至少一条子记录)
 *     !notNull && !unique → 0..*
 *     unique && notNull  → 1..1
 *     unique && !notNull → 0..1
 */
export function fkCardinality(col: Column): { child: string; parent: string } {
  let child: string;
  if (col.unique) child = col.notNull ? '1..1' : '0..1';
  else child = col.notNull ? '1..*' : '0..*';
  return { child, parent: '1' };
}

/** 表的全量外键 (含解析目标) */
export interface FkLink {
  from: GraphNode;          // 引用方 (子)
  to: GraphNode | null;     // 被引用方 (父)
  column: Column;
  cardinality: { child: string; parent: string };
  ambiguous: boolean;
}
export function fkLinks(ctx: DiagramContext): FkLink[] {
  const out: FkLink[] = [];
  for (const t of tableNodes(ctx)) {
    for (const col of columnsOf(t)) {
      if (!col.fk) continue;
      const cands = fkCandidates(ctx, col.fk);
      const target = fkTarget(ctx, t, col.fk);
      out.push({
        from: t,
        to: target,
        column: col,
        cardinality: fkCardinality(col),
        ambiguous: !target && cands.length > 1,
      });
    }
  }
  return out;
}

/* ── 部署 / 容器 ──────────────────────────────────────────────────── */

export interface DeploymentInfo {
  kind: string;
  namespace: string;
  replicas: number;
  manifest: string;
}
export function deploymentOf(n: GraphNode): DeploymentInfo | null {
  const d = n.properties.deployment as Partial<DeploymentInfo> | null | undefined;
  if (!d) return null;
  return {
    kind: String(d.kind ?? 'Deployment'),
    namespace: String(d.namespace ?? ''),
    replicas: Number(d.replicas ?? 0),
    manifest: String(d.manifest ?? ''),
  };
}
export function deployedServices(ctx: DiagramContext): GraphNode[] {
  return serviceNodes(ctx).filter((n) => n.properties.deployed === true && deploymentOf(n));
}

/** 业务域分面 (来自 containers 的 Floor 层, 与 spatial.json 同源) */
export interface DomainFacet {
  id: string;
  name: string;
  services: GraphNode[];
}
export function domainFacets(ctx: DiagramContext): DomainFacet[] {
  const floors = [...ctx.index.containers.values()].filter((c) => c.kind === 'Floor');
  const out: DomainFacet[] = [];
  for (const f of floors) {
    const roomIds = f.childrenIds.filter((cid) => ctx.index.containers.get(cid)?.kind === 'Room');
    const crates = new Set<string>();
    for (const rid of roomIds) {
      const room = ctx.index.containers.get(rid)!;
      const crate = String(room.properties?.crate ?? room.name);
      crates.add(crate);
    }
    const services = serviceNodes(ctx).filter((s) => crates.has(crateOf(s)));
    out.push({ id: f.name, name: f.name, services });
  }
  return out;
}

/* ── 选择域 (scope) ───────────────────────────────────────────────── */

export interface Scope {
  nodes: GraphNode[];
  edges: GraphEdge[];
  truncated: boolean;
  note: string;
}

/**
 * 依据当前选择收敛渲染域。
 * @param pick  参与本图种的节点判定
 * @param limit 数量上限 (超出截断并在图注声明)
 */
export function scopeFor(
  ctx: DiagramContext,
  pick: (n: GraphNode) => boolean,
  limit = 40,
): Scope {
  let pool = [...ctx.index.nodes.values()].filter(pick);

  // 有焦点时: 焦点 + 其一跳邻居优先
  const focus = ctx.selection.focusId;
  if (focus && ctx.index.nodes.has(focus)) {
    const keep = new Set<string>([focus]);
    for (const e of allEdges(ctx)) {
      if (e.sourceId === focus) keep.add(e.targetId!);
      if (e.targetId === focus) keep.add(e.sourceId!);
    }
    const narrowed = pool.filter((n) => keep.has(n.id));
    if (narrowed.length) pool = narrowed;
  } else if (ctx.selection.nodeIds.length) {
    const sel = new Set(ctx.selection.nodeIds);
    const keep = new Set<string>();
    for (const id of sel) {
      keep.add(id);
      for (const e of allEdges(ctx)) {
        if (e.sourceId === id) keep.add(e.targetId!);
        if (e.targetId === id) keep.add(e.sourceId!);
      }
    }
    const narrowed = pool.filter((n) => keep.has(n.id));
    if (narrowed.length) pool = narrowed;
  }

  const truncated = pool.length > limit;
  const nodes = truncated ? pool.slice(0, limit) : pool;
  const ids = new Set(nodes.map((n) => n.id));
  const edges = allEdges(ctx).filter(
    (e) => e.sourceId && e.targetId && ids.has(e.sourceId) && ids.has(e.targetId),
  );
  return {
    nodes,
    edges,
    truncated,
    note: truncated
      ? `节点数 ${pool.length} 超过上限 ${limit}, 已截断 (按模型顺序取前 ${limit}); 未显示部分不参与本图。`
      : '',
  };
}
