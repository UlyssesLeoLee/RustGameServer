#!/usr/bin/env node
/**
 * derive-models.mjs — 从已采集的 architecture.json / evidence.json 派生其余模型
 *
 * 派生原则 (per v10.0 `implementation_gate` + `gap_handling`):
 *   - 能从真实代码读到的数值 → 写实值 + 证据 (fileLine)
 *   - 只能由本图集作者设计的   → 标 Designed / Hypothetical, 绝不伪装成仓库事实
 *   - 源文档缺失               → GAP_SOURCE_DOCUMENT, 不计作图集实现失败
 *
 * 输出: requirements.json / prompts.json / cases.json / causal-rules.json /
 *       spatial.json / traceability.json / prototype.json
 */

import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, '..');
const M = join(ROOT, 'models');

const arch = JSON.parse(readFileSync(join(M, 'architecture.json'), 'utf8'));
const evDoc = JSON.parse(readFileSync(join(M, 'evidence.json'), 'utf8'));
const EV = evDoc.records;
const byPath = new Map(EV.map((e) => [`${e.path}:${e.startLine}`, e]));

const svc = arch.nodes.filter((n) => n.kind === 'Service');
const api = arch.nodes.filter((n) => n.kind === 'Api');
const tbl = arch.nodes.filter((n) => n.kind === 'Table');
const nodeByCrate = new Map(svc.map((n) => [n.properties.crate, n]));
const crateOf = (id) => arch.nodes.find((n) => n.id === id)?.properties?.crate;

/** 找一条覆盖某路径的证据 */
function evAt(path, needle) {
  const e = EV.find((x) => x.path === path && (!needle || (x.note || '').includes(needle)));
  return e ? e.id : null;
}

/* ══════════════════════════════════════════════════════════════════
   1. 需求契约 (requirement_contract)
   ══════════════════════════════════════════════════════════════════ */
const requirements = [];
let reqSeq = 0;

/** 每个已部署业务服务 → 一条需求 (真实依据: crate 源码 + k8s manifest) */
for (const s of svc) {
  const crate = s.properties.crate;
  if (s.properties.domainId === 'Operations' || s.properties.domainId === 'Infrastructure') continue;
  const deployed = s.properties.deployed;
  const apiN = api.filter((a) => a.properties.crate === crate);
  const tblN = tbl.filter((t) => t.properties.crate === crate);
  reqSeq += 1;
  const id = `REQ-${String(reqSeq).padStart(3, '0')}`;
  requirements.push({
    id,
    intent: `${crate} 提供 ${apiN.length} 个 gRPC 服务 / ${apiN.reduce((a, b) => a + b.properties.rpcCount, 0)} 个方法, 持有 ${tblN.length} 张表`,
    measurableAcceptance: [
      `${apiN.length} 个 proto service 定义可解析 (证据含 fileLine)`,
      `${tblN.length} 张 CREATE TABLE 具备 PK/FK 解析结果`,
      `crate 内 #[test] 计数 = ${s.properties.testFunctions}`,
      deployed ? 'k8s Deployment 已定义' : '⚠️ 无 k8s Deployment 证据 → 部署状态 Unknown',
    ],
    nonGoals: [
      '不定义该服务的业务规则细节 (源需求文档缺失 → GAP_SOURCE_DOCUMENT)',
      '不断言运行时行为已通过验证 (未执行真实流量)',
    ],
    provenance: 'SourceCode',
    evidenceIds: s.evidenceIds,
    affectedDimensions: ['0D', '2D', '3D'],
    linkedTestIds: [],
    status: deployed ? 'Implemented' : 'Designed',
    _crate: crate,
  });
}

