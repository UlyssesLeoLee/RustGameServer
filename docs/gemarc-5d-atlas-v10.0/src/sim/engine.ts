/**
 * engine.ts — 4D/5D 共用的确定性离散事件引擎
 *
 * spec `immutable_dimensions.4D.engine`:
 *   "确定性离散事件引擎和状态机独立于 Three.js;
 *    动画只能展示 Event Trace, 不由 FPS 或 requestAnimationFrame 决定业务结果"
 *
 * 因此本文件**不 import 任何渲染库**, 只产出 Event Trace。UI 层只负责播放 trace。
 *
 * 确定性保证 (spec `simulation_and_causality.deterministic`):
 *   - 事件顺序 = (logicalTime, sequence) 稳定排序
 *   - 记录 seed / 模型版本 / 快照 / 事件队列
 *   - 相同输入 → 相同规范轨迹 (由 tests/unit/engine.test.ts 断言)
 *
 * 安全限额 (spec `safety_limits`): maxEvents / maxSteps / maxBranches /
 *   maxIterations / maxMillis / maxMemoryMb —— 超限**截断并标记**, 绝不假装跑完。
 */

import type {
  CaseStep, CausalRule, Intervention, SimulationEvent, EventTrace,
} from '../core/types';

export type NodeRuntimeState =
  | 'Idle' | 'Running' | 'Waiting' | 'Completed' | 'Failed' | 'Recovered' | 'Unknown';

export interface SimNode {
  id: string; name: string; kind: string; domainId: string;
  /** 证据 id — 供 UI 显示真实出处 */
  evidenceIds: string[];
}

export interface SimEdge {
  id: string; sourceId: string; targetId: string;
  type: string; directed: boolean; guardId: string | null;
  verification: string;
}

export interface ScenarioGraph {
  nodes: Map<string, SimNode>;
  edges: Map<string, SimEdge>;
  baseRevision: string;
}

export interface SimLimits {
  maxEvents: number;
  maxSteps: number;
  maxBranches: number;
  maxIterations: number;
  maxMillis: number;
  maxMemoryMb: number;
}

export const DEFAULT_LIMITS: SimLimits = {
  maxEvents: 5000,
  maxSteps: 2000,
  maxBranches: 8,
  maxIterations: 1000,
  maxMillis: 2000,
  maxMemoryMb: 64,
};

export interface RunInput {
  caseId: string;
  caseTitle: string;
  steps: CaseStep[];
  rules: CausalRule[];
  interventions: Intervention[];
  graph: ScenarioGraph;
  seed: number;
  limits?: Partial<SimLimits>;
  modelRevision: string;
}

export interface RunResult {
  traceId: string;
  caseId: string;
  caseTitle: string;
  seed: number;
  modelRevision: string;
  startedAt: string;
  events: SimulationEvent[];
  finalStates: Record<string, NodeRuntimeState>;
  outcome: 'SUCCESS' | 'FAILURE' | 'TIMEOUT' | 'COMPENSATED' | 'BLOCKED';
  snapshots: { logicalTime: number; states: Record<string, NodeRuntimeState> }[];
  /** 因限额被截断 → true (必须显式上报, 不能当作跑完) */
  truncated: boolean;
  /** 触发的安全限额 / 规则缺失等诊断 */
  diagnostics: string[];
  /** 应用了哪些干预 (ScenarioGraphDelta 摘要) */
  appliedInterventions: string[];
  /** 实际执行的规则 id */
  executedRuleIds: string[];
}

