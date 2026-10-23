/**
 * diagrams/registry.ts — 14 种工程图渲染器注册表
 *
 * spec `2D.diagrams` — 每种 diagramType 必须由**对应**的渲染器绘制,
 * 不允许把全部图种当 Sequence 处理。注册表按 diagramType 精确分派。
 */

import type { DiagramRenderer, DiagramType } from './types';
import { umlClass } from './uml-class';
import { umlComponent } from './uml-component';
import { umlPackage } from './uml-package';
import { umlSequence } from './uml-sequence';
import { umlState } from './uml-state';
import { umlActivity } from './uml-activity';
import { umlDeployment } from './uml-deployment';
import { umlUseCase } from './uml-usecase';
import { er } from './er';
import { dfd } from './dfd';
import { bpmn } from './bpmn';
import { security } from './security';
import { network } from './network';
import { dataflow } from './dataflow';

export const DIAGRAMS: DiagramRenderer[] = [
  umlClass,
  umlComponent,
  umlPackage,
  umlSequence,
  umlState,
  umlActivity,
  umlDeployment,
  umlUseCase,
  er,
  dfd,
  bpmn,
  security,
  network,
  dataflow,
];

const byType = new Map<DiagramType, DiagramRenderer>();
for (const d of DIAGRAMS) {
  if (byType.has(d.type)) {
    throw new Error(`[atlas] 重复的 diagramType: ${d.type}`);
  }
  byType.set(d.type, d);
}

/** 按 diagramType 精确选择渲染器 (不 fallback 到 Sequence) */
export function rendererFor(type: DiagramType): DiagramRenderer {
  const r = byType.get(type);
  if (!r) throw new Error(`[atlas] 未注册的 diagramType: ${type}`);
  return r;
}

export const DIAGRAM_TYPES: DiagramType[] = DIAGRAMS.map((d) => d.type);