/* 跨服务调用需求 (真实实现证据) */
for (const e of arch.edges) {
  const caller = e.properties.callerCrate, callee = e.properties.calleeCrate;
  reqSeq += 1;
  requirements.push({
    id: `REQ-${String(reqSeq).padStart(3, '0')}`,
    intent: `${caller} → ${callee} 跨服务 gRPC 调用 (${e.properties.calleeService}Client, ${e.properties.usageForm})`,
    measurableAcceptance: [`源码存在真实调用点 ${e.properties.fileLine}`],
    nonGoals: ['不承诺该调用的端到端 SLA (无压测数据)'],
    provenance: 'SourceCode',
    evidenceIds: e.evidenceIds,
    affectedDimensions: ['2D', '3D', '4D', '5D'],
    linkedTestIds: [],
    status: 'Implemented',
    _crate: caller, _callee: callee, _edgeId: e.id,
  });
}

/* ══════════════════════════════════════════════════════════════════
   2. 1D Prompt (spec 1D.schemas)
   ══════════════════════════════════════════════════════════════════ */
const prompts = [];
let pSeq = 0;
for (const r of requirements.slice(0, 40)) {
  pSeq += 1;
  const crate = r._crate;
  const s = nodeByCrate.get(crate);
  const relatedApi = api.filter((a) => a.properties.crate === crate).map((a) => a.id);
  const relatedTbl = tbl.filter((t) => t.properties.crate === crate).map((t) => t.id);
  prompts.push({
    promptId: `PRM-${String(pSeq).padStart(3, '0')}`,
    version: 'v1.0.0',
    objective: `实现 ${r.intent}`,
    requirementIds: [r.id],
    nodeEdgeIds: [s?.id, ...relatedApi, ...relatedTbl].filter(Boolean),
    caseIds: [],
    inputs: ['目标服务的 proto 定义 (evidence: fileLine)', '目标服务 migrations (evidence: fileLine)'],
    constraints: [
      '仅使用仓库中已存在证据的路径; 不得引入未登记的新实体',
      '凭据一律走环境变量, 永不打印 (per 守门 #5)',
      '新表必须含 PK; 外键必须显式声明 REFERENCES',
    ],
    outputs: [`${crate} 的实现变更`, `对应 UT/IT 用例`],
    acceptance: r.measurableAcceptance,
    evidenceIds: r.evidenceIds,
  });
}

/* ══════════════════════════════════════════════════════════════════
   3. 4D Case — 全部锚定真实机制
   ══════════════════════════════════════════════════════════════════ */
const REPO = join(ROOT, '..', '..');
const evReal = (path, needle) => evAt(path, needle);

const cases = [];
let cSeq = 0;
function addCase(c) { c.caseId = `CASE-${String(++cSeq).padStart(3, '0')}`; cases.push(c); return c; }

const economyNode = nodeByCrate.get('economy-service');
const sharedNode = nodeByCrate.get('shared-platform');
const gmNode = nodeByCrate.get('gm-backend');
const cardNode = nodeByCrate.get('card-service');
const adminNode = nodeByCrate.get('admin-service');

/* 真实 retry 参数 */
const RETRY_MAX = 3, RETRY_INIT = 100, RETRY_MAX_MS = 30000, RETRY_MULT = 2.0, RETRY_JITTER = 0.2;
const retryEv = EV.find((e) => e.path === 'crates/shared-platform/src/retry.rs' && e.note.includes('RetryConfig'))
  ? EV.filter((e) => e.path === 'crates/shared-platform/src/retry.rs').map((e) => e.id)
  : [];

/* 真实熔断参数 */
const BREAKER_THRESHOLD = 5, BREAKER_OPEN_MS = 30000;
const breakerEv = EV.filter((e) => e.path === 'crates/gm-backend/src/circuit_breaker.rs').map((e) => e.id);