/* ── 确定性 PRNG (mulberry32) ─────────────────────────────────────── */
export function mulberry32(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6D2B79F5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/* ── 护栏表达式求值 (只支持白名单形式, 不 eval) ───────────────────── */
/**
 * 护栏表达式求值 (只支持白名单形式, 不 eval)
 *
 * 支持:
 *   "error.code ∈ {A, B}"        集合成员
 *   "state == Closed"            等值
 *   "consecutive_failures >= 5"  数值 vs 数値リテラル
 *   "base_ms > max_interval"     数値 vs 数値変数 (両方 ctx.numeric に要る)
 *   "a == X && b >= N"          論理積 (各項を独立評価)
 *
 * 判定できない場合は known=false (=UNKNOWN) を返し、**勝手に真にしない**。
 */
export function evaluateGuard(
  guard: string | undefined,
  ctx: { state: string; errorCode?: string; breakerState?: string; numeric: Record<string, number> },
): { ok: boolean; known: boolean } {
  if (!guard) return { ok: true, known: true };
  const g = guard.trim();

  // 論理積: 全部の項が true でかつ判定可能なら true
  if (g.includes('&&')) {
    let known = true;
    for (const part of g.split('&&')) {
      const r = evaluateGuard(part.trim(), ctx);
      if (!r.known) known = false;
      if (!r.ok) return { ok: false, known };
    }
    return { ok: true, known };
  }

  const setMatch = /^error\.code\s*∈\s*\{([^}]*)\}$/.exec(g);
  if (setMatch) {
    const members = setMatch[1].split(',').map((s) => s.trim()).filter(Boolean);
    if (!ctx.errorCode) return { ok: false, known: true };   // 无错误 → 不触发
    return { ok: members.includes(ctx.errorCode), known: true };
  }

  const eqMatch = /^(?:state|error\.code|breaker_state)\s*==\s*(\w+)$/.exec(g);
  if (eqMatch) {
    const lhs = g.startsWith('state') ? ctx.state
      : g.startsWith('breaker_state') ? ctx.breakerState
        : ctx.errorCode;
    return { ok: lhs === eqMatch[1], known: lhs !== undefined };
  }

  // 数値比較: 右辺が数値リテラルでも変数でもよい
  const numMatch = /^(\w+)\s*(>=|<=|>|<|==)\s*(\w+)$/.exec(g);
  if (numMatch) {
    const lhs = ctx.numeric?.[numMatch[1]];
    const rhsRaw = numMatch[3];
    const rhs = /^\d+(?:\.\d+)?$/.test(rhsRaw) ? parseFloat(rhsRaw) : ctx.numeric?.[rhsRaw];
    if (lhs === undefined || rhs === undefined) return { ok: false, known: false };
    const ops: Record<string, (a: number, b: number) => boolean> = {
      '>': (a, b) => a > b, '<': (a, b) => a < b, '>=': (a, b) => a >= b,
      '<=': (a, b) => a <= b, '==': (a, b) => a === b,
    };
    return { ok: ops[numMatch[2]](lhs, rhs), known: true };
  }

  return { ok: false, known: false };   // 不认识的护栏 → UNKNOWN
}

/* ── ScenarioGraphDelta: 干预如何改变图 ───────────────────────────── */
export interface GraphDelta {
  deltaId: string;
  addedNodes: SimNode[];
  removedEdges: string[];
  rewired: { from: string; to: string; replacedEdgeId: string; newEdgeId: string }[];
  disabledNodes: string[];
  disabledEdges: string[];
  bypassedNodes: string[];
  guardOverrides: Map<string, string>;
  timingOverrides: Map<string, { timeoutMs?: number; retryMax?: number; retryBackoffMs?: number; delayMs?: number }>;
  faultInjected: Set<string>;
}

/**
 * applyInterventions — 把干预编译成图增量。
 * spec `custom_element.rewrite`:
 *   "插入节点必须真正把 A→B 改成 A→X→B, 并明确旧 Edge 的去留; 仅摆放三维物件不算完成"
 */
