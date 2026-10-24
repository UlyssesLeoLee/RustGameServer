/**
 * graph.ts — 图核心 (graphology 底座 + 自实现图论查询)
 *
 * spec `canonical_engineering_model.queries` 要求:
 *   getNode / getEdge / neighbors / inEdges / outEdges /
 *   ancestors / descendants / subgraph / findPath / findCycles / findImpactSubgraph
 *
 * 依赖策略 (偏离说明见 QA_FINAL.md §偏离台账):
 *   - graphology 作为 Node/Edge 与邻接底座 (spec "优先作为...底座")
 *   - ancestors/descendants/subgraph/path/cycles/impact 用 graphology 原语自实现,
 *     避免引入 graphology-traversal/-subgraph/-shortest-path/-cycles 4 个额外包,
 *     保持单文件产物体积与许可清单最小 (spec: "只采用许可兼容、版本锁定、可离线内联的依赖")
 */

import Graph from 'graphology';
import type {
  ArchitectureModel, GraphEdge, GraphNode, GraphPort, Container,
} from './types';

export interface GraphIndex {
  graph: Graph;
  nodes: Map<string, GraphNode>;
  edges: Map<string, GraphEdge>;
  ports: Map<string, GraphPort>;
  containers: Map<string, Container>;
  /** designedOnlyEdges 单独索引, 渲染时用不同样式 */
  designedEdges: Map<string, GraphEdge>;
  model: ArchitectureModel;
}

export function buildIndex(model: ArchitectureModel): GraphIndex {
  const graph = new Graph({ type: 'directed', multi: true, allowSelfLoops: true });
  const nodes = new Map<string, GraphNode>();
  const edges = new Map<string, GraphEdge>();
  const ports = new Map<string, GraphPort>();
  const containers = new Map<string, Container>();
  const designedEdges = new Map<string, GraphEdge>();

  for (const n of model.nodes) {
    nodes.set(n.id, n);
    if (!graph.hasNode(n.id)) graph.addNode(n.id, n);
  }
  for (const c of model.containers) containers.set(c.id, c);
  for (const p of model.ports) ports.set(p.id, p);

  const addEdge = (e: GraphEdge) => {
    if (!e.sourceId || !e.targetId) return;
    if (!nodes.has(e.sourceId) || !nodes.has(e.targetId)) return;
    edges.set(e.id, e);
    graph.addDirectedEdge(e.sourceId, e.targetId, e);
  };
  for (const e of model.edges) addEdge(e);
  for (const e of model.designedOnlyEdges ?? []) {
    if (!e.sourceId || !e.targetId) continue;
    if (!nodes.has(e.sourceId) || !nodes.has(e.targetId)) continue;
    designedEdges.set(e.id, e);
    graph.addDirectedEdge(e.sourceId, e.targetId, e);
  }

  return { graph, nodes, edges, ports, containers, designedEdges, model };
}

/* ── spec queries ──────────────────────────────────────────────────── */

export function getNode(idx: GraphIndex, id: string): GraphNode | undefined {
  return idx.nodes.get(id);
}

export function getEdge(idx: GraphIndex, id: string): GraphEdge | undefined {
  return idx.edges.get(id) ?? idx.designedEdges.get(id);
}

export function neighbors(idx: GraphIndex, id: string): string[] {
  if (!idx.graph.hasNode(id)) return [];
  return idx.graph.neighbors(id);
}

export function inEdges(idx: GraphIndex, id: string): GraphEdge[] {
  if (!idx.graph.hasNode(id)) return [];
  return idx.graph.inEdges(id).map((k) => idx.graph.getEdgeAttributes(k) as unknown as GraphEdge);
}

export function outEdges(idx: GraphIndex, id: string): GraphEdge[] {
  if (!idx.graph.hasNode(id)) return [];
  return idx.graph.outEdges(id).map((k) => idx.graph.getEdgeAttributes(k) as unknown as GraphEdge);
}

/** ancestors — 沿入边向上可达的全部节点 (不含自身) */
export function ancestors(idx: GraphIndex, id: string): Set<string> {
  const out = new Set<string>();
  if (!idx.graph.hasNode(id)) return out;
  const stack = [...idx.graph.inNeighbors(id)];
  while (stack.length) {
    const cur = stack.pop()!;
    if (out.has(cur) || cur === id) continue;
    out.add(cur);
    stack.push(...idx.graph.inNeighbors(cur));
  }
  return out;
}

/** descendants — 沿出边向下可达的全部节点 (不含自身) */
export function descendants(idx: GraphIndex, id: string): Set<string> {
  const out = new Set<string>();
  if (!idx.graph.hasNode(id)) return out;
  const stack = [...idx.graph.outNeighbors(id)];
  while (stack.length) {
    const cur = stack.pop()!;
    if (out.has(cur) || cur === id) continue;
    out.add(cur);
    stack.push(...idx.graph.outNeighbors(cur));
  }
  return out;
}