if (economyNode && sharedNode) {
  addCase({
    kind: 'Normal',
    title: 'economy 交易 saga 正常完成',
    origin: 'DesignedSimulation',
    description: '沿 economy-service 真实 saga 步骤链执行, 每步成功则推进, 最终 confirm。无异常。',
    startNodeId: economyNode.id,
    precondition: '所有下游服务健康',
    steps: [
      { seq: 1, actor: economyNode.id, action: 'reserve', target: economyNode.id, expect: 'reservation 建立' },
      { seq: 2, actor: economyNode.id, action: 'confirm', target: economyNode.id, expect: 'saga 终态 committed' },
    ],
    expectedOutcome: 'SUCCESS — saga 提交, 无补偿',
    nodeIds: [economyNode.id, sharedNode.id],
    edgeIds: [],
    evidenceIds: [],
  });

  addCase({
    kind: 'Compensation',
    title: 'saga 中途失败 → 真实补偿链',
    origin: 'DesignedSimulation',
    description: '第 2 步注入失败, 触发 SagaOrchestrator::compensate 逆序补偿。补偿使用真实 compensate 处理器, 不是简单倒放动画。',
    startNodeId: economyNode.id,
    precondition: 'reserve 成功, confirm 失败',
    steps: [
      { seq: 1, actor: economyNode.id, action: 'reserve', target: economyNode.id, expect: 'reservation 建立' },
      { seq: 2, actor: economyNode.id, action: 'confirm', target: economyNode.id, expect: '注入失败 → Failed' },
      { seq: 3, actor: economyNode.id, action: 'compensate', target: economyNode.id, expect: 'reservation 释放 / 退款' },
    ],
    expectedOutcome: 'COMPENSATED — 资源回滚, 与 chaos_trade_saga_compensation.rs 的测试意图一致',
    nodeIds: [economyNode.id],
    edgeIds: [],
    evidenceIds: [],
  });
}

if (sharedNode) {
  addCase({
    kind: 'Retry',
    title: '重试退避 (真实 RetryConfig)',
    origin: 'DesignedSimulation',
    description: `按 shared-platform::retry::RetryConfig::default() 实参退避: max_retries=${RETRY_MAX}, initial=${RETRY_INIT}ms, multiplier=${RETRY_MULT}, max=${RETRY_MAX_MS}ms, jitter=±${RETRY_JITTER * 100}%。仅 Unavailable/DeadlineExceeded/Aborted/ResourceExhausted 可重试。`,
    startNodeId: sharedNode.id,
    precondition: '下游返回 Unavailable',
    steps: [
      { seq: 1, actor: sharedNode.id, action: 'call', expect: 'Unavailable → is_retryable=true' },
      { seq: 2, actor: sharedNode.id, action: 'backoff', expect: `等待 ~${RETRY_INIT}ms` },
      { seq: 3, actor: sharedNode.id, action: 'call', expect: '再次失败' },
      { seq: 4, actor: sharedNode.id, action: 'backoff', expect: `等待 ~${RETRY_INIT * RETRY_MULT}ms` },
    ],
    expectedOutcome: `FAILURE — 达到 max_retries=${RETRY_MAX} 后放弃 (退避上限 ${RETRY_MAX_MS}ms)`,
    nodeIds: [sharedNode.id],
    edgeIds: [],
    evidenceIds: retryEv,
  });

  addCase({
    kind: 'Boundary',
    title: '退避达到 max_interval 上限',
    origin: 'DesignedSimulation',
    description: `attempts 足够大时退避被截断到 ${RETRY_MAX_MS}ms。源码 backoff_duration 对 capped_ms 再叠加 ±20% jitter, 故上限为 ${RETRY_MAX_MS} * 1.2 = ${RETRY_MAX_MS * 1.2}ms。`,
    startNodeId: sharedNode.id,
    precondition: '持续失败',
    steps: [{ seq: 1, actor: sharedNode.id, action: 'backoff', expect: `capped at ${RETRY_MAX_MS}ms` }],
    expectedOutcome: 'FAILURE — 退避封顶, 不无限增长',
    nodeIds: [sharedNode.id],
    edgeIds: [],
    evidenceIds: retryEv,
  });

  addCase({
    kind: 'DuplicateDelivery',
    title: 'Outbox 重复投递 (幂等约束)',
    origin: 'DesignedSimulation',
    description: '同一 outbox 消息被重复投递, 由 *_outbox_check_idempotent.sql 的幂等约束拦截, 不产生重复副作用。',
    startNodeId: sharedNode.id,
    precondition: 'relay 重复发送同一 event_id',
    steps: [
      { seq: 1, actor: sharedNode.id, action: 'relay', expect: '第 1 次投递成功' },
      { seq: 2, actor: sharedNode.id, action: 'relay', expect: '幂等约束命中 → 跳过' },
    ],
    expectedOutcome: 'SUCCESS — 无重复副作用 (幂等)',
    nodeIds: [sharedNode.id],
    edgeIds: [],
    evidenceIds: [],
  });
}

