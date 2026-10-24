/**
 * engine.test.ts — 4D/5D 引擎单元测试 (vitest)
 *
 * 覆盖 spec `quality_assurance.coverage` 要求的确定性/Guard/Transition 分支:
 *   - 相同 seed + 相同模型 → 相同规范轨迹 (spec regression: "事件回放 3 次结果一致")
 *   - 不同 seed 在有 jitter 的规则下可能不同, 但**同 seed 必须完全一致**
 *   - 护栏求值: 集合成员 / 等值 / 数值比较 / 不可判定 → UNKNOWN
 *   - 干预: DisableNode / InjectFault / AddInteractiveElement 真改写边
 *   - 安全限额: maxEvents / maxSteps 截断必须被标记, 不得假装跑完
 */

import { describe, it, expect } from 'vitest';
import {
  runSimulation, canonicalTrace, evaluateGuard, applyInterventions, mulberry32,
  type ScenarioGraph, type SimNode, type SimEdge,
} from '../../src/sim/engine';
import { runBranch, canonicalEquality } from '../../src/sim/causal';
import { buildIndex } from '../../src/core/graph';
import type { CaseRecord, CausalRule, Intervention, ArchitectureModel } from '../../src/core/types';

const N1: SimNode = { id: 'n1', name: 'alpha', kind: 'Service', domainId: 'Core', evidenceIds: [] };
const N2: SimNode = { id: 'n2', name: 'beta', kind: 'Service', domainId: 'Data', evidenceIds: [] };
const N3: SimNode = { id: 'n3', name: 'gamma', kind: 'Service', domainId: 'Core', evidenceIds: [] };
const E1: SimEdge = { id: 'e1', sourceId: 'n1', targetId: 'n2', type: 'grpc-call', directed: true, guardId: null, verification: 'Implemented' };
const E2: SimEdge = { id: 'e2', sourceId: 'n2', targetId: 'n3', type: 'grpc-call', directed: true, guardId: null, verification: 'Implemented' };

const graph: ScenarioGraph = {
  nodes: new Map([['n1', N1], ['n2', N2], ['n3', N3]]),
  edges: new Map([['e1', E1], ['e2', E2]]),
  baseRevision: 'test-rev',
};

const steps = [
  { seq: 1, actor: 'n1', action: 'call', target: 'n2' },
  { seq: 2, actor: 'n2', action: 'call', target: 'n3' },
];

const rules: CausalRule[] = [
  {
    ruleId: 'R-RETRY', name: '重试退避', scopeNodeIds: ['n1'],
    when: 'error.code ∈ {Unavailable}', then: 'retry', retry: { max: 3, backoffMs: 100 },
    evidenceIds: [], origin: 'ObservedReplay',
  },
];

const baseInput = {
  caseId: 'CASE-T', caseTitle: 'test', steps, rules, interventions: [],
  graph, seed: 42, modelRevision: 'test-rev',
};

describe('确定性 (spec: 事件回放 3 次结果一致)', () => {
  it('同 seed 连跑 3 次, 规范轨迹完全一致', () => {
    const a = runSimulation(baseInput);
    const b = runSimulation(baseInput);
    const c = runSimulation(baseInput);
    expect(canonicalTrace(a)).toBe(canonicalTrace(b));
    expect(canonicalTrace(b)).toBe(canonicalTrace(c));
  });

  it('同 seed 的分支比较判定为 same', () => {
    const a = runSimulation(baseInput);
    const b = runSimulation(baseInput);
    expect(canonicalEquality(a, b).same).toBe(true);
  });

  it('mulberry32 同 seed 输出序列一致', () => {
    const r1 = mulberry32(7); const r2 = mulberry32(7);
    for (let i = 0; i < 20; i++) expect(r1()).toBe(r2());
  });

  it('不同 seed 产生不同 jitter 序列', () => {
    const r1 = mulberry32(1); const r2 = mulberry32(2);
    expect(r1()).not.toBe(r2());
  });
});

describe('护栏求值', () => {
  it('集合成员: 命中', () => {
    expect(evaluateGuard('error.code ∈ {Unavailable, Timeout}', { state: 'Idle', errorCode: 'Timeout' }).ok).toBe(true);
  });
  it('集合成员: 未命中', () => {
    expect(evaluateGuard('error.code ∈ {Unavailable}', { state: 'Idle', errorCode: 'Internal' }).ok).toBe(false);
  });
  it('等值比较', () => {
    expect(evaluateGuard('state == Closed', { state: 'Closed' }).ok).toBe(true);
  });
  it('数值比较', () => {
    expect(evaluateGuard('consecutive_failures >= 5', { state: 'Idle', numeric: { consecutive_failures: 5 } }).ok).toBe(true);
  });
  it('不可判定的护栏 → known=false (UNKNOWN, 不猜)', () => {
    const r = evaluateGuard('some_unknown_expr == 1', { state: 'Idle' });
    expect(r.known).toBe(false);
    expect(r.ok).toBe(false);
  });
  it('无护栏 → 恒真', () => {
    expect(evaluateGuard(undefined, { state: 'Idle' })).toEqual({ ok: true, known: true });
  });
});

