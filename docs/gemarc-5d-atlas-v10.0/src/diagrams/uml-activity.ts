/**
 * diagrams/uml-activity.ts — UML 活动图 (spec `2D.diagrams.UML_Activity`)
 *
 * 专有符号: 圆角活动、菱形判定、泳道 (swimlane)、初始/终止节点。
 *
 * 数据来源:
 *   - 活动 = 真实 grpc-call 边 (调用 / 被调用)
 *   - 判定 = models/causal-rules.json 的真实 when guard
 *   - 泳道 = architecture.json containers 的业务域 (Floor)
 */

import type { DiagramRenderer, DiagramContext, DiagramResult } from './types';
import { esc, text, textLines, trunc, arrowDefs, svgRoot, roundRect, domainVar, uid } from './svg';
import { allEdges, isDesigned, inbound, outbound, scopeFor, serviceNodes, domainFacets } from './data';

const AW = 176;
const AH = 46;
const LANE_W = 230;

export const umlActivity: DiagramRenderer = {
  type: 'UML_Activity',
  label: 'UML 活动图',
  symbols: ['圆角活动', '菱形判定', '泳道', '初始/终止圆'],
  dataSource: 'architecture.json: grpc-call 边 + containers 业务域 + models/causal-rules.json (when)',
  render(ctx: DiagramContext): DiagramResult {
    const id = uid('act');
    const notes: string[] = [];
    const scope = scopeFor(ctx, (n) => n.kind === 'Service', 12);

    if (!scope.nodes.length) {
      return {
        svg: svgRoot(600, 120, 'UML 活动图', text(20, 40, '当前选择下没有可绘制的活动。', { fill: 'var(--text-2)', 'font-size': 13 })),
        width: 600, height: 120, title: 'UML 活动图', summary: '0 个活动',
        notes: ['当前选择域内没有 Service 节点。'],
      };
    }

    // 泳道 = 业务域
    const facets = domainFacets(ctx).filter((f) => f.services.some((s) => scope.nodes.some((n) => n.id === s.id)));
    const laneNames = facets.length ? facets.map((f) => f.name) : [...new Set(scope.nodes.map((n) => n.domainId))];

    interface Act { lane: string; label: string; sub: string; kind: 'call' | 'recv' | 'decision'; id: string; }
    const acts: Act[] = [];
    for (const s of scope.nodes) {
      const lane = s.domainId;
      for (const e of outbound(ctx, s.id)) {
        if (!e.targetId) continue;
        acts.push({
          lane, kind: 'call', id: e.id,
          label: `call ${String(e.properties.calleeService ?? e.targetId)}`,
          sub: String(e.properties.fileLine ?? ''),
        });
      }
      for (const e of inbound(ctx, s.id)) {
        if (e.sourceId === s.id) continue;
        acts.push({
          lane, kind: 'recv', id: e.id,
          label: `receive ${String(e.properties.calleeService ?? e.id)}`,
          sub: String(e.properties.fileLine ?? ''),
        });
      }
    }
    // 真实判定 (规则 when)
    const scopeIds = new Set(scope.nodes.map((n) => n.id));
    for (const r of ctx.aux.rules) {
      if (!r.scopeNodeIds.some((n) => scopeIds.has(n))) continue;
      const owner = ctx.index.nodes.get(r.scopeNodeIds[0]);
      acts.push({
        lane: owner?.domainId ?? 'Unknown', kind: 'decision', id: r.ruleId,
        label: trunc(r.when, 34), sub: r.ruleId,
      });
    }

    // 泳道布局
    const laneOrder = laneNames.slice();
    const laneX = new Map<string, number>();
    laneOrder.forEach((l, i) => laneX.set(l, 20 + i * LANE_W));
    const laneActs = new Map<string, Act[]>();
    for (const a of acts) {
      if (!laneActs.has(a.lane)) laneActs.set(a.lane, []);
      laneActs.get(a.lane)!.push(a);
    }
    const maxPerLane = Math.max(1, ...[...laneActs.values()].map((v) => v.length));
    const laneH = 46 + maxPerLane * (AH + 22) + 30;

    const pos = new Map<string, { x: number; y: number }>();
    for (const [lane, list] of laneActs) {
      const x = laneX.get(lane) ?? 20;
      list.forEach((a, i) => {
        pos.set(a.id, { x: x + (LANE_W - AW) / 2, y: 70 + i * (AH + 22) });
      });
    }

    const parts: string[] = [arrowDefs(id)];
    parts.push(text(20, 22, `UML 活动图 — ${acts.length} 个活动节点 / ${laneOrder.length} 条泳道`, { 'font-size': 13, 'font-weight': 600 }));
    parts.push(text(20, 38, '活动 = 真实 gRPC 调用点; 菱形判定 = 因果规则的 when guard; 泳道 = 业务域。', {
      'font-size': 10, fill: 'var(--text-2)',
    }));

    // 泳道
    laneOrder.forEach((lane, i) => {
      const x = laneX.get(lane)!;
      parts.push(`<rect x="${x}" y="46" width="${LANE_W}" height="${laneH}" fill="none" stroke="${domainVar(lane)}" stroke-width="1.1"/>`);
      parts.push(`<rect x="${x}" y="46" width="${LANE_W}" height="22" fill="var(--surface)" stroke="${domainVar(lane)}" stroke-width="1.1"/>`);
      parts.push(text(x + 8, 61, `«lane» ${lane}`, { 'font-size': 10, 'font-weight': 600, fill: domainVar(lane) }));
    });

    // 初始 / 终止
    const firstLane = laneOrder[0];
    const fx = (laneX.get(firstLane) ?? 20) + LANE_W / 2;
    parts.push(`<circle cx="${fx}" cy="80" r="9" fill="var(--text)" stroke="var(--text)" stroke-width="1"/>`);
    const lastLane = laneOrder[laneOrder.length - 1];
    const lx = (laneX.get(lastLane) ?? 20) + LANE_W / 2;
    parts.push(`<circle cx="${lx}" cy="${46 + laneH - 16}" r="10" fill="none" stroke="var(--text)" stroke-width="1.6"/>`);

    // 活动节点
    for (const a of acts) {
      const p = pos.get(a.id)!;
      if (a.kind === 'decision') {
        const w = AW;
        const h = AH + 12;
        parts.push(`<path d="M ${p.x + w / 2} ${p.y} L ${p.x + w} ${p.y + h / 2} L ${p.x + w / 2} ${p.y + h} L ${p.x} ${p.y + h / 2} Z" `
          + `fill="var(--surface)" stroke="var(--warn)" stroke-width="1.3"/>`);
        parts.push(text(p.x + w / 2, p.y + h / 2 + 3, a.label, { 'font-size': 9, 'text-anchor': 'middle' }));
        parts.push(text(p.x + w / 2, p.y + h / 2 + 15, a.sub, { 'font-size': 8, 'text-anchor': 'middle', fill: 'var(--text-2)' }));
        continue;
      }
      const stroke = a.kind === 'call' ? 'var(--info)' : 'var(--text-2)';
      parts.push(`<path d="${roundRect(p.x, p.y, AW, AH, 10)}" fill="var(--surface)" stroke="${stroke}" stroke-width="1.2"/>`);
      parts.push(text(p.x + AW / 2, p.y + 19, trunc(a.label, 30), { 'font-size': 10, 'text-anchor': 'middle' }));
      parts.push(text(p.x + AW / 2, p.y + 33, trunc(a.sub, 32), { 'font-size': 8, 'text-anchor': 'middle', fill: 'var(--text-2)' }));
    }

    // 流: 初始 → 每个泳道首个活动; 同 lane 内顺序流
    for (const [lane, list] of laneActs) {
      const x = (laneX.get(lane) ?? 20) + LANE_W / 2;
      for (let i = 0; i < list.length; i++) {
        const p = pos.get(list[i].id)!;
        const cx = p.x + AW / 2;
        if (i === 0) {
          parts.push(`<line x1="${x}" y1="89" x2="${cx}" y2="${p.y - 4}" stroke="var(--text-2)" stroke-width="1.1" marker-end="url(#${id}-f)"/>`);
        } else {
          const prev = pos.get(list[i - 1].id)!;
          parts.push(`<line x1="${prev.x + AW / 2}" y1="${prev.y + AH}" x2="${cx}" y2="${p.y - 4}" stroke="var(--text-2)" stroke-width="1.1" marker-end="url(#${id}-f)"/>`);
        }
      }
      const lastP = pos.get(list[list.length - 1].id)!;
      const endX = (laneX.get(laneOrder[laneOrder.length - 1]) ?? 20) + LANE_W / 2;
      parts.push(`<line x1="${lastP.x + AW / 2}" y1="${lastP.y + AH}" x2="${endX}" y2="${46 + laneH - 26}" stroke="var(--text-2)" stroke-width="1.1" marker-end="url(#${id}-f)"/>`);
    }

    const canvasW = 40 + laneOrder.length * LANE_W;
    const canvasH = 46 + laneH + 30;
    notes.push(`泳道来自 containers 的业务域 Floor; 每条活动对应一条真实 grpc-call 边 (fileLine 见节点副标题)。`);
    if (scope.truncated) notes.push(scope.note);

    return {
      svg: svgRoot(canvasW, canvasH, 'UML 活动图', parts.join('')),
      width: canvasW,
      height: canvasH,
      title: 'UML 活动图',
      summary: `${acts.length} 个活动 / ${laneOrder.length} 条泳道 / ${acts.filter((a) => a.kind === 'decision').length} 个判定`,
      notes,
    };
  },
};
