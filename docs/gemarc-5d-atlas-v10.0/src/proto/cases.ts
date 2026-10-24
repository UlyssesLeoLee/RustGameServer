/**
 * cases.ts — 4D/5D Case ↔ 原型 DOM 状态 的映射
 *
 * spec `prototype_workbench.five_d_binding`:
 *   "5D 计算的错误/等待/成功/补偿状态必须映射到原型界面真实 DOM 状态;
 *    无绑定则明确标为 Mock"
 *
 * 数据来源: window.__ATLAS_MODELS.cases (models/cases.json, 构建期内联)。
 * 绑定来源: models/prototype.json 的 links[].caseIds —— 即模型自己声明的关系,
 *           不由本层臆造。未被任何 link 引用的 component → 显式标 Mock。
 */

import type { CaseRecord } from '../core/types';
import type { CasePhase, ProtoDoc } from './model';

export interface CaseInfo {
  caseId: string;
  kind: string;
  title: string;
  origin: string;
  description: string;
  expectedOutcome: string;
}

/** 读取内联的 Case 列表 */
export function readCases(): CaseInfo[] {
  const models = (window as unknown as { __ATLAS_MODELS?: { cases?: CaseRecord[] } }).__ATLAS_MODELS ?? {};
  return (models.cases ?? []).map((c) => ({
    caseId: c.caseId,
    kind: c.kind,
    title: c.title,
    origin: c.origin,
    description: c.description,
    expectedOutcome: c.expectedOutcome,
  }));
}

/** Case kind → 原型 DOM 相位 */
export function phaseOfCase(c: CaseInfo): CasePhase {
  switch (c.kind) {
    case 'Normal':
    case 'Recovery':
      return 'success';
    case 'Compensation':
      return 'compensating';
    case 'Retry':
    case 'Timeout':
    case 'Delay':
      return 'waiting';
    case 'FailurePropagation':
    case 'Abnormal':
      return 'error';
    case 'Concurrent':
    case 'ResourceContention':
      return 'blocked';
    case 'Boundary':
    case 'DuplicateDelivery':
    case 'OutOfOrder':
    case 'Combined':
      return c.expectedOutcome.startsWith('SUCCESS') ? 'success' : 'error';
    default:
      return 'idle';
  }
}

/** 相位 → 界面中文标签 (spec 示例: 检测中 / 已确认 / 补偿中) */
export const PHASE_LABEL: Record<CasePhase, string> = {
  idle: '待检测',
  waiting: '检测中',
  success: '已确认',
  error: '检测到故障',
  compensating: '补偿中',
  blocked: '已阻塞',
};

/** 相位 → 状态语义色 (用 CSS 令牌, 不写魔数色) */
export const PHASE_TONE: Record<CasePhase, string> = {
  idle: 'idle',
  waiting: 'waiting',
  success: 'success',
  error: 'error',
  compensating: 'compensating',
  blocked: 'blocked',
};

/**
 * 由 doc.links (模型自带) 派生 caseBindings。
 * - link.caseIds 非空 → binding = 'bound'
 * - component 无任何 link → 界面上显示 Mock 徽标 (不静默)
 */
export function buildCaseBindings(doc: ProtoDoc, cases: CaseInfo[]): void {
  const byId = new Map(cases.map((c) => [c.caseId, c]));
  const bindings: ProtoDoc['caseBindings'] = [];
  let seq = 0;
  for (const link of doc.links) {
    for (const caseId of link.caseIds) {
      const info = byId.get(caseId);
      if (!info) continue;
      seq++;
      bindings.push({
        id: `CB-${String(seq).padStart(3, '0')}`,
        caseId,
        screenId: link.screenId,
        componentId: link.componentId,
        phase: phaseOfCase(info),
        binding: 'bound',
        note: `来自 models/prototype.json links (${link.screenId}/${link.componentId})`,
        version: 1,
      });
    }
  }
  doc.caseBindings = bindings;
}

/** 某 component 是否绑定了真实 Case */
export function boundCaseIds(doc: ProtoDoc, componentId: string): string[] {
  return doc.caseBindings.filter((b) => b.componentId === componentId).map((b) => b.caseId);
}

/** 某 component 是否有任何绑定 (否则 = Mock) */
export function isBoundComponent(doc: ProtoDoc, componentId: string): boolean {
  return doc.caseBindings.some((b) => b.componentId === componentId);
}