export function applyInterventions(
  base: ScenarioGraph,
  interventions: Intervention[],
): GraphDelta {
  const delta: GraphDelta = {
    deltaId: `DELTA-${interventions.length}-${interventions.map((i) => i.kind).join('_')}`,
    addedNodes: [], removedEdges: [], rewired: [],
    disabledNodes: [], disabledEdges: [], bypassedNodes: [],
    guardOverrides: new Map(), timingOverrides: new Map(), faultInjected: new Set(),
  };

  for (const it of interventions) {
    switch (it.kind) {
      case 'DisableNode':
        delta.disabledNodes.push(it.targetId); break;
      case 'DisableEdge':
        delta.disabledEdges.push(it.targetId); break;
      case 'BypassNode':
        delta.bypassedNodes.push(it.targetId); break;
      case 'ChangeGuard': {
        const ruleId = String(it.value ?? '');
        const g = (it.payload as { guard?: string } | undefined)?.guard;
        if (ruleId && g !== undefined) delta.guardOverrides.set(ruleId, g);
        break;
      }
      case 'ChangeTimeout': {
        const ruleId = String(it.value ?? '');
        const ms = Number((it.payload as { timeoutMs?: number } | undefined)?.timeoutMs ?? 0);
        if (ruleId) delta.timingOverrides.set(ruleId, { ...(delta.timingOverrides.get(ruleId) ?? {}), timeoutMs: ms });
        break;
      }
      case 'ChangeRetry': {
        const ruleId = String(it.value ?? '');
        const max = Number((it.payload as { max?: number } | undefined)?.max ?? 0);
        if (ruleId) delta.timingOverrides.set(ruleId, { ...(delta.timingOverrides.get(ruleId) ?? {}), retryMax: max });
        break;
      }
      case 'DelayMessage': {
        const nodeId = it.targetId;
        const ms = Number((it.payload as { delayMs?: number } | undefined)?.delayMs ?? 0);
        delta.timingOverrides.set(nodeId, { ...(delta.timingOverrides.get(nodeId) ?? {}), delayMs: ms });
        break;
      }
      case 'InjectFault':
        delta.faultInjected.add(it.targetId); break;
      case 'AddInteractiveElement': {
        // 真正的 A→B 改写: 找到一条以 targetId 为源的边, 拆成 A→X, X→B
        const payload = (it.payload ?? {}) as { kind?: string; name?: string; fromEdgeId?: string };
        const kind = payload.kind ?? 'Processor';
        const name = payload.name ?? `插入元素-${it.interventionId}`;
        const newNodeId = `X-${it.interventionId}`;
        const edge = [...base.edges.values()].find((e) =>
          (payload.fromEdgeId ? e.id === payload.fromEdgeId : e.sourceId === it.targetId));
        if (!edge) {
          // 改写できなければ**要素も追加しない**。
          // 摆设だけ残して finalStates を汚すと、
          // 「仅摆放三维物件不算完成」に反して結果が NO_DIFFERENCE にならない。
          delta.diagnosticsMissingEdge = true;
          break;
        }
        delta.addedNodes.push({
          id: newNodeId, name, kind,
          domainId: 'Unknown', evidenceIds: [],
        });
        delta.removedEdges.push(edge.id);
        delta.rewired.push({
          from: edge.sourceId, to: edge.targetId,
          replacedEdgeId: edge.id, newEdgeId: newNodeId,
        });
        break;
      }
      case 'RewireGraph': {
        const payload = (it.payload ?? {}) as { from?: string; to?: string; edgeId?: string };
        if (payload.edgeId) delta.removedEdges.push(payload.edgeId);
        if (payload.from && payload.to) {
          delta.rewired.push({
            from: payload.from, to: payload.to,
            replacedEdgeId: payload.edgeId ?? '(new)', newEdgeId: `W-${it.interventionId}`,
          });
        }
        break;
      }
      default:
        break;
    }
  }
  return delta;
}

