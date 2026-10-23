/**
 * model.ts — Prototype 文档模型 (序列化真源)
 *
 * spec `prototype_workbench.state_model`:
 *   "Page、Component、Action、Guard、State、Fixture、Route、Variant 版本化;
 *    Prototype JSON 序列化规则, 不只序列化 Konva Stage.toJSON"
 *
 * 因此本文件定义 **9 类实体 + 1 个文档**, 每个实体带 `version` 字段,
 * 文档带 `meta.revision` / `meta.updatedAt`。导出时三份格式:
 *   - PrototypeJSON         全量文档 (含 Page/Component/Action/Guard/State/Fixture/Route/Variant)
 *   - ActionGraphJSON       动作图 (component → action → screen/case/state)
 *   - StateTransitionTable  状态转移表 (screen.state / component.state)
 *
 * 边界: core/types.ts 的 PrototypeModel 是**只读输入**(models/prototype.json),
 * 这里在它的基础上补齐 spec 要求的实体, 不修改 core/types.ts。
 */

import type {
  ActionKind, ProtoAction, ProtoComponent, ProtoScreen, PrototypeModel, ScenarioOrigin, WidgetType,
} from '../core/types';

export type ViewportName = 'desktop' | 'tablet' | 'mobile';
export type FixtureMode = 'ok' | 'error' | 'timeout' | 'recover';
export type CasePhase = 'idle' | 'waiting' | 'success' | 'error' | 'compensating' | 'blocked';

export const VIEWPORTS: { name: ViewportName; label: string; w: number; h: number }[] = [
  { name: 'desktop', label: 'Desktop 1440×900', w: 1440, h: 900 },
  { name: 'tablet', label: 'Tablet 768×1024', w: 768, h: 1024 },
  { name: 'mobile', label: 'Mobile 390×844', w: 390, h: 844 },
];

export const FIXTURE_MODES: { mode: FixtureMode; label: string }[] = [
  { mode: 'ok', label: '正常' },
  { mode: 'error', label: '错误' },
  { mode: 'timeout', label: '超时' },
  { mode: 'recover', label: '恢复' },
];

export const ALL_ACTION_KINDS: ActionKind[] = [
  'Navigate', 'OpenTab', 'SwitchTab', 'SetState', 'SetVariable', 'ShowHide', 'Validate',
  'SubmitMock', 'OpenDialog', 'CloseDialog', 'Filter', 'Sort', 'Load', 'Fail', 'Retry', 'TriggerCase',
];

export const ALL_WIDGETS: WidgetType[] = [
  'Button', 'Input', 'Textarea', 'Select', 'Radio', 'Checkbox', 'Tabs', 'List', 'Table',
  'Card', 'Dialog', 'Drawer', 'Toast', 'Navigation', 'Loading', 'Empty', 'Error',
];

/* ── 实体 ─────────────────────────────────────────────────────────── */

export interface ProtoPage {
  id: string;
  name: string;
  order: number;
  screenIds: string[];
  version: number;
  note: string;
}

export interface ProtoGuard {
  id: string;
  name: string;
  expr: string;
  description: string;
  version: number;
}

export interface ProtoStateDef {
  id: string;
  name: string;
  scope: 'screen' | 'component' | 'global';
  key: string;
  initial: string;
  description: string;
  version: number;
}

export interface ProtoFixture {
  id: string;
  mode: FixtureMode;
  delayMs: number;
  status: number;
  message: string;
  data: unknown;
  version: number;
}

export interface ProtoRoute {
  id: string;
  path: string;
  name: string;
  screenId: string;
  version: number;
}

export interface ProtoVariant {
  id: string;
  name: string;
  screenId: string;
  viewport: ViewportName;
  overrides: Record<string, unknown>;
  version: number;
}

export interface ProtoCaseBinding {
  id: string;
  caseId: string;
  screenId: string;
  componentId: string;
  phase: CasePhase;
  binding: 'bound' | 'mock';
  note: string;
  version: number;
}

export interface ProtoLink {
  screenId: string;
  componentId: string;
  requirementIds: string[];
  nodeIds: string[];
  caseIds: string[];
  promptIds: string[];
  pageFlowIds: string[];
  scenarioIds: string[];
}

export interface ProtoLogEntry {
  seq: number;
  actionId: string;
  kind: ActionKind;
  label: string;
  screenId: string;
  componentId: string | null;
  guard: string | null;
  guardPassed: boolean;
  effect: string;
  detail: string;
  ok: boolean;
}

