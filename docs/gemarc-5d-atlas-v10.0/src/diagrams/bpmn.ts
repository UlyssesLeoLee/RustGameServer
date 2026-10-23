/**
 * diagrams/bpmn.ts — BPMN 流程图 (spec `2D.diagrams.BPMN`)
 *
 * 专有符号: 池 (pool) / 泳道 (lane)、事件 (细圆=开始 / 粗圆=结束 / 双圆=中间)、
 *           网关 (菱形=排他)、任务 (圆角矩形)、顺序流。
 *
 * 数据来源:
 *   - 池 = architecture.json containers 的业务域 (Floor)
 *   - 泳道 = 该域下的真实 crate
 *   - 任务 = 真实 grpc-call 边
 *   - 网关 = models/causal-rules.json 的真实 when guard
 */

import type { DiagramRenderer, DiagramContext, DiagramResult } from './types';
import type { GraphEdge } from '../core/types';
import { esc, text, textLines, trunc, arrowDefs, svgRoot, roundRect, domainVar, uid } from './svg';
import { allEdges, isDesigned, domainFacets, serviceNodes, crateOf, outbound } from './data';

const TASK_W = 160;
const TASK_H = 44;
const LANE_W = 200;
const LANE_HDR = 26;

interface Task { id: string; label: string; sub: string; lane: string; pool: string; designed: boolean; }

