/**
 * causal.ts — 5D 因果分析: A/B 分支执行、首次分歧定位、受影响子图、终止差异
 *
 * spec `simulation_and_causality.branch_execution`:
 *   1. 共同初态及输入/随机种子
 *   2. 复制 t* 时刻状态与队列
 *   3. 应用 B 世界干预
 *   4. 分别独立执行 A/B
 *   5. 定位首次事件/状态差异
 *   6. 计算受影响子图
 *   7. 分析阻断反馈补偿收敛
 *   8. 比较最终结果和解释规则证据
 *
 * spec `equality_and_knowledge`:
 *   - 无效果 → NO_DIFFERENCE
 *   - 没有执行规则 → UNKNOWN
 *   - 仅模型内因果 → **不宣称经验验证因果**
 *
 * spec `5D.prohibited`:
 *   - 预制光轨冒充因果  → 本模块只从真实 Event Trace 推导差异, 轨迹是算出来的不是画出来的
 *   - 只改变颜色不重算 → 干预后**真的重跑引擎**
 *   - 新增装饰 Node 就声称干预成功 → A→X→B 必须真改写边 (engine.applyInterventions)
 *   - 实验修改正式事实图 → 一切变更属于 ScenarioGraphDelta, 基础图只读
 */

import {
  runSimulation, canonicalTrace, type RunResult, type ScenarioGraph,
} from './engine';
import type { CausalExplanation, Intervention, CausalRule } from '../core/types';
import { findImpactSubgraph, type GraphIndex } from '../core/graph';

export interface BranchInput {
  caseId: string;
  caseTitle: string;
  steps: Parameters<typeof runSimulation>[0]['steps'];
  rules: CausalRule[];
  graph: ScenarioGraph;
  seed: number;
  modelRevision: string;
  limits?: Parameters<typeof runSimulation>[0]['limits'];
}

/**
 * runBranch — 复制同一初态与种子, 应用 B 干预后独立执行, 与 A 比较。
 * 返回 baseline / branch 两条真实 trace + 因果解释。
 */
