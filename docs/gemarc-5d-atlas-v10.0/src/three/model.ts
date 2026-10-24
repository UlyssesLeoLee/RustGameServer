/**
 * model.ts — 3D 场景的语义模型 (全部来自 models/*.json 的真实数据)
 *
 * 数据来源 (无任何编造):
 *   · architecture.json → 节点/边/端口/容器层级 (System→Building→Floor→Room)
 *   · spatial.json      → 房间 bounds + 楼层 y + building 名
 *   · causal-rules.json → 规则作用域 (Guard/条件)
 *   · cases.json        → Case 步骤 (事件/时间动画源)
 *
 * 已知缺口 (缺标比错标, 显式列出而不是掩盖):
 *   GAP-S1 spatial.json 的 31 个 room 全部声明 floorId="FL-undefined" 且 bounds.y 恒为 -26,
 *         即**楼层归属与纵向坐标在该文件里是退化的**。真实楼层归属来自 architecture.json 的
 *         容器层级 (C-FLOOR-<domain> → C-ROOM-<crate>), 楼层 y 来自 spatial.json 的
 *         building.floors。本模块据此组装, 并在 FloorInfo.source 标注每个值的出处。
 *   GAP-S2 节点在房间内的坐标是**确定性布局派生** (不是物理位置), UI 与检查器均如实标注。
 *   GAP-S3 architecture.json 没有运行时状态字段 → 所有节点的 ring = 'Unknown', 不臆造。
 *   GAP-S4 spatial.json 的 room.semantics 全为 'logical' → 场景是逻辑投影, 不是物理机房拓扑。
 */

import type { ArchitectureModel, GraphEdge, GraphNode, GraphPort, SpatialModel } from '../core/types';
import type { GraphIndex } from '../core/graph';
import { roomNodes, containerPath } from '../core/graph';
import { DOMAIN_COLOR, RING_STATE_COLOR } from '../core/tokens';
import type { GemRole } from './gems';

export interface Vec3 { x: number; y: number; z: number }

export interface FloorInfo {
  id: string;
  name: string;
  domainId: string;
  y: number;
  roomIds: string[];
  /** 该楼层 y 的出处 */
  source: string;
}

export interface RoomInfo {
  id: string;
  spatialId: string;
  name: string;
  crate: string;
  domainId: string;
  floorId: string;
  semantics: 'physical' | 'logical';
  bounds: { x: number; y: number; z: number; w: number; h: number; d: number };
  center: Vec3;
  nodeIds: string[];
  evidenceIds: string[];
  /** 容器路径名 (System→Building→Floor→Room) */
  path: string[];
  /** spatial.json 声明的 floorId (退化时为 'FL-undefined') */
  declaredFloorId: string;
}

export interface NodeInfo {
  id: string;
  node: GraphNode;
  role: GemRole;
  facet: string;
  ring: string;
  ringColor: string;
  color: string;
  metricLabel: string;
  metricValue: number;
  radius: number;
  position: Vec3;
  roomId: string;
  floorId: string;
  display: string;
  /** 部署信息 (来自 node.properties.deployment, 没有则 null → UI 标 Unknown) */
  deployment: Record<string, unknown> | null;
}

export interface PortInfo {
  id: string;
  port: GraphPort;
  roomId: string;
  position: Vec3;
}

export interface EdgeInfo {
  id: string;
  edge: GraphEdge;
  sourceId: string;
  targetId: string;
  sourceRoomId: string;
  targetRoomId: string;
  crossFloor: boolean;
  designedOnly: boolean;
  multiIndex: number;
  multiCount: number;
  label: string;
}

export interface CausalRuleInfo {
  ruleId: string;
  name: string;
  when: string;
  then: string;
  elseBranch?: string;
  scopeNodeIds: string[];
  evidenceIds: string[];
  origin: string;
}

export interface CaseInfo {
  caseId: string;
  title: string;
  kind: string;
  origin: string;
  description: string;
  nodeIds: string[];
  edgeIds: string[];
  steps: { seq: number; actor: string; action: string; target?: string; guard?: string; expect?: string }[];
}

export interface AtlasModel {
  floors: FloorInfo[];
  rooms: RoomInfo[];
  nodes: Map<string, NodeInfo>;
  /** 全部端口 (按 id) — 带 3D 位置 */
  portIndex: Map<string, PortInfo>;
  /** 端口 → 房间 */
  portRoom: Map<string, string>;
  edges: EdgeInfo[];
  roomById: Map<string, RoomInfo>;
  floorById: Map<string, FloorInfo>;
  nodeByRoom: Map<string, string[]>;
  rules: CausalRuleInfo[];
  cases: CaseInfo[];
  buildingName: string;
  buildingId: string;
  /** 全部房间均为 logical → 逻辑投影, 非物理部署事实 */
  allLogical: boolean;
  meta: { revision: string; shortRevision: string; branch: string; subject: string };
}