describe('干预真的改变图 (spec: 不是只改变颜色)', () => {
  it('AddInteractiveElement 必须真的拆边 A→X→B', () => {
    const it: Intervention = {
      interventionId: 'INT-1', kind: 'AddInteractiveElement', targetId: 'n1',
      label: 'insert', origin: 'HypotheticalExperiment',
      payload: { kind: 'Processor', name: 'X' },
    };
    const delta = applyInterventions(graph, [it]);
    expect(delta.addedNodes.length).toBe(1);
    expect(delta.removedEdges).toContain('e1');
    expect(delta.rewired.length).toBe(1);
  });

  it('DisableNode 让运行以 BLOCKED 结束', () => {
    const it: Intervention = {
      interventionId: 'INT-2', kind: 'DisableNode', targetId: 'n1',
      label: 'disable', origin: 'HypotheticalExperiment',
    };
    const r = runSimulation({ ...baseInput, interventions: [it] });
    expect(r.outcome).toBe('BLOCKED');
    expect(r.diagnostics.some((d) => d.includes('DisableNode'))).toBe(true);
  });

  it('InjectFault 注入故障 → 状态为 Failed', () => {
    const it: Intervention = {
      interventionId: 'INT-3', kind: 'InjectFault', targetId: 'n1',
      label: 'fault', origin: 'HypotheticalExperiment',
    };
    const r = runSimulation({ ...baseInput, interventions: [it] });
    expect(r.finalStates.n1).toBe('Failed');
  });
});

describe('安全限额 (spec: 超限必须标记, 不得假装跑完)', () => {
  it('maxEvents 截断 → truncated=true 且有诊断', () => {
    const r = runSimulation({ ...baseInput, limits: { maxEvents: 2 } });
    expect(r.truncated).toBe(true);
    expect(r.diagnostics.some((d) => d.includes('maxEvents'))).toBe(true);
  });
});

describe('A/B 因果分支 (spec: NO_DIFFERENCE 不得伪造分支)', () => {
  /** 用**真实** buildIndex 造一个最小模型 —— 不用 mock, 避免测试与真实调用路径脱钩 */
  const mkIndex = () => {
    const model = {
      meta: { schemaVersion: '1.0.0', subject: 't', repo: null, revision: 'test-rev', shortRevision: 'test-rev', branch: 'test', capturedAt: '', generator: 'test', generatedFiles: [], honestyNotice: '' },
      counts: {},
      containers: [],
      nodes: [
        { id: 'n1', kind: 'Service', name: 'alpha', qualifiedName: 'a', role: '', domainId: 'Core', parentId: null, portIds: [], properties: {}, verification: 'Designed', evidenceIds: [] },
        { id: 'n2', kind: 'Service', name: 'beta', qualifiedName: 'b', role: '', domainId: 'Data', parentId: null, portIds: [], properties: {}, verification: 'Designed', evidenceIds: [] },
        { id: 'n3', kind: 'Service', name: 'gamma', qualifiedName: 'c', role: '', domainId: 'Core', parentId: null, portIds: [], properties: {}, verification: 'Designed', evidenceIds: [] },
      ],
      edges: [
        { id: 'e1', sourceId: 'n1', targetId: 'n2', sourcePortId: null, targetPortId: null, type: 'grpc-call', directed: true, guardId: null, properties: {}, verification: 'Implemented', evidenceIds: [] },
        { id: 'e2', sourceId: 'n2', targetId: 'n3', sourcePortId: null, targetPortId: null, type: 'grpc-call', directed: true, guardId: null, properties: {}, verification: 'Implemented', evidenceIds: [] },
      ],
      ports: [],
      sourceRoles: {},
    } as unknown as ArchitectureModel;
    return buildIndex(model);
  };

  it('无干预 → NO_DIFFERENCE', () => {
    const r = runBranch({
      caseId: 'CASE-T', caseTitle: 't', steps, rules, graph, seed: 42, modelRevision: 'r',
    }, [], mkIndex());
    expect(r.explanation.conclusion).toBe('NO_DIFFERENCE');
  });

  it('注入故障 → DIFFERENT 且能定位首次分歧', () => {
    const it: Intervention = {
      interventionId: 'INT-9', kind: 'InjectFault', targetId: 'n1',
      label: 'fault', origin: 'HypotheticalExperiment',
    };
    const r = runBranch({
      caseId: 'CASE-T', caseTitle: 't', steps, rules, graph, seed: 42, modelRevision: 'r',
    }, [it], mkIndex());
    expect(r.explanation.conclusion).toBe('DIFFERENT');
    expect(r.explanation.firstDivergence).not.toBeNull();
  });

  it('结论必须附带"仅模型内因果, 非经验验证"的假设', () => {
    const r = runBranch({
      caseId: 'CASE-T', caseTitle: 't', steps, rules, graph, seed: 42, modelRevision: 'r',
    }, [], mkIndex());
    expect(r.explanation.assumptions.some((a) => a.includes('不构成'))).toBe(true);
  });
});