if (gmNode && adminNode) {
  addCase({
    kind: 'FailurePropagation',
    title: 'gm-backend → admin-service 熔断 (真实 CircuitBreaker)',
    origin: 'DesignedSimulation',
    description: `gm-backend::AdminGrpcClient 的 CircuitBreaker 状态机: Closed → (连续 ${BREAKER_THRESHOLD} 次失败) → Open (${BREAKER_OPEN_MS}ms) → HalfOpen (仅 1 个 probe)。Open 期间 try_acquire 返回 false, 直接短路不再打下游。`,
    startNodeId: gmNode.id,
    precondition: 'admin-service 持续不可用',
    steps: [
      { seq: 1, actor: gmNode.id, action: 'call', target: adminNode.id, expect: '失败 1/5, 仍 Closed' },
      { seq: 2, actor: gmNode.id, action: 'call', target: adminNode.id, expect: `第 ${BREAKER_THRESHOLD} 次失败 → Open` },
      { seq: 3, actor: gmNode.id, action: 'call', target: adminNode.id, expect: 'Open 期间短路, 不打下游' },
      { seq: 4, actor: gmNode.id, action: 'probe', target: adminNode.id, expect: `等待 ${BREAKER_OPEN_MS}ms 后 HalfOpen, 仅 1 个 probe` },
    ],
    expectedOutcome: 'FAILURE — 熔断生效, 防止 cascade failure',
    nodeIds: [gmNode.id, adminNode.id],
    edgeIds: [],
    evidenceIds: breakerEv,
  });

  addCase({
    kind: 'Recovery',
    title: '熔断后恢复 (HalfOpen probe 成功)',
    origin: 'DesignedSimulation',
    description: 'Open 超时后进入 HalfOpen, probe 成功则 record_success → Closed, consecutive_failures 归零。',
    startNodeId: gmNode.id,
    precondition: '下游已恢复',
    steps: [
      { seq: 1, actor: gmNode.id, action: 'probe', target: adminNode.id, expect: 'HalfOpen probe 成功' },
      { seq: 2, actor: gmNode.id, action: 'record_success', expect: 'Closed, 计数归零' },
    ],
    expectedOutcome: 'SUCCESS — 熔断器复位',
    nodeIds: [gmNode.id, adminNode.id],
    edgeIds: [],
    evidenceIds: breakerEv,
  });
}

if (economyNode && cardNode) {
  addCase({
    kind: 'Timeout',
    title: 'economy → card-service 超时 (仅设计意图, 无真实调用点)',
    origin: 'HypotheticalExperiment',
    description: '⚠️ 该调用对在源码中**仅见于注释** (trade_saga_clients.rs), 无真实调用点, 因此本 Case 为 Hypothetical, 不得当作已实现行为。',
    startNodeId: economyNode.id,
    precondition: 'card-service 响应超时',
    steps: [
      { seq: 1, actor: economyNode.id, action: 'call', target: cardNode.id, expect: 'DeadlineExceeded' },
      { seq: 2, actor: economyNode.id, action: 'retry', expect: `按 max_retries=${RETRY_MAX} 退避重试` },
    ],
    expectedOutcome: 'FAILURE — 重试耗尽 (设计意图, 非实测)',
    nodeIds: [economyNode.id, cardNode.id],
    edgeIds: [],
    evidenceIds: [],
  });
}