export interface ProtoDoc {
  meta: {
    schemaVersion: string;
    origin: ScenarioOrigin;
    note: string;
    generator: string;
    revision: number;
    updatedAt: string;
    /** 示范层 (Designed/Hypothetical) 实体的 ID 清单 —— 界面必须显式标注 */
    demoEntities: string[];
  };
  pages: ProtoPage[];
  screens: ProtoScreen[];
  components: ProtoComponent[];
  actions: ProtoAction[];
  guards: ProtoGuard[];
  states: ProtoStateDef[];
  /** 数据 fixture (models/prototype.json 原样保留: screenId → componentId → data) */
  fixtures: Record<string, unknown>;
  /** 确定性 Mock fixture (正常/错误/超时/恢复) */
  mockFixtures: ProtoFixture[];
  routes: ProtoRoute[];
  variants: ProtoVariant[];
  links: ProtoLink[];
  caseBindings: ProtoCaseBinding[];
  /** 操作日志 (导出带出, 供 ImportAndReplay 复原交互规则) */
  log: ProtoLogEntry[];
}

/* ── 载入 / 归一 ──────────────────────────────────────────────────── */

interface AtlasModels {
  prototype?: PrototypeModel;
}

function atlasModels(): AtlasModels {
  return (window as unknown as { __ATLAS_MODELS?: AtlasModels }).__ATLAS_MODELS ?? {};
}

function clone<T>(v: T): T {
  return JSON.parse(JSON.stringify(v)) as T;
}

function nowIso(): string {
  return new Date().toISOString();
}

/**
 * 从 models/prototype.json 归一化为完整 ProtoDoc。
 * 模型只提供 screens / components / actions / fixtures / links;
 * spec 要求的 Page / Guard / State / Fixture(mock) / Route / Variant 由本层补齐,
 * 全部标为 Designed (示范), 不冒充真实产品资料。
 */
export function loadDoc(): ProtoDoc {
  const raw = atlasModels().prototype;
  if (!raw) {
    return emptyDoc('未找到 models/prototype.json 的内联数据');
  }
  const doc = emptyDoc(raw.meta?.note ?? '');
  doc.meta.schemaVersion = raw.meta?.schemaVersion ?? '1.0.0';
  doc.meta.origin = raw.meta?.origin ?? 'HypotheticalExperiment';

  doc.screens = clone(raw.screens ?? []);
  doc.components = clone(raw.components ?? []);
  doc.actions = clone(raw.actions ?? []);
  doc.fixtures = clone(raw.fixtures ?? {});
  doc.links = (raw.links ?? []).map((l) => ({
    screenId: l.screenId,
    componentId: l.componentId,
    requirementIds: [...(l.requirementIds ?? [])],
    nodeIds: [...(l.nodeIds ?? [])],
    caseIds: [...(l.caseIds ?? [])],
    promptIds: [...(l.promptIds ?? [])],
    pageFlowIds: [],
    scenarioIds: [],
  }));

  /* Page: 模型无页面概念 → 建 1 个页面承载全部 Screen */
  doc.pages = [{
    id: 'PAGE-001',
    name: '机台运维 (假设性示范)',
    order: 0,
    screenIds: doc.screens.map((s) => s.id),
    version: 1,
    note: '由 models/prototype.json 的 3 个 Screen 归组而来; 模型本身无 Page 实体。',
  }];

  /* Route: 由 Screen.route 派生, 稳定 ID */
  doc.routes = doc.screens.map((s, i) => ({
    id: `RT-${String(i + 1).padStart(3, '0')}`,
    path: s.route,
    name: s.name,
    screenId: s.id,
    version: 1,
  }));

  /* Variant: 由 Screen.variant 派生 */
  doc.variants = doc.screens
    .filter((s) => s.variant)
    .map((s, i) => ({
      id: `VAR-${String(i + 1).padStart(3, '0')}`,
      name: `${s.name} · ${s.variant}`,
      screenId: s.id,
      viewport: (s.variant as ViewportName) ?? 'desktop',
      overrides: {},
      version: 1,
    }));

  /* State: 由 Screen.states 派生 */
  doc.states = [];
  let stSeq = 0;
  for (const s of doc.screens) {
    for (const [key, initial] of Object.entries(s.states ?? {})) {
      stSeq++;
      doc.states.push({
        id: `ST-${String(stSeq).padStart(3, '0')}`,
        name: `${s.name} / ${key}`,
        scope: 'screen',
        key,
        initial,
        description: `来自 models/prototype.json 的 ${s.id}.states.${key}`,
        version: 1,
      });
    }
  }

  /* Guard: 模型无 guard → 建 2 条示范 guard 供 设计与测试 Tab 校验 */
  doc.guards = [
    {
      id: 'GRD-001',
      name: '故障面板已打开',
      expr: "screen.panel == 'open'",
      description: '示范 guard: 故障详情抽屉处于打开态时才允许"确认已处理"。',
      version: 1,
    },
    {
      id: 'GRD-002',
      name: '未处于错误态',
      expr: "component.CMP-003.status != 'error'",
      description: '示范 guard: 机台总览表未处于 error 态时才允许"重试"。',
      version: 1,
    },
  ];

  /* mockFixtures: 4 种确定性模式 */
  doc.mockFixtures = [
    { id: 'FX-OK', mode: 'ok', delayMs: 0, status: 200, message: '查询成功 (确定性 Mock)', data: null, version: 1 },
    { id: 'FX-ERR', mode: 'error', delayMs: 0, status: 503, message: '下游不可用 (确定性 Mock, 非真实调用)', data: null, version: 1 },
    { id: 'FX-TMO', mode: 'timeout', delayMs: 1200, status: 504, message: '请求超时 (确定性 Mock, 非真实调用)', data: null, version: 1 },
    { id: 'FX-REC', mode: 'recover', delayMs: 800, status: 200, message: '首次失败, 重试后恢复 (确定性 Mock)', data: null, version: 1 },
  ];

  return doc;
}

