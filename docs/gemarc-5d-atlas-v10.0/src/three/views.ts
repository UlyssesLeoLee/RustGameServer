/**
 * views.ts — 12 个 View 的过滤与布局规则 (spec `3D.views`)
 *
 * spec: views = [Overview, BirdEye, ExplodedFloor, RoomCutaway, Logical, Development,
 *                Deployment, Data, Process, Security, Operations, Dependency]
 *
 * 每个 View 都有**自己的过滤规则 + 布局规则 + 机位 + 标签模式**, 不是同一画面换名字。
 * 过滤全部基于 models/*.json 的真实字段 (domainId / kind / properties.deployment /
 * 边端点 / 容器层级), 不引入假数据。
 */

import type { AtlasModel, EdgeInfo, NodeInfo, RoomInfo } from './model';
import type { LabelMode } from './labels';

export type ViewId =
  | 'Overview' | 'BirdEye' | 'ExplodedFloor' | 'RoomCutaway' | 'Logical'
  | 'Development' | 'Deployment' | 'Data' | 'Process' | 'Security'
  | 'Operations' | 'Dependency';

export type LayoutMode =
  | 'stack' | 'plan' | 'explode' | 'cutaway' | 'aggregate' | 'layered' | 'grid';

export interface CameraPreset {
  az: number;
  el: number;
  dist: number;
}

export interface ViewContext {
  model: AtlasModel;
  activeFloorId: string;
  activeRoomId: string;
  selectedNodeId: string | null;
  /** 当前选中对象的图论相关集 (findImpactSubgraph ∪ ancestors) */
  relatedNodeIds: Set<string>;
  /** 直接邻居 */
  directNeighborIds: Set<string>;
}

export interface ViewDef {
  id: ViewId;
  label: string;
  desc: string;
  layout: LayoutMode;
  labelMode: LabelMode;
  camera: CameraPreset;
  /** 房间是否可见 */
  roomVisible: (room: RoomInfo, ctx: ViewContext) => boolean;
  /** 节点是否可见 */
  nodeVisible: (node: NodeInfo, ctx: ViewContext) => boolean;
  /** 边是否可见 */
  edgeVisible: (edge: EdgeInfo, ctx: ViewContext) => boolean;
  /** 网格布局的分组键 (layout = 'grid' 时生效) */
  groupKey?: (node: NodeInfo) => string;
  /** 该 View 是否以"房间"为主对象 (Logical: 房间聚合) */
  roomIsSubject?: boolean;
  /** 端口是否展示 */
  portsVisible: (ctx: ViewContext) => boolean;
}

const ALL = () => true;

const serviceNodes = (ctx: ViewContext) =>
  new Set([...ctx.model.nodes.values()].filter((n) => n.node.kind === 'Service').map((n) => n.id));

/** 参与真实调用边的节点 (边的任一端) */
const edgeParticipantNodes = (ctx: ViewContext) => {
  const s = new Set<string>();
  for (const e of ctx.model.edges) { s.add(e.sourceId); s.add(e.targetId); }
  return s;
};