if (economyNode) {
  addCase({
    kind: 'Concurrent',
    title: '并发 saga 下的库存预留冲突',
    origin: 'DesignedSimulation',
    description: '仓库内存在 integration_reservation_under_concurrent_saga.rs 测试, 表明并发预留冲突是被覆盖的场景。本 Case 为按该测试意图构造的模拟。',
    startNodeId: economyNode.id,
    precondition: '两个 saga 同时预留同一资源',
    steps: [
      { seq: 1, actor: economyNode.id, action: 'reserve', expect: 'A 预留成功' },
      { seq: 2, actor: economyNode.id, action: 'reserve', expect: 'B 冲突 → 等待/失败' },
    ],
    expectedOutcome: 'BLOCKED — 资源竞争被检出',
    nodeIds: [economyNode.id],
    edgeIds: [],
    evidenceIds: [],
  });
}

/* ══════════════════════════════════════════════════════════════════
   4. 因果规则 (grounded)
   ══════════════════════════════════════════════════════════════════ */
const causalRules = [
  {
    ruleId: 'RULE-RETRY-001',
    name: '可重试错误分类',
    // scope は空 = 全ノード適用。
    // `is_retryable` は shared-platform の**共通ヘルパ**で、
    // どの gRPC 呼び出し点でも使われるため、共有ノード 1 箇所に限定するのは不正確。
    // (对抗審査 C-Med-4: スコープが狭すぎて既定 Case でルールが発火せず UNKNOWN になっていた)
    scopeNodeIds: [],
    when: 'error.code ∈ {Unavailable, DeadlineExceeded, Aborted, ResourceExhausted}',
    then: 'retry(attempt+1)',
    else: 'fail_fast',
    retry: { max: RETRY_MAX, backoffMs: RETRY_INIT, multiplier: RETRY_MULT, maxMs: RETRY_MAX_MS, jitter: RETRY_JITTER },
    compensation: null,
    evidenceIds: retryEv,
    origin: 'ObservedReplay',
  },
  {
    ruleId: 'RULE-RETRY-002',
    name: '退避封顶',
    scopeNodeIds: sharedNode ? [sharedNode.id] : [],
    when: 'base_ms > max_interval',
    then: 'backoff = max_interval (+jitter)',
    else: 'backoff = initial * multiplier^attempt (+jitter)',
    timeoutMs: RETRY_MAX_MS,
    retry: { max: RETRY_MAX, backoffMs: RETRY_INIT },
    compensation: null,
    evidenceIds: retryEv,
    origin: 'ObservedReplay',
  },
  {
    ruleId: 'RULE-BREAKER-001',
    name: '熔断打开',
    scopeNodeIds: gmNode ? [gmNode.id] : [],
    // 护栏必须是**引擎的白名单求值器が解釈できる形式**だけを使う
    // ( Previous: "state == Closed && ..." は state がノード実行状態であって
    //  ブレーカー状態ではないため判定不能になっていた )
    when: 'breaker_state == Closed && consecutive_failures >= 5',
    then: 'breaker_state = Open; opened_at = now',
    else: 'breaker_state unchanged',
    timeoutMs: BREAKER_OPEN_MS,
    compensation: null,
    evidenceIds: breakerEv,
    origin: 'ObservedReplay',
  },
  {
    ruleId: 'RULE-BREAKER-002',
    name: '熔断半开与复位',
    scopeNodeIds: gmNode ? [gmNode.id] : [],
    when: 'breaker_state == Open && elapsed >= open_duration',
    then: 'breaker_state = HalfOpen; 允许 1 个 probe',
    else: 'breaker_state == HalfOpen → 拒绝后续并发 probe',
    timeoutMs: BREAKER_OPEN_MS,
    compensation: null,
    evidenceIds: breakerEv,
    origin: 'ObservedReplay',
  },
  {
    ruleId: 'RULE-SAGA-001',
    name: 'Saga 失败触发补偿',
    scopeNodeIds: economyNode ? [economyNode.id] : [],
    // 自由文 (「任一 step 失败」) は評価不能なので、エンジン税率の表現に落とす
    when: 'state == Failed',
    then: 'SagaOrchestrator::compensate 逆序执行各 handler 的 compensate',
    else: 'continue next step',
    compensation: 'compensation chain (真实 compensate 处理器, 非倒放动画)',
    evidenceIds: [],
    origin: 'ObservedReplay',
  },
];