export function emptyDoc(note: string): ProtoDoc {
  return {
    meta: {
      schemaVersion: '1.1.0',
      origin: 'HypotheticalExperiment',
      note,
      generator: 'GemArc Atlas Prototype Workbench (Konva 开源核心)',
      revision: 1,
      updatedAt: nowIso(),
      demoEntities: [],
    },
    pages: [],
    screens: [],
    components: [],
    actions: [],
    guards: [],
    states: [],
    fixtures: {},
    mockFixtures: [],
    routes: [],
    variants: [],
    links: [],
    caseBindings: [],
    log: [],
  };
}

export function cloneDoc(doc: ProtoDoc): ProtoDoc {
  return clone(doc);
}

/** 文档修订: 每次结构性变更调用 */
export function touch(doc: ProtoDoc): void {
  doc.meta.revision += 1;
  doc.meta.updatedAt = nowIso();
}

/* ── 选择器 ───────────────────────────────────────────────────────── */

export function screenById(doc: ProtoDoc, id: string | null): ProtoScreen | undefined {
  return id ? doc.screens.find((s) => s.id === id) : undefined;
}
export function componentById(doc: ProtoDoc, id: string | null): ProtoComponent | undefined {
  return id ? doc.components.find((c) => c.id === id) : undefined;
}
export function actionById(doc: ProtoDoc, id: string | null): ProtoAction | undefined {
  return id ? doc.actions.find((a) => a.actionId === id) : undefined;
}
export function guardById(doc: ProtoDoc, id: string | null): ProtoGuard | undefined {
  return id ? doc.guards.find((g) => g.id === id) : undefined;
}
export function routeById(doc: ProtoDoc, id: string | null): ProtoRoute | undefined {
  return id ? doc.routes.find((r) => r.id === id) : undefined;
}
export function variantById(doc: ProtoDoc, id: string | null): ProtoVariant | undefined {
  return id ? doc.variants.find((v) => v.id === id) : undefined;
}
export function stateById(doc: ProtoDoc, id: string | null): ProtoStateDef | undefined {
  return id ? doc.states.find((s) => s.id === id) : undefined;
}
export function fixtureById(doc: ProtoDoc, id: string | null): ProtoFixture | undefined {
  return id ? doc.mockFixtures.find((f) => f.id === id) : undefined;
}
export function componentsOfScreen(doc: ProtoDoc, screenId: string): ProtoComponent[] {
  const screen = screenById(doc, screenId);
  if (!screen) return [];
  return screen.componentIds
    .map((id) => componentById(doc, id))
    .filter((c): c is ProtoComponent => !!c);
}
export function actionsOfScreen(doc: ProtoDoc, screenId: string): ProtoAction[] {
  const screen = screenById(doc, screenId);
  if (!screen) return [];
  return screen.actionIds
    .map((id) => actionById(doc, id))
    .filter((a): a is ProtoAction => !!a);
}
/** 某 Screen 上全部 action (含 component 引用但未登记在 screen.actionIds 的) */
export function allActionsOfScreen(doc: ProtoDoc, screenId: string): ProtoAction[] {
  const screen = screenById(doc, screenId);
  if (!screen) return [];
  const ids = new Set<string>(screen.actionIds);
  for (const c of componentsOfScreen(doc, screenId)) {
    for (const aid of c.actionIds) ids.add(aid);
  }
  return [...ids].map((id) => actionById(doc, id)).filter((a): a is ProtoAction => !!a);
}