/* ── 角色映射 (形状 = 功能角色) ───────────────────────────────────────── */

/**
 * node.kind → 宝石角色。
 * architecture.json 的 role 字段只有三类 (Rust crate / 业务逻辑单元、gRPC 服务、表/数据模型),
 * 分别对应 Processor / Interface / Storage。
 * 其余 8 个角色 (Actor/Decision/Message/Security/Agent/Composition/Input/Output) 的几何体已实现,
 * 但当前数据集没有对应实体 → 属于"已实现但未被数据触发", 不用假数据填充。
 */
export function roleOfKind(kind: string): GemRole {
  switch (kind) {
    case 'Service': return 'Processor';
    case 'Api': return 'Interface';
    case 'Table': return 'Storage';
    case 'Actor': return 'Actor';
    case 'Message': return 'Message';
    case 'Component': return 'Composition';
    case 'Screen': return 'Interface';
    case 'Requirement': return 'Decision';
    default: return 'Processor';
  }
}

/** 切面 = 角色子类 (来自真实 properties) */
export function facetOf(node: GraphNode): string {
  const p = node.properties ?? {};
  switch (node.kind) {
    case 'Service':
      return Number(p.testFunctions ?? 0) > 0 ? 'Tested' : 'NoTest';
    case 'Api':
      return String(p.package ?? 'unknown');
    case 'Table':
      return Array.isArray(p.fkColumns) && p.fkColumns.length > 0 ? 'Relational' : 'Flat';
    default:
      return 'Generic';
  }
}

/** 尺寸指标 = 该角色下"有证据的数量指标" */
export function metricOf(node: GraphNode): { label: string; value: number } {
  const p = node.properties ?? {};
  switch (node.kind) {
    case 'Service':
      return { label: '测试函数', value: Number(p.testFunctions ?? 0) };
    case 'Api':
      return { label: 'rpc', value: Number(p.rpcCount ?? 0) };
    case 'Table':
      return { label: '列', value: Array.isArray(p.columns) ? p.columns.length : 0 };
    default:
      return { label: '证据', value: node.evidenceIds.length };
  }
}

export function radiusFor(value: number): number {
  return Math.min(2.2, 0.62 + 0.3 * Math.log1p(Math.max(0, value)));
}

/* ── 布局 (语义坐标) ──────────────────────────────────────────────────── */

const TABLE_Y = 1.15;
const API_Y = 8.3;
const SERVICE_Y = 4.6;

function gridPositions(count: number, cols: number, spacing: number): { dx: number; dz: number }[] {
  const out: { dx: number; dz: number }[] = [];
  const c = Math.max(1, cols);
  const rows = Math.ceil(count / c);
  for (let i = 0; i < count; i++) {
    const r = Math.floor(i / c);
    const col = i % c;
    const rowCount = r === rows - 1 ? count - r * c : c;
    out.push({
      dx: (col - (rowCount - 1) / 2) * spacing,
      dz: (r - (rows - 1) / 2) * spacing,
    });
  }
  return out;
}

/* ── 主构建函数 ───────────────────────────────────────────────────────── */

