/**
 * diagrams/types.ts — 专业工程图渲染器契约 (spec `immutable_dimensions.2D.diagrams`)
 *
 * 2D 的 "独立专业工程图审查单元" = 每种 diagramType 一个渲染器, 返回 SVG 字符串。
 * 渲染器只读真实模型 (models/architecture.json + cases + causal-rules),
 * 不允许编造节点/边/属性。
 */

import type { ArchitectureModel, GraphNode, GraphEdge, CaseRecord, CausalRule, Requirement } from '../core/types';
import type { GraphIndex } from '../core/graph';

/** spec `2D.diagrams` — 14 种工程图 */
export type DiagramType =
  | 'UML_Class'
  | 'UML_Component'
  | 'UML_Package'
  | 'UML_Sequence'
  | 'UML_State'
  | 'UML_Activity'
  | 'UML_Deployment'
  | 'UML_UseCase'
  | 'ER'
  | 'DFD'
  | 'BPMN'
  | 'Security'
  | 'Network'
  | 'DataFlow';

/** 画布选择状态传入渲染器的快照 */
export interface DiagramSelection {
  /** 选中的节点 id (可多选) */
  nodeIds: string[];
  /** 选中的边 id (可多选) */
  edgeIds: string[];
  /** 聚焦节点 id (单选, 驱动"聚焦"语义) */
  focusId: string | null;
}

/** 辅助模型 (来自 models/cases.json + models/causal-rules.json + models/requirements.json) */
export interface AuxModels {
  cases: CaseRecord[];
  rules: CausalRule[];
  requirements: Requirement[];
}

export interface DiagramContext {
  model: ArchitectureModel;
  index: GraphIndex;
  selection: DiagramSelection;
  /** 是否渲染 Inferred (虚线) 边 */
  showInferred: boolean;
  /** 4D 用例 / 5D 因果规则 / 需求 (供 Sequence / State / Activity / BPMN 使用) */
  aux: AuxModels;
}

/** 渲染结果 — SVG 字符串 + 尺寸 + 可读摘要 + 诚实声明 (缺口/推导规则) */
export interface DiagramResult {
  svg: string;
  width: number;
  height: number;
  title: string;
  /** 一行摘要: 该图画了什么、多少元素 */
  summary: string;
  /** 诚实说明: 推导规则 / 截断 / 已知缺口 */
  notes: string[];
}

export interface DiagramRenderer {
  type: DiagramType;
  /** 中文标签 */
  label: string;
  /** 该图种必须出现的专有符号 (验收自检用) */
  symbols: string[];
  /** 数据来源说明 (全部指向真实模型字段) */
  dataSource: string;
  render(ctx: DiagramContext): DiagramResult;
}