/** 画布自然尺寸 (由组件外接矩形推导, 至少 1200×760) */
export function canvasSize(doc: ProtoDoc, screenId: string): { w: number; h: number } {
  let w = 1200;
  let h = 760;
  for (const c of componentsOfScreen(doc, screenId)) {
    w = Math.max(w, c.x + c.w);
    h = Math.max(h, c.y + c.h);
  }
  return { w: Math.ceil(w), h: Math.ceil(h) };
}

/* ── 序列化 ───────────────────────────────────────────────────────── */

/** PrototypeJSON —— 全量文档 */
export function serializePrototypeJSON(doc: ProtoDoc): string {
  return JSON.stringify({
    kind: 'GemArc.PrototypeDocument',
    schemaVersion: '1.1.0',
    meta: doc.meta,
    pages: doc.pages,
    screens: doc.screens,
    components: doc.components,
    actions: doc.actions,
    guards: doc.guards,
    states: doc.states,
    fixtures: doc.fixtures,
    mockFixtures: doc.mockFixtures,
    routes: doc.routes,
    variants: doc.variants,
    links: doc.links,
    caseBindings: doc.caseBindings,
    log: doc.log,
  }, null, 2);
}

export interface ActionGraphNode {
  id: string;
  type: 'screen' | 'component' | 'action' | 'state' | 'case';
  label: string;
  kind?: ActionKind;
  guardId?: string | null;
}

export interface ActionGraphEdge {
  id: string;
  from: string;
  to: string;
  relation: 'contains' | 'triggers' | 'navigates' | 'targets' | 'sets' | 'guards' | 'binds';
  guardId: string | null;
}

/** ActionGraphJSON —— 动作图 (component → action → screen/state/case) */
export function serializeActionGraphJSON(doc: ProtoDoc): string {
  const nodes: ActionGraphNode[] = [];
  const edges: ActionGraphEdge[] = [];
  let eSeq = 0;
  const edge = (from: string, to: string, relation: ActionGraphEdge['relation'], guardId: string | null = null) => {
    eSeq++;
    edges.push({ id: `E-${String(eSeq).padStart(4, '0')}`, from, to, relation, guardId });
  };

  for (const s of doc.screens) {
    nodes.push({ id: s.id, type: 'screen', label: s.name });
    for (const cid of s.componentIds) {
      const c = componentById(doc, cid);
      if (!c) continue;
      nodes.push({ id: c.id, type: 'component', label: `${c.widget} ${c.id}` });
      edge(s.id, c.id, 'contains');
      for (const aid of c.actionIds) {
        const a = actionById(doc, aid);
        if (!a) continue;
        nodes.push({ id: a.actionId, type: 'action', label: a.label, kind: a.kind, guardId: a.guard ?? null });
        edge(c.id, a.actionId, 'triggers', a.guard ?? null);
        if (a.guard) edge(a.actionId, a.guard, 'guards', a.guard);
        const target = a.target ?? (a.payload?.screenId as string | undefined);
        if (target && screenById(doc, target)) edge(a.actionId, target, 'navigates', a.guard ?? null);
        const caseId = a.payload?.caseId as string | undefined;
        if (caseId) {
          nodes.push({ id: caseId, type: 'case', label: caseId });
          edge(a.actionId, caseId, 'binds', a.guard ?? null);
        }
        const key = a.payload?.key as string | undefined;
        if (key) {
          const sid = `STATE:${a.payload?.scope === 'component' ? (a.payload?.componentId ?? '') : s.id}:${key}`;
          nodes.push({ id: sid, type: 'state', label: key });
          edge(a.actionId, sid, 'sets', a.guard ?? null);
        }
      }
    }
  }
  for (const a of doc.actions) {
    if (!nodes.some((n) => n.id === a.actionId)) {
      nodes.push({ id: a.actionId, type: 'action', label: a.label, kind: a.kind, guardId: a.guard ?? null });
    }
  }
  for (const g of doc.guards) {
    if (!nodes.some((n) => n.id === g.id)) nodes.push({ id: g.id, type: 'state', label: g.name });
  }

  return JSON.stringify({
    kind: 'GemArc.PrototypeActionGraph',
    schemaVersion: '1.1.0',
    generatedFrom: { revision: doc.meta.revision, updatedAt: doc.meta.updatedAt, origin: doc.meta.origin },
    counts: { nodes: nodes.length, edges: edges.length },
    nodes,
    edges,
  }, null, 2);
}