/**
 * subgraph — 取诱导子图 (保留两端都在集合内的边)。
 * spec `graph_requirements.SubgraphProjection`
 */
export function subgraph(idx: GraphIndex, nodeIds: Iterable<string>): {
  nodeIds: string[]; edgeIds: string[];
} {
  const keep = new Set(nodeIds);
  const nodeIdsOut = [...keep].filter((n) => idx.nodes.has(n));
  const edgeIds: string[] = [];
  for (const e of idx.edges.values()) {
    if (e.sourceId && e.targetId && keep.has(e.sourceId) && keep.has(e.targetId)) edgeIds.push(e.id);
  }
  for (const e of idx.designedEdges.values()) {
    if (e.sourceId && e.targetId && keep.has(e.sourceId) && keep.has(e.targetId)) edgeIds.push(e.id);
  }
  return { nodeIds: nodeIdsOut, edgeIds };
}

/** findPath — BFS 最短有向路径 (source→target), 无路返回 null */
export function findPath(idx: GraphIndex, source: string, target: string): string[] | null {
  if (!idx.graph.hasNode(source) || !idx.graph.hasNode(target)) return null;
  if (source === target) return [source];
  const prev = new Map<string, string>();
  const seen = new Set([source]);
  const queue: string[] = [source];
  while (queue.length) {
    const cur = queue.shift()!;
    for (const nb of idx.graph.outNeighbors(cur)) {
      if (seen.has(nb)) continue;
      seen.add(nb); prev.set(nb, cur);
      if (nb === target) {
        const path = [target];
        let p = target;
        while (prev.has(p)) { p = prev.get(p)!; path.unshift(p); }
        return path;
      }
      queue.push(nb);
    }
  }
  return null;
}

/** findCycles — DFS 找回边 (Johnson 简化版), 返回环的节点序列 */
export function findCycles(idx: GraphIndex, limit = 50): string[][] {
  const cycles: string[][] = [];
  const color = new Map<string, 0 | 1 | 2>();   // 0 white 1 gray 2 black
  const stack: string[] = [];
  const ids = idx.nodes.keys();

  const dfs = (u: string) => {
    if (cycles.length >= limit) return;
    color.set(u, 1); stack.push(u);
    for (const v of idx.graph.outNeighbors(u)) {
      const c = color.get(v) ?? 0;
      if (c === 1) {
        const at = stack.indexOf(v);
        if (at >= 0) cycles.push([...stack.slice(at), v]);
      } else if (c === 0) {
        dfs(v);
      }
    }
    stack.pop(); color.set(u, 2);
  };
  for (const id of ids) if ((color.get(id) ?? 0) === 0) dfs(id);
  return cycles;
}

/**
 * findImpactSubgraph — 变更某节点后受影响的下游闭包 + 使用的边。
 * spec `graph_requirements.ImpactTraversal`
 */
export function findImpactSubgraph(idx: GraphIndex, id: string): {
  nodeIds: string[]; edgeIds: string[];
} {
  const affected = descendants(idx, id);
  affected.add(id);
  return subgraph(idx, affected);
}

/** 依赖路径 — 从 root 到 target 的所有简单路径 (限量, 防爆炸) */
export function dependencyPaths(idx: GraphIndex, root: string, limit = 20): string[][] {
  const out: string[][] = [];
  const walk = (cur: string, path: string[]) => {
    if (out.length >= limit) return;
    for (const nb of idx.graph.outNeighbors(cur)) {
      if (path.includes(nb)) continue;
      const np = [...path, nb];
      out.push(np);
      walk(nb, np);
    }
  };
  if (idx.graph.hasNode(root)) walk(root, [root]);
  return out;
}

/* ── 容器层级查询 (3D 空间投影用) ──────────────────────────────────── */
export function containerChildren(idx: GraphIndex, id: string): Container[] {
  const c = idx.containers.get(id);
  if (!c) return [];
  return c.childrenIds.map((cid) => idx.containers.get(cid)!).filter(Boolean);
}

export function containerPath(idx: GraphIndex, id: string): Container[] {
  const out: Container[] = [];
  let cur = idx.containers.get(id);
  while (cur) { out.unshift(cur); cur = cur.parentId ? idx.containers.get(cur.parentId) : undefined; }
  return out;
}

/** 某个 room 下归属的全部节点 */
export function roomNodes(idx: GraphIndex, roomId: string): GraphNode[] {
  const room = idx.containers.get(roomId);
  if (!room) return [];
  const crate = (room.properties?.crate as string) ?? room.name;
  return [...idx.nodes.values()].filter((n) => n.properties?.crate === crate);
}