export function buildAtlasModel(
  arch: ArchitectureModel,
  idx: GraphIndex,
  spatial: SpatialModel,
  rules: CausalRuleInfo[],
  cases: CaseInfo[],
): AtlasModel {
  /* 楼层: 名称来自 spatial.json 的 building.floors, 归属来自 architecture.json 容器 */
  const floorByDomain = new Map<string, { id: string; y: number; roomIds: string[] }>();
  for (const f of spatial.building.floors) {
    floorByDomain.set(f.name, { id: f.id, y: f.y, roomIds: [] });
  }
  const floorByContainerId = new Map<string, FloorInfo>();

  const roomById = new Map<string, RoomInfo>();
  const rooms: RoomInfo[] = [];

  for (const c of idx.containers.values()) {
    if (c.kind !== 'Room') continue;
    const crate = String(c.properties?.crate ?? c.name);
    const domainId = String(c.properties?.domain ?? 'Unknown');
    const parentFloorId = c.parentId ?? '';
    const parentFloor = idx.containers.get(parentFloorId);
    const floorName = parentFloor?.name ?? domainId;
    const spatialFloor = floorByDomain.get(floorName);
    const y = spatialFloor?.y ?? 0;

    // spatial.json 的 room (按 crate 名匹配)
    const sp = spatial.rooms.find((r) => r.name === crate || r.id === `ROOM-${crate}`);
    const bounds = sp
      ? { ...sp.bounds }
      : { x: 0, y: 0, z: 0, w: 18, h: 12, d: 14 };

    const nodes = roomNodes(idx, c.id);
    const path = containerPath(idx, c.id).map((x) => x.name);

    const room: RoomInfo = {
      id: c.id,
      spatialId: sp?.id ?? `ROOM-${crate}`,
      name: c.name,
      crate,
      domainId,
      floorId: parentFloorId,
      semantics: sp?.semantics ?? 'logical',
      bounds,
      center: { x: bounds.x + bounds.w / 2, y: y + bounds.h / 2, z: bounds.z + bounds.d / 2 },
      nodeIds: nodes.map((n) => n.id),
      evidenceIds: sp?.evidenceIds ?? [],
      path,
      declaredFloorId: sp?.floorId ?? 'FL-undefined',
    };
    rooms.push(room);
    roomById.set(room.id, room);

    const fl = floorByContainerId.get(parentFloorId);
    if (fl) fl.roomIds.push(room.id);
    else {
      floorByContainerId.set(parentFloorId, {
        id: parentFloorId,
        name: floorName,
        domainId,
        y,
        roomIds: [room.id],
        source: `spatial.json building.floors[${floorName}].y=${y} + architecture.json 容器层级`,
      });
    }
    if (spatialFloor) spatialFloor.roomIds.push(room.id);
  }

  const floors = [...floorByContainerId.values()].sort((a, b) => a.y - b.y);

  /* 节点: 位置派生 + 排他语义通道 */
  const nodes = new Map<string, NodeInfo>();
  const nodeByRoom = new Map<string, string[]>();

  for (const room of rooms) {
    const inRoom = room.nodeIds
      .map((id) => idx.nodes.get(id))
      .filter((n): n is GraphNode => !!n);
    const services = inRoom.filter((n) => n.kind === 'Service');
    const apis = inRoom.filter((n) => n.kind === 'Api');
    const tables = inRoom.filter((n) => n.kind === 'Table');
    const others = inRoom.filter((n) => n.kind !== 'Service' && n.kind !== 'Api' && n.kind !== 'Table');

    const cx = room.bounds.x + room.bounds.w / 2;
    const cz = room.bounds.z + room.bounds.d / 2;
    const floorY = room.center.y - room.bounds.h / 2;

    const place = (list: GraphNode[], y: number, spacing: number, colsHint: number) => {
      const g = gridPositions(list.length, colsHint, spacing);
      list.forEach((n, i) => {
        const metric = metricOf(n);
        const ring = typeof n.properties?.runtimeState === 'string'
          ? String(n.properties.runtimeState)
          : 'Unknown';
        const info: NodeInfo = {
          id: n.id,
          node: n,
          role: roleOfKind(n.kind),
          facet: facetOf(n),
          ring,
          ringColor: RING_STATE_COLOR[ring] ?? RING_STATE_COLOR.Unknown,
          color: DOMAIN_COLOR[n.domainId] ?? DOMAIN_COLOR.Unknown,
          metricLabel: metric.label,
          metricValue: metric.value,
          radius: radiusFor(metric.value),
          position: { x: cx + g[i].dx, y: floorY + y, z: cz + g[i].dz },
          roomId: room.id,
          floorId: room.floorId,
          display: n.name,
          deployment: (n.properties?.deployment as Record<string, unknown> | null) ?? null,
        };
        nodes.set(n.id, info);
      });
    };

    // 服务 (Processor) 居中悬于房间中部
    for (const n of services) {
      const metric = metricOf(n);
      const ring = typeof n.properties?.runtimeState === 'string' ? String(n.properties.runtimeState) : 'Unknown';
      const info: NodeInfo = {
        id: n.id,
        node: n,
        role: roleOfKind(n.kind),
        facet: facetOf(n),
        ring,
        ringColor: RING_STATE_COLOR[ring] ?? RING_STATE_COLOR.Unknown,
        color: DOMAIN_COLOR[n.domainId] ?? DOMAIN_COLOR.Unknown,
        metricLabel: metric.label,
        metricValue: metric.value,
        radius: radiusFor(metric.value),
        position: { x: cx, y: floorY + SERVICE_Y, z: cz },
        roomId: room.id,
        floorId: room.floorId,
        display: n.name,
        deployment: (n.properties?.deployment as Record<string, unknown> | null) ?? null,
      };
      nodes.set(n.id, info);
    }
    // API (Interface) 上层网格
    place(apis, API_Y, 3.4, Math.ceil(Math.sqrt(Math.max(1, apis.length))));
    // 表 (Storage) 房间地板层网格
    place(tables, TABLE_Y, 3.0, Math.ceil(Math.sqrt(Math.max(1, tables.length) * (room.bounds.w / Math.max(1, room.bounds.d)))));
    place(others, API_Y, 3.4, 3);

    nodeByRoom.set(room.id, [...services, ...apis, ...tables, ...others].map((n) => n.id));
  }

  /* ポート: 記録所属ルーム + **3D 上の位置**。
   * PortInfo は position を持つが、以前 portIndex は生 GraphPort を入れており
   * position が未定義 → 3D シーン構築時に "reading 'x'" で落ちる。
   * ポートは「所属 crate のサービスノードの周囲」に決定論的な角度オフセットで配置する
   * (同一入力 → 同一配置を保つ = 3D の決定性要件)。 */
  const portIndex = new Map<string, PortInfo>();
  const portRoom = new Map<string, string>();
  const portsByCrate = new Map<string, GraphPort[]>();
  for (const p of idx.ports.values()) {
    const room = rooms.find((r) => r.crate === p.crate);
    if (!room) continue;
    portRoom.set(p.id, room.id);
    const list = portsByCrate.get(p.crate) ?? [];
    list.push(p);
    portsByCrate.set(p.crate, list);
  }
  for (const [crate, list] of portsByCrate) {
    const room = rooms.find((r) => r.crate === crate)!;
    const svcNode = [...nodes.values()].find((n) => n.node.properties?.crate === crate && n.node.kind === 'Service');
    const anchor = svcNode?.position ?? { x: room.center.x, y: room.center.y, z: room.center.z };
    const radius = (svcNode?.radius ?? 2) * 1.9;
    list.forEach((p, i) => {
      const angle = (i / Math.max(1, list.length)) * Math.PI * 2;
      portIndex.set(p.id, {
        id: p.id,
        port: p,
        roomId: room.id,
        position: {
          x: anchor.x + Math.cos(angle) * radius,
          y: anchor.y,
          z: anchor.z + Math.sin(angle) * radius,
        },
      });
    });
  }

  /* 边: 真实边 + designedOnly 边; 同 (source,target) 多条 → 记 multiIndex */
  const pairCount = new Map<string, number>();
  const pairSeen = new Map<string, number>();
  const allEdges = [...idx.edges.values(), ...idx.designedEdges.values()];
  const edges: EdgeInfo[] = [];
  for (const e of allEdges) {
    if (!e.sourceId || !e.targetId) continue;
    const src = nodes.get(e.sourceId);
    const dst = nodes.get(e.targetId);
    if (!src || !dst) continue;
    const key = `${e.sourceId}→${e.targetId}`;
    const total = (pairCount.get(key) ?? 0) + 1;
    pairCount.set(key, total);
    const seen = pairSeen.get(key) ?? 0;
    pairSeen.set(key, seen + 1);
    const designedOnly = e.verification === 'Inferred';
    edges.push({
      id: e.id,
      edge: e,
      sourceId: e.sourceId,
      targetId: e.targetId,
      sourceRoomId: src.roomId,
      targetRoomId: dst.roomId,
      crossFloor: src.floorId !== dst.floorId,
      designedOnly,
      multiIndex: seen,
      multiCount: total,
      label: `${e.type} · ${src.display} → ${dst.display}`,
    });
  }

  return {
    floors,
    rooms,
    nodes,
    portIndex,
    portRoom,
    edges,
    roomById,
    floorById: new Map(floors.map((f) => [f.id, f])),
    nodeByRoom,
    rules,
    cases,
    buildingName: spatial.building.name,
    buildingId: spatial.building.id,
    allLogical: rooms.length > 0 && rooms.every((r) => r.semantics === 'logical'),
    meta: {
      revision: arch.meta.revision,
      shortRevision: arch.meta.shortRevision,
      branch: arch.meta.branch,
      subject: arch.meta.subject,
    },
  };
}

/** 生成房间内端口的位置 (端口甲板: 房间地板上的密集凸台阵列) */
export function portPositions(model: AtlasModel, roomId: string): PortInfo[] {
  const room = model.roomById.get(roomId);
  if (!room) return [];
  const list: PortInfo[] = [];
  const inCrate = [...model.portIndex.values()].filter((p) => p.crate === room.crate);
  const spacing = 0.78;
  const cols = Math.max(1, Math.floor((room.bounds.w - 1.2) / spacing));
  const rows = Math.max(1, Math.ceil(inCrate.length / cols));
  const floorY = room.center.y - room.bounds.h / 2;
  inCrate.forEach((p, i) => {
    const r = Math.floor(i / cols);
    const c = i % cols;
    list.push({
      id: p.id,
      port: p,
      roomId: room.id,
      position: {
        x: room.bounds.x + 0.6 + spacing * 0.5 + c * spacing,
        y: floorY + 0.42,
        z: room.bounds.z + 0.6 + spacing * 0.5 + (r % rows) * spacing,
      },
    });
  });
  return list;
}