export const VIEWS: ViewDef[] = [
  {
    id: 'Overview',
    label: '总览 Overview',
    desc: '全部 31 房间 / 160 节点 / 21 条边按空间层级堆叠。实线 = Implemented 真实调用点, 虚线 = Inferred 仅注释提及。',
    layout: 'stack',
    labelMode: 'Smart',
    camera: { az: 0.62, el: 0.44, dist: 270 },
    roomVisible: ALL,
    nodeVisible: ALL,
    edgeVisible: ALL,
    portsVisible: () => false,
  },
  {
    id: 'BirdEye',
    label: '俯视 BirdEye',
    desc: '俯视平面图: 8 个楼层全部压平到 y=0, 按 spatial.json 的房间网格看平面占位 (不显示调用连线, 避免俯视噪声)。',
    layout: 'plan',
    labelMode: 'Context',
    camera: { az: 0, el: 1.36, dist: 250 },
    roomVisible: ALL,
    nodeVisible: ALL,
    edgeVisible: () => false,
    portsVisible: () => false,
  },
  {
    id: 'ExplodedFloor',
    label: '楼层爆炸 ExplodedFloor',
    desc: '只保留当前激活楼层, 房间沿 XZ 外扩, 楼层内调用边保留, 便于看单层拓扑。',
    layout: 'explode',
    labelMode: 'Smart',
    camera: { az: 0.58, el: 0.52, dist: 130 },
    roomVisible: (room, ctx) => room.floorId === ctx.activeFloorId,
    nodeVisible: (node, ctx) => node.floorId === ctx.activeFloorId,
    edgeVisible: (edge, ctx) => {
      const s = ctx.model.nodes.get(edge.sourceId);
      const t = ctx.model.nodes.get(edge.targetId);
      return !!s && !!t && s.floorId === ctx.activeFloorId && t.floorId === ctx.activeFloorId;
    },
    portsVisible: () => false,
  },
  {
    id: 'RoomCutaway',
    label: '房间剖切 RoomCutaway',
    desc: '只保留激活房间: 前墙切除, 内部节点放大, 该 crate 的全部 gRPC 端口以凸台阵列铺在地板上, 可逐个拾取。',
    layout: 'cutaway',
    labelMode: 'All',
    camera: { az: 0.42, el: 0.26, dist: 46 },
    roomVisible: (room, ctx) => room.id === ctx.activeRoomId,
    nodeVisible: (node, ctx) => node.roomId === ctx.activeRoomId,
    edgeVisible: (edge, ctx) => edge.sourceRoomId === ctx.activeRoomId || edge.targetRoomId === ctx.activeRoomId,
    portsVisible: (ctx) => true,
  },
  {
    id: 'Logical',
    label: '逻辑投影 Logical',
    desc: '容器视角: 房间升为一级对象 (System→Building→Floor→Room), 边按真实调用折叠到房间级。semantics=logical, 是逻辑投影而非物理机房拓扑。',
    layout: 'aggregate',
    labelMode: 'Smart',
    camera: { az: 0.7, el: 0.46, dist: 265 },
    roomVisible: ALL,
    nodeVisible: () => false,
    edgeVisible: ALL,
    roomIsSubject: true,
    portsVisible: () => false,
  },
  {
    id: 'Development',
    label: '开发态 Development',
    desc: '只显示代码类实体 (Service + Api): 尺寸 = 测试函数数 / rpc 数, 切面 = Tested / NoTest (testFunctions>0)。表属于 schema, 不在本视图。',
    layout: 'grid',
    labelMode: 'Smart',
    camera: { az: 0.5, el: 0.36, dist: 210 },
    roomVisible: () => false,
    nodeVisible: (n) => n.node.kind === 'Service' || n.node.kind === 'Api',
    edgeVisible: (edge) => edge.edge.type === 'grpc-call' || edge.edge.type === 'grpc-call-designed',
    groupKey: (n) => String(n.node.properties?.crate ?? n.roomId),
    portsVisible: () => false,
  },
  {
    id: 'Deployment',
    label: '部署态 Deployment',
    desc: '只显示 Service: 有 k8s deployment 记录的 (10 个) 正常着色并显示副本数, 无记录的显示为 Unknown 幽灵态 —— 不等同于"未部署"。',
    layout: 'grid',
    labelMode: 'Smart',
    camera: { az: 0.42, el: 0.32, dist: 185 },
    roomVisible: () => false,
    nodeVisible: (n) => n.node.kind === 'Service',
    edgeVisible: (edge) => edge.edge.type === 'grpc-call' || edge.edge.type === 'grpc-call-designed',
    groupKey: (n) => (n.deployment ? '有 deployment 记录' : 'Unknown (无记录)'),
    portsVisible: () => false,
  },
  {
    id: 'Data',
    label: '数据层 Data',
    desc: '只显示 96 张表: 尺寸 = 列数, 切面 = Relational (有 FK) / Flat。模型里的 21 条边全是 service→service 的 gRPC 调用, 表之间没有建模边。',
    layout: 'grid',
    labelMode: 'Smart',
    camera: { az: 0.3, el: 0.56, dist: 205 },
    roomVisible: () => false,
    nodeVisible: (n) => n.node.kind === 'Table',
    edgeVisible: () => false,
    groupKey: (n) => String(n.node.properties?.crate ?? n.roomId),
    portsVisible: () => false,
  },
  {
    id: 'Process',
    label: '流程 Process',
    desc: '只保留真实调用链: 节点按调用深度分层 (根 = 出度最大的服务), 边带方向箭头, 展示主链路而非空间堆叠。',
    layout: 'layered',
    labelMode: 'Smart',
    camera: { az: 0.22, el: 0.26, dist: 210 },
    roomVisible: () => false,
    nodeVisible: (n, ctx) => edgeParticipantNodes(ctx).has(n.id),
    edgeVisible: ALL,
    portsVisible: () => false,
  },
  {
    id: 'Security',
    label: '安全域 Security',
    desc: '只显示业务域 = Security 的 16 个节点 (admin-service / i18n-service 两个房间), 保留 domainId 的排他配色。',
    layout: 'grid',
    labelMode: 'Smart',
    camera: { az: 0.44, el: 0.3, dist: 125 },
    roomVisible: (room) => room.domainId === 'Security',
    nodeVisible: (n) => n.node.domainId === 'Security',
    edgeVisible: (edge, ctx) => {
      const s = ctx.model.nodes.get(edge.sourceId);
      const t = ctx.model.nodes.get(edge.targetId);
      return !!s && !!t && s.node.domainId === 'Security' && t.node.domainId === 'Security';
    },
    groupKey: (n) => String(n.node.properties?.crate ?? n.roomId),
    portsVisible: () => false,
  },
  {
    id: 'Operations',
    label: '运维域 Operations',
    desc: '运维相关: 业务域 Operations / Infrastructure + 因果规则作用域节点 (causal-rules.json 的 scopeNodeIds), 规则条件可查阅。',
    layout: 'grid',
    labelMode: 'Smart',
    camera: { az: 0.5, el: 0.32, dist: 150 },
    roomVisible: (room) => room.domainId === 'Operations' || room.domainId === 'Infrastructure',
    nodeVisible: (n, ctx) => {
      if (n.node.domainId === 'Operations' || n.node.domainId === 'Infrastructure') return true;
      return ctx.model.rules.some((r) => r.scopeNodeIds.includes(n.id));
    },
    edgeVisible: (edge, ctx) => {
      const s = ctx.model.nodes.get(edge.sourceId);
      const t = ctx.model.nodes.get(edge.targetId);
      if (!s || !t) return false;
      const ok = (x: NodeInfo) => x.node.domainId === 'Operations' || x.node.domainId === 'Infrastructure';
      return ok(s) && ok(t);
    },
    groupKey: (n) => (n.node.domainId === 'Operations' ? 'Operations' : 'Infrastructure'),
    portsVisible: () => false,
  },
  {
    id: 'Dependency',
    label: '依赖闭包 Dependency',
    desc: '纯图论视图: 以选中节点为根, 取其祖先 + 后代闭包, 节点按依赖深度分层。未选中时展示全图。',
    layout: 'layered',
    labelMode: 'RelatedOnly',
    camera: { az: 0.3, el: 0.22, dist: 190 },
    roomVisible: () => false,
    nodeVisible: (n, ctx) => {
      if (!ctx.selectedNodeId) return true;
      return ctx.relatedNodeIds.has(n.id);
    },
    edgeVisible: (edge, ctx) => {
      if (!ctx.selectedNodeId) return true;
      return ctx.relatedNodeIds.has(edge.sourceId) && ctx.relatedNodeIds.has(edge.targetId);
    },
    portsVisible: () => false,
  },
];

export function viewById(id: ViewId): ViewDef {
  return VIEWS.find((v) => v.id === id) ?? VIEWS[0];
}

/** 流程视图的根节点: 出度最大的服务 (真实边算出) */
export function processRoot(model: AtlasModel): string | null {
  const out = new Map<string, number>();
  for (const e of model.edges) out.set(e.sourceId, (out.get(e.sourceId) ?? 0) + 1);
  let best: string | null = null;
  let bestN = -1;
  for (const [id, n] of out) {
    const node = model.nodes.get(id);
    if (!node || node.node.kind !== 'Service') continue;
    if (n > bestN) { bestN = n; best = id; }
  }
  return best ?? (model.nodes.keys().next().value as string | undefined) ?? null;
}