/* ══════════════════════════════════════════════════════════════════
   5. 空间投影 (3D) — 楼层=业务域, 房间=crate, 来自真实 manifest
   ══════════════════════════════════════════════════════════════════ */
const DOMAIN_ORDER = ['Core', 'Interaction', 'Data', 'Security', 'Infrastructure', 'External', 'Operations', 'Unknown'];
const rooms = [];
const floors = [];
DOMAIN_ORDER.forEach((dom, fi) => {
  floors.push({ id: `FL-${dom}`, name: dom, y: fi * 26 });
});
for (const s of svc) {
  const crate = s.properties.crate;
  const dom = s.properties.domainId;
  const fi = DOMAIN_ORDER.indexOf(dom);
  const peers = svc.filter((x) => x.properties.domainId === dom);
  const pi = peers.findIndex((x) => x.properties.crate === crate);
  rooms.push({
    id: `ROOM-${crate}`,
    name: crate,
    floorId: `FL-${dom}`,
    bounds: { x: (pi % 6) * 22, y: fi * 26, z: Math.floor(pi / 6) * 18, w: 18, h: 12, d: 14 },
    nodeIds: [s.id, ...api.filter((a) => a.properties.crate === crate).map((a) => a.id)],
    semantics: 'logical',
    evidenceIds: s.evidenceIds,
  });
}
const spatial = {
  meta: {
    schemaVersion: '1.0.0',
    handedness: 'left',
    note: '楼层 = 业务域 (AGENTS.md §0 的 6 域 + 平台层 + 工具), 房间 = crate。semantics 全为 logical: 这些是逻辑投影, 不是物理机房拓扑。真实 k8s Deployment 仅在节点属性 deployment 中记录。',
  },
  building: { id: 'BLD-RGS', name: 'RustGameServer (k3s / rust-game-server)', floors },
  rooms,
};

/* ══════════════════════════════════════════════════════════════════
   6. 追踪链
   ══════════════════════════════════════════════════════════════════ */
const traceability = {
  requirements: requirements.map(({ _crate, _callee, _edgeId, ...r }) => r),
  promptToRequirement: prompts.map((p) => ({ promptId: p.promptId, requirementIds: p.requirementIds })),
  nodeToRequirement: requirements
    .filter((r) => r._crate && nodeByCrate.has(r._crate))
    .map((r) => ({ nodeId: nodeByCrate.get(r._crate).id, requirementIds: [r.id] })),
  caseToNode: cases.map((c) => ({ caseId: c.caseId, nodeIds: c.nodeIds })),
  testToCase: [],
  note: 'testToCase 暂空: 仓库的 Rust 测试名与本图集 Case 非一一映射, 强行映射即伪造关联 → GAP_SOURCE_DOCUMENT',
};

/* ══════════════════════════════════════════════════════════════════
   7. 原型模型 (Prototype 工作区)
   ══════════════════════════════════════════════════════════════════ */