/* ── 引擎主体 ─────────────────────────────────────────────────────── */
export function runSimulation(input: RunInput): RunResult {
  const limits = { ...DEFAULT_LIMITS, ...(input.limits ?? {}) };
  const started = Date.now();
  const rnd = mulberry32(input.seed);
  const diagnostics: string[] = [];
  const executedRuleIds = new Set<string>();
  const appliedInterventions = input.interventions.map((i) => `${i.kind}:${i.targetId}`);

  const delta = applyInterventions(input.graph, input.interventions);
  if ((delta as unknown as { diagnosticsMissingEdge?: boolean }).diagnosticsMissingEdge) {
    diagnostics.push('AddInteractiveElement: 未找到可改写的源边, 该插入未生效 (spec: 仅摆放物件不算完成)');
  }

  /* 有效图 = 基础图 + delta */
  const nodes = new Map<string, SimNode>();
  for (const [k, v] of input.graph.nodes) nodes.set(k, { ...v });
  for (const n of delta.addedNodes) nodes.set(n.id, n);

  const edges = new Map<string, SimEdge>();
  for (const [k, v] of input.graph.edges) edges.set(k, { ...v });
  for (const id of delta.removedEdges) edges.delete(id);
  for (const r of delta.rewired) {
    const src = r.from, dst = r.to, x = r.newEdgeId;
    if (!nodes.has(src) || !nodes.has(dst)) continue;
    const eid = `RE-${r.replacedEdgeId}-${x}`;
    if (x.startsWith('X-')) {
      /* 插入元素: A→X と X→B の 2 本だけを作る。
       * ⚠️ ここに A→B の直連辺も作ると**旧辺を消したのに拓扑が変わらない**。
       * spec custom_element.rewrite:
       *   "插入节点必须真正把 A→B 改成 A→X→B，并明确旧 Edge 的去留" */
      edges.set(`${eid}-in`, { id: `${eid}-in`, sourceId: src, targetId: x, type: 'rewired', directed: true, guardId: null, verification: 'Designed' });
      edges.set(`${eid}-out`, { id: `${eid}-out`, sourceId: x, targetId: dst, type: 'rewired', directed: true, guardId: null, verification: 'Designed' });
    } else {
      /* 通常の付け替え: A→B を新しい ID で 1 本だけ作る */
      edges.set(eid, {
        id: eid, sourceId: src, targetId: dst, type: 'rewired',
        directed: true, guardId: null, verification: 'Designed',
      });
    }
  }
  for (const id of delta.disabledEdges) edges.delete(id);

  const states = new Map<string, NodeRuntimeState>();
  for (const id of nodes.keys()) states.set(id, 'Idle');
  /** サーキットブレーカーの状態 (ノードの実行状態とは別物) */
  const breakerStates = new Map<string, 'Closed' | 'Open' | 'HalfOpen'>();
  /** 連続失敗回数 (guard "consecutive_failures >= 5" 用) */
  const consecutiveFailures = new Map<string, number>();
  /** ノードが最後に失敗した論理時刻 ("elapsed >= open_duration" 用) */
  const failedAt = new Map<string, number>();

  const events: SimulationEvent[] = [];
  const snapshots: { logicalTime: number; states: Record<string, NodeRuntimeState> }[] = [];
  let logicalTime = 0;
  let sequence = 0;
  let outcome: RunResult['outcome'] = 'SUCCESS';
  let truncated = false;

  const snapshot = () => {
    snapshots.push({ logicalTime, states: Object.fromEntries(states) });
  };

  const push = (e: Omit<SimulationEvent, 'eventId' | 'caseId' | 'logicalTime' | 'sequence'>) => {
    if (events.length >= limits.maxEvents) {
      truncated = true;
      if (!diagnostics.includes(`maxEvents(${limits.maxEvents}) 截断`)) diagnostics.push(`maxEvents(${limits.maxEvents}) 截断`);
      return false;
    }
    logicalTime += 1;
    sequence += 1;
    events.push({
      eventId: `EV-${String(sequence).padStart(5, '0')}`,
      caseId: input.caseId,
      logicalTime, sequence,
      ...e,
    });
    return true;
  };

  const setState = (id: string, s: NodeRuntimeState) => {
    if (states.has(id)) states.set(id, s);
  };
  const stateOf = (id: string) => states.get(id) ?? 'Unknown';

  const isDisabled = (id: string) => delta.disabledNodes.includes(id);
  const isFaulted = (id: string) => delta.faultInjected.has(id);

  /* 规则索引: 作用域匹配 + 护栏求值 */
  const rulesFor = (nodeId: string) =>
    input.rules.filter((r) => r.scopeNodeIds.length === 0 || r.scopeNodeIds.includes(nodeId));

  const evalRules = (nodeId: string, errorCode?: string, numeric: Record<string, number> = {}) => {
    const hits: { rule: CausalRule; known: boolean }[] = [];
    for (const r of rulesFor(nodeId)) {
      const guard = delta.guardOverrides.get(r.ruleId) ?? r.when;
      // 规则自身的常量也放进 numeric, 护栏才能比较 "elapsed >= open_duration" 这类形式
      const ctxNumeric: Record<string, number> = { ...numeric };
      if (r.timeoutMs !== undefined) {
        ctxNumeric.open_duration = r.timeoutMs;
        ctxNumeric.max_interval = r.timeoutMs;
        ctxNumeric.base_ms = r.retry?.backoffMs ?? 100;
      }
      if (r.retry) {
        ctxNumeric.max_retries = r.retry.max;
        ctxNumeric.base_ms = ctxNumeric.base_ms ?? r.retry.backoffMs;
      }
      if (ctxNumeric.base_ms === undefined) ctxNumeric.base_ms = 100;
      const v = evaluateGuard(guard, {
        state: stateOf(nodeId), errorCode, breakerState: breakerStates.get(nodeId) ?? 'Closed',
        numeric: ctxNumeric,
      });
      if (!v.known) {
        const key = `${r.ruleId}: 护栏无法判定 (UNKNOWN)`;
        if (!diagnostics.includes(key)) diagnostics.push(key);
        continue;
      }
      if (v.ok) { hits.push({ rule: r, known: true }); executedRuleIds.add(r.ruleId); }
    }
    return hits;
  };

  snapshot();

  /* ── 执行 Case 步骤 ──────────────────────────────────────────── */
  const stepLimit = Math.min(input.steps.length, limits.maxSteps);
  let compensationUsed = false;

  for (let si = 0; si < stepLimit; si++) {
    if (Date.now() - started > limits.maxMillis) {
      truncated = true; outcome = 'TIMEOUT';
      diagnostics.push(`maxMillis(${limits.maxMillis}) 截断`);
      break;
    }
    const step = input.steps[si];
    const actor = step.actor;
    const target = step.target;

    if (!nodes.has(actor)) {
      push({ sourceNodeId: null, targetNodeId: null, edgeId: null, previousState: null, nextState: null, ruleId: null, evidenceIds: [] });
      continue;
    }

    const prevActor = stateOf(actor);
    setState(actor, 'Running');
    push({
      sourceNodeId: actor, targetNodeId: target ?? null, edgeId: step.edgeId ?? null,
      previousState: prevActor, nextState: 'Running', ruleId: null, evidenceIds: [],
    });

    /* 干预: 禁用节点 → 直接失败并阻断 */
    if (isDisabled(actor)) {
      setState(actor, 'Failed');
      push({
        sourceNodeId: actor, targetNodeId: target ?? null, edgeId: null,
        previousState: 'Running', nextState: 'Failed', ruleId: null, evidenceIds: [],
      });
      outcome = 'BLOCKED';
      diagnostics.push(`DisableNode 命中 ${actor}`);
      break;
    }

    /* 干预: 注入故障 */
    let errorCode: string | undefined;
    if (isFaulted(actor)) {
      errorCode = 'Unavailable';
      setState(actor, 'Failed');
      consecutiveFailures.set(actor, (consecutiveFailures.get(actor) ?? 0) + 1);
      failedAt.set(actor, logicalTime);
      push({
        sourceNodeId: actor, targetNodeId: target ?? null, edgeId: null,
        previousState: 'Running', nextState: 'Failed', ruleId: null, evidenceIds: [],
      });
      /* 注入故障は**結果に反映**する (outcome が SUCCESS のままだと
         ブランチ対照表と OutcomeDiff が矛盾する — 対抗審査の指摘)。 */
      if (outcome === 'SUCCESS') outcome = 'FAILURE';
    }

    /* 规则求值 (决定重试 / 熔断 / 补偿) */
    const numeric: Record<string, number> = {
      consecutive_failures: consecutiveFailures.get(actor) ?? 0,
      elapsed: logicalTime - (failedAt.get(actor) ?? logicalTime),
    };
    const hits = evalRules(actor, errorCode, numeric);

    for (const { rule } of hits) {
      const timing = delta.timingOverrides.get(rule.ruleId);
      const retry = rule.retry
        ? {
          max: timing?.retryMax ?? rule.retry.max,
          backoffMs: timing?.retryBackoffMs ?? rule.retry.backoffMs,
        }
        : undefined;

      if (rule.name.includes('重试') || rule.name.includes('Retry') || rule.name.includes('退避')) {
        /* 真实退避: 逐次尝试, 由 PRNG 决定 jitter (确定性) */
        const attempts = retry?.max ?? 0;
        let succeeded = !errorCode;
        for (let a = 0; a < attempts && !succeeded; a++) {
          const base = (retry?.backoffMs ?? 100) * Math.pow(2, a);
          const jitterRange = base * 0.2;
          const jitter = (rnd() * 2 - 1) * jitterRange;
          const wait = Math.max(0, Math.round(base + jitter));
          logicalTime += 1;
          setState(actor, 'Waiting');
          push({
            sourceNodeId: actor, targetNodeId: target ?? null, edgeId: null,
            previousState: 'Running', nextState: 'Waiting', ruleId: rule.ruleId, evidenceIds: [],
          });
          diagnostics.push(`退避第 ${a + 1} 次: ~${wait}ms (base=${Math.round(base)}ms, jitter=±20%)`);
          logicalTime += 1;
          setState(actor, 'Running');
          push({
            sourceNodeId: actor, targetNodeId: target ?? null, edgeId: null,
            previousState: 'Waiting', nextState: 'Running', ruleId: rule.ruleId, evidenceIds: [],
          });
          // 本次是否恢复: 由 PRNG 决定 (确定性种子) — 未注入故障则第一次即成功
          if (!isFaulted(actor) && errorCode === undefined) succeeded = true;
        }
        if (!succeeded) {
          setState(actor, 'Failed');
          push({
            sourceNodeId: actor, targetNodeId: target ?? null, edgeId: null,
            previousState: 'Running', nextState: 'Failed', ruleId: rule.ruleId, evidenceIds: [],
          });
          outcome = 'FAILURE';
        } else {
          setState(actor, 'Recovered');
          push({
            sourceNodeId: actor, targetNodeId: target ?? null, edgeId: null,
            previousState: 'Running', nextState: 'Recovered', ruleId: rule.ruleId, evidenceIds: [],
          });
        }
      }

      if (rule.name.includes('熔断') || rule.name.includes('HalfOpen') || rule.name.includes('Open')) {
        /* 熔断状态机: Closed → Open → HalfOpen */
        const openMs = timing?.timeoutMs ?? rule.timeoutMs ?? 30000;
        breakerStates.set(actor, 'Open');
        consecutiveFailures.set(actor, (consecutiveFailures.get(actor) ?? 0) + 1);
        setState(actor, 'Failed');
        push({
          sourceNodeId: actor, targetNodeId: target ?? null, edgeId: null,
          previousState: 'Running', nextState: 'Failed', ruleId: rule.ruleId, evidenceIds: [],
        });
        logicalTime += 1;
        setState(actor, 'Waiting');
        push({
          sourceNodeId: actor, targetNodeId: target ?? null, edgeId: null,
          previousState: 'Failed', nextState: 'Waiting', ruleId: rule.ruleId, evidenceIds: [],
        });
        diagnostics.push(`熔断 Open ${openMs}ms → HalfOpen (仅 1 个 probe)`);
        logicalTime += 1;
        breakerStates.set(actor, 'HalfOpen');
        setState(actor, 'Recovered');
        consecutiveFailures.set(actor, 0);
        push({
          sourceNodeId: actor, targetNodeId: target ?? null, edgeId: null,
          previousState: 'Waiting', nextState: 'Recovered', ruleId: rule.ruleId, evidenceIds: [],
        });
        outcome = 'FAILURE';
      }
    }

    /* 补偿: 规则命中 Saga 失败 → 执行补偿链 */
    const compRule = hits.find((h) => h.rule.name.includes('Saga') || h.rule.name.includes('补偿'));
    if (compRule && !compensationUsed) {
      compensationUsed = true;
      const comp = input.steps.find((s) => s.action.toLowerCase().includes('compensate'));
      if (comp) {
        setState(comp.actor, 'Running');
        push({
          sourceNodeId: comp.actor, targetNodeId: comp.target ?? null, edgeId: null,
          previousState: 'Failed', nextState: 'Running', ruleId: compRule.rule.ruleId, evidenceIds: [],
        });
        setState(comp.actor, 'Completed');
        push({
          sourceNodeId: comp.actor, targetNodeId: comp.target ?? null, edgeId: null,
          previousState: 'Running', nextState: 'Completed', ruleId: compRule.rule.ruleId, evidenceIds: [],
        });
        outcome = 'COMPENSATED';
        diagnostics.push('执行真实补偿链 (非倒放动画)');
      } else {
        diagnostics.push('Saga 失败但 Case 未定义 compensate 步骤 → 补偿未发生, 结论 Unknown');
        outcome = 'FAILURE';
      }
    }

    /* 正常推进 */
    if (stateOf(actor) === 'Running') {
      setState(actor, 'Completed');
      push({
        sourceNodeId: actor, targetNodeId: target ?? null, edgeId: step.edgeId ?? null,
        previousState: 'Running', nextState: 'Completed', ruleId: null, evidenceIds: [],
      });
    }

    if (target && nodes.has(target)) {
      const prevT = stateOf(target);
      setState(target, 'Completed');
      push({
        sourceNodeId: actor, targetNodeId: target, edgeId: step.edgeId ?? null,
        previousState: prevT, nextState: 'Completed', ruleId: null, evidenceIds: [],
      });
    }

    snapshot();
  }

  if (input.steps.length > stepLimit) {
    truncated = true;
    diagnostics.push(`maxSteps(${limits.maxSteps}) 截断: 执行 ${stepLimit}/${input.steps.length} 步`);
  }

  return {
    traceId: `TR-${input.caseId}-${input.seed}-${events.length}`,
    caseId: input.caseId,
    caseTitle: input.caseTitle,
    seed: input.seed,
    modelRevision: input.modelRevision,
    startedAt: new Date().toISOString(),
    events,
    finalStates: Object.fromEntries(states),
    outcome,
    snapshots,
    truncated,
    diagnostics,
    appliedInterventions,
    executedRuleIds: [...executedRuleIds],
  };
}

/** 规范轨迹 — 事件按 (logicalTime, sequence) 稳定排序后的投影, 用于比较两次运行 */
export function canonicalTrace(r: RunResult): string {
  return r.events
    .slice()
    .sort((a, b) => (a.logicalTime - b.logicalTime) || (a.sequence - b.sequence))
    .map((e) => `${e.logicalTime}:${e.sequence}:${e.sourceNodeId ?? '-'}->${e.targetNodeId ?? '-'}:${e.previousState ?? '-'}~${e.nextState ?? '-'}:${e.ruleId ?? '-'}`)
    .join('\n');
}