export const bpmn: DiagramRenderer = {
  type: 'BPMN',
  label: 'BPMN 流程图',
  symbols: ['池/泳道', '事件圆', '网关菱形', '任务圆角矩形', '顺序流'],
  dataSource: 'architecture.json: containers 业务域 + grpc-call 边 + models/causal-rules.json (when)',
  render(ctx: DiagramContext): DiagramResult {
    const id = uid('bpmn');
    const notes: string[] = [];
    const edges = allEdges(ctx);

    if (!edges.length) {
      return {
        svg: svgRoot(620, 130, 'BPMN 流程图', text(20, 40, '模型中没有调用边, 无法生成 BPMN 流程。', { fill: 'var(--text-2)', 'font-size': 13 })),
        width: 620, height: 130, title: 'BPMN 流程图', summary: '0 个任务',
        notes: ['architecture.json 中没有 grpc-call 边。'],
      };
    }

    // 任务 = 出向调用
    const tasks: Task[] = [];
    for (const e of edges) {
      const caller = String(e.properties.callerCrate ?? '');
      const svc = serviceNodes(ctx).find((s) => crateOf(s) === caller);
      if (!svc) continue;
      tasks.push({
        id: e.id,
        label: trunc(String(e.properties.calleeService ?? e.targetId ?? ''), 24),
        sub: trunc(String(e.properties.fileLine ?? ''), 28),
        lane: caller,
        pool: svc.domainId,
        designed: isDesigned(ctx, e),
      });
    }

    // 池 = 业务域
    const facets = domainFacets(ctx);
    const poolOrder = facets.length
      ? facets.map((f) => f.name).filter((p) => tasks.some((t) => t.pool === p))
      : [...new Set(tasks.map((t) => t.pool))];

    const laneOf = (pool: string) => [...new Set(tasks.filter((t) => t.pool === pool).map((t) => t.lane))].sort();

    let y = 78;
    const poolBoxes: { name: string; x: number; y: number; w: number; h: number; lanes: string[] }[] = [];
    const taskPos = new Map<string, { x: number; y: number }>();

    for (const pool of poolOrder) {
      const lanes = laneOf(pool);
      const poolTasks = tasks.filter((t) => t.pool === pool);
      const maxInLane = Math.max(1, ...lanes.map((l) => poolTasks.filter((t) => t.lane === l).length));
      const poolH = 30 + maxInLane * (TASK_H + 30) + 24;
      const poolW = LANE_HDR + lanes.length * LANE_W + 20;
      const box = { name: pool, x: 40, y, w: poolW, h: poolH, lanes };
      poolBoxes.push(box);

      lanes.forEach((l, li) => {
        const lx = box.x + LANE_HDR + li * LANE_W;
        const lt = poolTasks.filter((t) => t.lane === l);
        lt.forEach((t, ti) => {
          taskPos.set(t.id, { x: lx + (LANE_W - TASK_W) / 2, y: box.y + 44 + ti * (TASK_H + 30) });
        });
      });
      y += poolH + 22;
    }

    const parts: string[] = [arrowDefs(id)];
    parts.push(text(20, 22, `BPMN 流程图 — ${poolBoxes.length} 个池 / ${poolBoxes.reduce((n, p) => n + p.lanes.length, 0)} 条泳道 / ${tasks.length} 个任务`, {
      'font-size': 13, 'font-weight': 600,
    }));
    parts.push(text(20, 38, '池 = 业务域; 泳道 = crate; 任务 = 真实 gRPC 调用; 网关 = 因果规则的 when guard。', {
      'font-size': 10, fill: 'var(--text-2)',
    }));

    // 池 + 泳道
    for (const p of poolBoxes) {
      parts.push(`<rect x="${p.x}" y="${p.y}" width="${p.w}" height="${p.h}" rx="3" fill="none" stroke="${domainVar(p.name)}" stroke-width="1.4"/>`);
      parts.push(`<rect x="${p.x}" y="${p.y}" width="${p.w}" height="24" fill="var(--surface)" stroke="${domainVar(p.name)}" stroke-width="1.4"/>`);
      parts.push(text(p.x + 8, p.y + 16, `«pool» ${p.name}`, { 'font-size': 11, 'font-weight': 600, fill: domainVar(p.name) }));
      p.lanes.forEach((l, li) => {
        const lx = p.x + LANE_HDR + li * LANE_W;
        parts.push(`<line x1="${lx}" y1="${p.y + 24}" x2="${lx}" y2="${p.y + p.h}" stroke="var(--edge)" stroke-width="1"/>`);
        parts.push(`<line x1="${lx + LANE_W}" y1="${p.y + 24}" x2="${lx + LANE_W}" y2="${p.y + p.h}" stroke="var(--edge)" stroke-width="1"/>`);
        parts.push(`<text x="${lx + 10}" y="${p.y + p.h - 10}" font-size="10" fill="var(--text-2)" transform="rotate(-90 ${lx + 10} ${p.y + p.h - 10})">«lane» ${trunc(l, 24)}</text>`);
      });
    }

    // 任务 + 事件 + 网关
    const rulesByScope = new Map<string, string>();
    for (const r of ctx.aux.rules) {
      for (const sid of r.scopeNodeIds) rulesByScope.set(sid, r.when);
    }

    for (const p of poolBoxes) {
      p.lanes.forEach((l, li) => {
        const lx = p.x + LANE_HDR + li * LANE_W;
        const cx = lx + LANE_W / 2;
        const lt = tasks.filter((t) => t.lane === l && t.pool === p.name);
        if (!lt.length) return;
        // 开始事件
        parts.push(`<circle cx="${cx}" cy="${p.y + 36}" r="9" fill="none" stroke="var(--success)" stroke-width="1.4"/>`);
        // 网关 (真实 when guard)
        const svc = serviceNodes(ctx).find((s) => crateOf(s) === l);
        const guard = svc ? rulesByScope.get(svc.id) : undefined;
        let prevY = p.y + 36;
        let prevX = cx;
        if (guard) {
          const gy = p.y + 60;
          parts.push(`<path d="M ${cx} ${gy - 13} L ${cx + 15} ${gy} L ${cx} ${gy + 13} L ${cx - 15} ${gy} Z" `
            + `fill="var(--surface)" stroke="var(--warn)" stroke-width="1.3"/>`);
          parts.push(text(cx + 20, gy + 3, trunc(guard, 30), { 'font-size': 8, fill: 'var(--warn)' }));
          parts.push(`<line x1="${cx}" y1="${prevY + 9}" x2="${cx}" y2="${gy - 13}" stroke="var(--text-2)" stroke-width="1.2" marker-end="url(#${id}-f)"/>`);
          prevY = gy;
        }
        for (const t of lt) {
          const pos = taskPos.get(t.id)!;
          const color = t.designed ? 'var(--warn)' : 'var(--info)';
          parts.push(`<path d="${roundRect(pos.x, pos.y, TASK_W, TASK_H, 8)}" fill="var(--surface)" stroke="${color}" stroke-width="1.3" `
            + `${t.designed ? 'stroke-dasharray="5 3"' : ''}/>`);
          parts.push(text(pos.x + TASK_W / 2, pos.y + 19, t.label, { 'font-size': 10, 'text-anchor': 'middle' }));
          parts.push(text(pos.x + TASK_W / 2, pos.y + 33, t.sub, { 'font-size': 8, 'text-anchor': 'middle', fill: 'var(--text-2)' }));
          parts.push(`<line x1="${prevX}" y1="${prevY + 9}" x2="${cx}" y2="${pos.y - 4}" stroke="var(--text-2)" stroke-width="1.2" marker-end="url(#${id}-f)"/>`);
          prevX = cx;
          prevY = pos.y + TASK_H;
        }
        // 结束事件
        parts.push(`<circle cx="${cx}" cy="${prevY + 22}" r="11" fill="none" stroke="var(--fail)" stroke-width="2.6"/>`);
        parts.push(`<line x1="${cx}" y1="${prevY}" x2="${cx}" y2="${prevY + 11}" stroke="var(--text-2)" stroke-width="1.2" marker-end="url(#${id}-f)"/>`);
      });
    }

    const canvasW = Math.max(820, ...poolBoxes.map((p) => p.x + p.w + 30));
    const canvasH = Math.max(360, y + 20);

    const gwCount = [...rulesByScope.keys()].filter((sid) => {
      const s = serviceNodes(ctx).find((x) => x.id === sid);
      return s && poolOrder.includes(s.domainId);
    }).length;
    notes.push(`池/泳道来自真实业务域与 crate; ${gwCount} 个网关来自真实因果规则的 when guard, 其余泳道无 guard 故不画网关 (不虚构)。`);
    notes.push('黄色虚线任务 = Inferred 边 (仅注释提及的调用); 实线任务 = Implemented 真实调用点。');

    return {
      svg: svgRoot(canvasW, canvasH, 'BPMN 流程图', parts.join('')),
      width: canvasW,
      height: canvasH,
      title: 'BPMN 流程图',
      summary: `${poolBoxes.length} 个池 / ${poolBoxes.reduce((n, p) => n + p.lanes.length, 0)} 条泳道 / ${tasks.length} 个任务 / ${gwCount} 个网关`,
      notes,
    };
  },
};