export function runBranch(
  base: BranchInput,
  interventions: Intervention[],
  index: GraphIndex,
): { baseline: RunResult; branch: RunResult; explanation: CausalExplanation } {
  const baseline = runSimulation({ ...base, interventions: [] });
  const branch = runSimulation({ ...base, interventions });

  const a = baseline.events.slice().sort((x, y) => (x.logicalTime - y.logicalTime) || (x.sequence - y.sequence));
  const b = branch.events.slice().sort((x, y) => (x.logicalTime - y.logicalTime) || (x.sequence - y.sequence));

  /* ── 首次分歧 ─────────────────────────────────────────────── */
  let firstDivergence: CausalExplanation['firstDivergence'] = null;
  /** 分歧点の状態迁移 (A と B の nextState 差) — 表示しないと「同じ ID だけ」で読めない */
  let firstDivergenceDetail: { a: string; b: string; aEvent: string; bEvent: string } | null = null;
  const max = Math.max(a.length, b.length);
  for (let i = 0; i < max; i++) {
    const ea = a[i], eb = b[i];
    const ka = ea ? `${ea.logicalTime}:${ea.sourceNodeId}:${ea.nextState}:${ea.ruleId}` : '(无)';
    const kb = eb ? `${eb.logicalTime}:${eb.sourceNodeId}:${eb.nextState}:${eb.ruleId}` : '(无)';
    if (ka !== kb) {
      firstDivergence = {
        logicalTime: eb?.logicalTime ?? ea?.logicalTime ?? 0,
        sequence: eb?.sequence ?? ea?.sequence ?? 0,
        eventA: ea ? ea.eventId : '(无对应事件)',
        eventB: eb ? eb.eventId : '(无对应事件)',
      };
      firstDivergenceDetail = {
        a: ea ? `${ea.previousState ?? '—'} → ${ea.nextState ?? '—'}` : '(无)',
        b: eb ? `${eb.previousState ?? '—'} → ${eb.nextState ?? '—'}` : '(无)',
        aEvent: ka,
        bEvent: kb,
      };
      break;
    }
  }

  /* ── 终态差异 ─────────────────────────────────────────────── */
  /* **両分岐に存在するノードだけ**を比較する。
   * 干渉で追加されたノード (X-INT-1) は片側にしか存在しないため、
   * 把它当成"结果差异"会把"图结构变了"误报成"行为变了"。
   * 構造差分は ScenarioGraphDelta 側で提示する (spec isolation)。 */
  const terminalDiff: CausalExplanation['terminalDiff'] = {};
  const commonNodeIds = Object.keys(baseline.finalStates)
    .filter((id) => id in branch.finalStates);
  for (const id of commonNodeIds) {
    const sa = baseline.finalStates[id] ?? 'Unknown';
    const sb = branch.finalStates[id] ?? 'Unknown';
    if (sa !== sb) terminalDiff[id] = { baseline: sa, branch: sb };
  }
  /** 片側にしか存在しないノード (構造差分) — 終态差异とは区別して保持 */
  const structuralOnly = [
    ...Object.keys(branch.finalStates).filter((id) => !(id in baseline.finalStates)),
    ...Object.keys(baseline.finalStates).filter((id) => !(id in branch.finalStates)),
  ];

  /* ── 受影响节点 / 边 ───────────────────────────────────────── */
  let affectedNodeIds: string[] = [];
  let affectedEdgeIds: string[] = [];
  let propagationPath: string[] = [];

  if (firstDivergence) {
    const origin = branch.events.find((e) => e.eventId === firstDivergence!.eventB);
    if (origin?.sourceNodeId) {
      const impact = findImpactSubgraph(index, origin.sourceNodeId);
      affectedNodeIds = impact.nodeIds;
      affectedEdgeIds = impact.edgeIds;
      propagationPath = impact.nodeIds.slice(0, 12);
    }
  } else {
    affectedNodeIds = Object.keys(terminalDiff);
  }

  /* ── 结论 ─────────────────────────────────────────────────── */
  let conclusion: CausalExplanation['conclusion'];
  if (!firstDivergence && Object.keys(terminalDiff).length === 0) {
    // 追跡された行動差も終态差も無い → 干渉の効果は無かった
    conclusion = 'NO_DIFFERENCE';
  } else if (branch.executedRuleIds.length === 0 && baseline.executedRuleIds.length === 0) {
    // 差分は出たが、差を**説明するルールが 1 つも実行されていない** → 因果は不明
    conclusion = 'UNKNOWN';
  } else {
    conclusion = 'DIFFERENT';
  }

  /* ── 恢复 / 补偿 ──────────────────────────────────────────── */
  const recoveryEvent = branch.events.find((e) => e.nextState === 'Recovered');
  const compensationEvent = branch.events.find((e) =>
    e.ruleId && /saga|补偿|compensate/i.test(e.ruleId));

  const assumptions: string[] = [
    '结论仅来自本图集模型内的确定性重算, **不构成对真实生产系统因果关系的经验验证** (spec equality_and_knowledge)',
    `随机源: mulberry32(seed=${base.seed}) — 相同 seed + 相同模型 + 相同干预 → 相同轨迹`,
  ];
  if (baseline.truncated || branch.truncated) {
    assumptions.push('⚠️ 至少一条分支因安全限额被截断, 结论仅覆盖已执行部分');
  }
  if (branch.diagnostics.length) {
    assumptions.push(`B 分支诊断 ${branch.diagnostics.length} 条 (见执行面板)`);
  }
  if (!interventions.length) {
    assumptions.push('未施加任何干预 → A/B 同源, 差异只可能来自引擎自身不确定性 (应恒为 NO_DIFFERENCE)');
  }

  const evidenceIds = [...new Set([...baseline.executedRuleIds, ...branch.executedRuleIds])]
    .flatMap((ruleId) => {
      const rule = base.rules.find((r) => r.ruleId === ruleId);
      return rule?.evidenceIds ?? [];
    });

  const explanation: CausalExplanation = {
    explanationId: `CE-${base.caseId}-${base.seed}`,
    baselineCaseId: baseline.traceId,
    branchCaseId: branch.traceId,
    firstDivergence,
    firstDivergenceDetail,
    affectedNodeIds,
    affectedEdgeIds,
    propagationPath,
    recovery: recoveryEvent ? `${recoveryEvent.eventId} (${recoveryEvent.sourceNodeId} → ${recoveryEvent.nextState})` : null,
    compensation: compensationEvent ? `${compensationEvent.eventId} (rule=${compensationEvent.ruleId})` : null,
    terminalDiff,
    structuralOnlyNodeIds: structuralOnly,
    assumptions,
    evidenceIds: [...new Set(evidenceIds)],
    conclusion,
  };

  return { baseline, branch, explanation };
}

/**
 * canonicalEquality — 判定两次运行是否真的一致 (spec regression: "事件回放 3 次结果一致")
 */
export function canonicalEquality(a: RunResult, b: RunResult): { same: boolean; diffAt: number | null } {
  const ca = canonicalTrace(a).split('\n');
  const cb = canonicalTrace(b).split('\n');
  const max = Math.max(ca.length, cb.length);
  for (let i = 0; i < max; i++) {
    if ((ca[i] ?? '') !== (cb[i] ?? '')) return { same: false, diffAt: i };
  }
  return { same: true, diffAt: null };
}