export interface StateTransitionRow {
  id: string;
  scope: 'screen' | 'component' | 'global';
  owner: string;
  from: string;
  event: string;
  eventLabel: string;
  guard: string;
  to: string;
  note: string;
}

/** StateTransitionTable —— 状态转移表 */
export function serializeStateTransitionTable(doc: ProtoDoc): string {
  const rows: StateTransitionRow[] = [];
  let seq = 0;
  for (const s of doc.screens) {
    const initialStates = Object.entries(s.states ?? {});
    for (const a of allActionsOfScreen(doc, s.id)) {
      const key = (a.payload?.key as string | undefined) ?? (a.nextState ? 'nextState' : undefined);
      if (!key) continue;
      const scope = (a.payload?.scope as 'screen' | 'component' | 'global' | undefined) ?? 'screen';
      const owner = scope === 'component' ? String(a.payload?.componentId ?? '') : s.id;
      const from = scope === 'component'
        ? String((a.payload?.from as string | undefined) ?? '*')
        : String(initialStates.find(([k]) => k === key)?.[1] ?? '*');
      seq++;
      rows.push({
        id: `TR-${String(seq).padStart(3, '0')}`,
        scope,
        owner,
        from,
        event: a.actionId,
        eventLabel: a.label,
        guard: a.guard ?? '',
        to: String(a.payload?.value ?? a.nextState ?? '?'),
        note: `${s.id} · ${a.kind}`,
      });
    }
  }
  return JSON.stringify({
    kind: 'GemArc.PrototypeStateTransitionTable',
    schemaVersion: '1.1.0',
    generatedFrom: { revision: doc.meta.revision, updatedAt: doc.meta.updatedAt },
    states: doc.states,
    transitions: rows,
  }, null, 2);
}

/* ── 导入 ─────────────────────────────────────────────────────────── */

export interface ImportResult {
  ok: boolean;
  doc?: ProtoDoc;
  error?: string;
  /** 导入后仍存在的 action 数量 (验证"导入后 Action 不丢失") */
  actionCount?: number;
}

/** 解析并校验导入的 JSON; 缺字段时补默认值, 不丢 action */
export function parseImportedDoc(text: string): ImportResult {
  let raw: unknown;
  try {
    raw = JSON.parse(text);
  } catch (e) {
    return { ok: false, error: `JSON 解析失败: ${(e as Error).message}` };
  }
  if (!raw || typeof raw !== 'object') return { ok: false, error: '顶层不是对象' };
  const obj = raw as Partial<ProtoDoc> & { kind?: string };

  if (!Array.isArray(obj.screens)) return { ok: false, error: '缺少 screens 数组' };
  if (!Array.isArray(obj.components)) return { ok: false, error: '缺少 components 数组' };
  if (!Array.isArray(obj.actions)) return { ok: false, error: '缺少 actions 数组' };

  const base = emptyDoc(obj.meta?.note ?? '');
  const doc: ProtoDoc = {
    ...base,
    meta: { ...base.meta, ...(obj.meta ?? {}) },
    pages: Array.isArray(obj.pages) ? obj.pages : base.pages,
    screens: obj.screens,
    components: obj.components,
    actions: obj.actions,
    guards: Array.isArray(obj.guards) ? obj.guards : base.guards,
    states: Array.isArray(obj.states) ? obj.states : base.states,
    fixtures: obj.fixtures ?? {},
    mockFixtures: Array.isArray(obj.mockFixtures) ? obj.mockFixtures : base.mockFixtures,
    routes: Array.isArray(obj.routes) ? obj.routes : base.routes,
    variants: Array.isArray(obj.variants) ? obj.variants : base.variants,
    links: Array.isArray(obj.links) ? obj.links : base.links,
    caseBindings: Array.isArray(obj.caseBindings) ? obj.caseBindings : base.caseBindings,
    log: Array.isArray(obj.log) ? obj.log : [],
  };
  doc.meta.revision = (obj.meta?.revision ?? 1) + 1;
  doc.meta.updatedAt = nowIso();
  return { ok: true, doc, actionCount: doc.actions.length };
}