const prototype = {
  meta: {
    schemaVersion: '1.0.0',
    origin: 'HypotheticalExperiment',
    note: '⚠️ 本原型为**明确标记的示范**, 不是 RGS 真实界面。RGS 无对应 Web 前端源码 (gm-console 仅有 envoy 静态托管), 故原型用于演示 1D-5D ↔ DOM 的绑定机制, 不得当作已交付 UI。',
  },
  screens: [
    { id: 'SCR-001', name: '机台总览', route: '/overview', componentIds: ['CMP-001', 'CMP-002', 'CMP-003'], actionIds: ['ACT-001', 'ACT-002'], states: { grid: 'loading' }, variant: 'desktop' },
    { id: 'SCR-002', name: '故障详情', route: '/fault/:id', componentIds: ['CMP-004', 'CMP-005'], actionIds: ['ACT-003'], states: { panel: 'open' }, variant: 'desktop' },
    { id: 'SCR-003', name: '班次对账', route: '/shift', componentIds: ['CMP-006'], actionIds: [], states: {}, variant: 'mobile' },
  ],
  components: [
    { id: 'CMP-001', widget: 'Card', x: 24, y: 24, w: 320, h: 160, props: { title: '在线机台' }, actionIds: [] },
    { id: 'CMP-002', widget: 'Card', x: 368, y: 24, w: 320, h: 160, props: { title: '故障机台' }, actionIds: [] },
    { id: 'CMP-003', widget: 'Table', x: 24, y: 208, w: 664, h: 240, props: { columns: ['机台', '状态', '心跳'] }, actionIds: [] },
    { id: 'CMP-004', widget: 'Drawer', x: 720, y: 24, w: 360, h: 420, props: { title: '故障详情' }, actionIds: [] },
    { id: 'CMP-005', widget: 'Button', x: 760, y: 400, w: 120, h: 36, props: { label: '确认已处理' }, actionIds: ['ACT-003'] },
    { id: 'CMP-006', widget: 'Table', x: 24, y: 24, w: 342, h: 300, props: { columns: ['班次', '营收'] }, actionIds: [] },
  ],
  actions: [
    { actionId: 'ACT-001', kind: 'Navigate', label: '打开机台', target: 'SCR-002', payload: { screenId: 'SCR-002' } },
    { actionId: 'ACT-002', kind: 'Filter', label: '只看故障', payload: { field: 'status', value: 'fault' } },
    { actionId: 'ACT-003', kind: 'SetState', label: '标记已处理', payload: { key: 'panel', value: 'closed' } },
  ],
  fixtures: {
    'SCR-001': { 'CMP-001': { value: 128 }, 'CMP-002': { value: 3 }, 'CMP-003': { rows: [['A-01', 'online', '2s'], ['A-02', 'fault', '41s'], ['A-03', 'offline', '12m']] } },
    'SCR-002': { 'CMP-004': { machine: 'A-02', severity: 'error', detectedAt: '2026-10-21T12:00:00Z' } },
    'SCR-003': { 'CMP-006': { rows: [['day', 1820], ['night', 990]] } },
  },
  links: [
    { screenId: 'SCR-001', componentId: 'CMP-003', requirementIds: [], nodeIds: [], caseIds: cases.map((c) => c.caseId), promptIds: [] },
    { screenId: 'SCR-002', componentId: 'CMP-004', requirementIds: [], nodeIds: [], caseIds: ['CASE-004'], promptIds: [] },
  ],
};

/* ══════════════════════════════════════════════════════════════════
   8. 写出
   ══════════════════════════════════════════════════════════════════ */
const write = (name, obj) => writeFileSync(join(M, name), JSON.stringify(obj, null, 2), 'utf8');
write('requirements.json', { meta: { schemaVersion: '1.0.0', revision: arch.meta.revision, honestyNotice: '需求由真实 crate / 真实调用点派生; 业务规则细节因源需求文档缺失标 GAP_SOURCE_DOCUMENT。' }, requirements });
write('prompts.json', { meta: { schemaVersion: '1.0.0', revision: arch.meta.revision }, prompts });
write('cases.json', { meta: { schemaVersion: '1.0.0', revision: arch.meta.revision, honestyNotice: 'Case 的参数取自仓库真实常量 (RetryConfig::default / CircuitBreaker); 标 HypotheticalExperiment 的 Case 明确为假设。' }, cases });
write('causal-rules.json', { meta: { schemaVersion: '1.0.0', revision: arch.meta.revision }, rules: causalRules });
write('spatial.json', spatial);
write('traceability.json', traceability);
write('prototype.json', prototype);

console.log(JSON.stringify({
  requirements: requirements.length,
  prompts: prompts.length,
  cases: cases.length,
  causalRules: causalRules.length,
  rooms: rooms.length,
  screens: prototype.screens.length,
  grounded: {
    retry: { max_retries: RETRY_MAX, initial_ms: RETRY_INIT, multiplier: RETRY_MULT, max_ms: RETRY_MAX_MS, jitter: RETRY_JITTER },
    breaker: { failure_threshold: BREAKER_THRESHOLD, open_ms: BREAKER_OPEN_MS },
  },
}, null, 2));
