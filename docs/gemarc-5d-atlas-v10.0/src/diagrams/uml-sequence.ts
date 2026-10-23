/**
 * diagrams/uml-sequence.ts — UML 时序图 (spec `2D.diagrams.UML_Sequence`)
 *
 * 专有符号: 生命线 (竖虚线)、激活条 (activation bar)、
 *           alt / opt / loop / par 组合框 (interaction frame)。
 *
 * 数据来源:
 *   - 消息 = architecture.json 的真实 grpc-call 边 (按 fileLine 行号排序)
 *   - 组合框 = models/cases.json 的真实 CaseKind (alt/opt/loop/par 映射见 FRAME_MAP)
 *   - 生命线 = 真实 crate + 对应 proto service 名
 */

import type { DiagramRenderer, DiagramContext, DiagramResult } from './types';
import type { GraphEdge } from '../core/types';
import { esc, text, textLines, trunc, arrowDefs, svgRoot, frameCorner, uid } from './svg';
import { allEdges, isDesigned, edgeCalleeService, edgeFileLine } from './data';

/** 真实 CaseKind → UML 组合框类型 (显式映射, 写入图注) */
const FRAME_MAP: Record<string, 'alt' | 'opt' | 'loop' | 'par'> = {
  Abnormal: 'alt',
  FailurePropagation: 'alt',
  Timeout: 'alt',
  Compensation: 'alt',
  Boundary: 'alt',
  Recovery: 'alt',
  Normal: 'opt',
  Retry: 'loop',
  DuplicateDelivery: 'loop',
  OutOfOrder: 'loop',
  Concurrent: 'par',
  ResourceContention: 'par',
  Combined: 'par',
};

const FRAME_NOTE =
  '组合框类型由 models/cases.json 的真实 CaseKind 映射: '
  + 'alt ← Abnormal/FailurePropagation/Timeout/Compensation/Boundary/Recovery; '
  + 'opt ← Normal; loop ← Retry/DuplicateDelivery/OutOfOrder; par ← Concurrent/ResourceContention/Combined。';

const MSG_GAP = 52;
const HEAD_H = 54;
const TOP = 84;

export const umlSequence: DiagramRenderer = {
  type: 'UML_Sequence',
  label: 'UML 时序图',
  symbols: ['生命线竖虚线', '激活条', 'alt/opt/loop/par 组合框'],
  dataSource: 'architecture.json: grpc-call 边 (含 fileLine) + models/cases.json: CaseKind',
  render(ctx: DiagramContext): DiagramResult {
    const id = uid('seq');
    const notes: string[] = [FRAME_NOTE];

    // 参与生命线: 选中边优先, 否则取真实调用图全部 crate
    const selEdges = ctx.selection.edgeIds
      .map((eid) => ctx.index.edges.get(eid) ?? ctx.index.designedEdges.get(eid))
      .filter((e): e is GraphEdge => !!e);
    let edges = selEdges.length ? selEdges : allEdges(ctx).filter((e) => e.verification === 'Implemented');
    if (!edges.length) edges = allEdges(ctx);
    if (!edges.length) {
      return {
        svg: svgRoot(600, 120, 'UML 时序图', text(20, 40, '模型中没有任何调用边, 无法生成时序图。', { fill: 'var(--text-2)', 'font-size': 13 })),
        width: 600, height: 120, title: 'UML 时序图', summary: '0 条消息',
        notes: ['architecture.json 的 edges 与 designedOnlyEdges 均为空。'],
      };
    }

    // 按 fileLine 行号排序 = 真实调用点顺序
    const lineOf = (e: GraphEdge) => {
      const m = /:(\d+)/.exec(String(e.properties.fileLine ?? ''));
      return m ? Number(m[1]) : 0;
    };
    edges = [...edges].sort((a, b) => lineOf(a) - lineOf(b));

    const crates: string[] = [];
    for (const e of edges) {
      for (const c of [String(e.properties.callerCrate ?? ''), String(e.properties.calleeCrate ?? '')]) {
        if (c && !crates.includes(c)) crates.push(c);
      }
    }
    if (crates.length > 6) crates.length = 6;

    const xOf = new Map<string, number>();
    crates.forEach((c, i) => xOf.set(c, 110 + i * 168));

    const svcOfCrate = new Map<string, string>();
    for (const e of edges) {
      const cc = String(e.properties.calleeCrate ?? '');
      if (cc && !svcOfCrate.has(cc)) svcOfCrate.set(cc, String(e.properties.calleeService ?? ''));
    }

    const parts: string[] = [arrowDefs(id)];
    parts.push(text(20, 22, `UML 时序图 — ${edges.length} 条消息 / ${crates.length} 条生命线`, { 'font-size': 13, 'font-weight': 600 }));
    parts.push(text(20, 38, '消息按真实调用点 fileLine 行号排序; 实线 = Implemented, 黄色虚线 = Inferred (仅注释提及)。', {
      'font-size': 10, fill: 'var(--text-2)',
    }));

    // 生命线头 + 竖虚线
    const bottom = TOP + edges.length * MSG_GAP + 30;
    for (const c of crates) {
      const x = xOf.get(c)!;
      parts.push(`<rect x="${x - 66}" y="${TOP - HEAD_H}" width="132" height="${HEAD_H - 6}" rx="3" fill="var(--surface)" stroke="var(--edge)" stroke-width="1.2"/>`);
      parts.push(text(x, TOP - HEAD_H + 15, '«crate»', { 'font-size': 9, fill: 'var(--text-2)', 'text-anchor': 'middle' }));
      parts.push(text(x, TOP - HEAD_H + 30, trunc(c, 22), { 'font-size': 11, 'font-weight': 600, 'text-anchor': 'middle' }));
      const svc = svcOfCrate.get(c);
      if (svc) parts.push(text(x, TOP - HEAD_H + 43, trunc(svc, 24), { 'font-size': 9, fill: 'var(--info)', 'text-anchor': 'middle' }));
      parts.push(`<line x1="${x}" y1="${TOP - 6}" x2="${x}" y2="${bottom}" stroke="var(--edge)" stroke-width="1.2" stroke-dasharray="6 5"/>`);
    }

    // 消息 + 激活条
    const actStart = new Map<string, number[]>();
    edges.forEach((e, i) => {
      const y = TOP + i * MSG_GAP;
      const caller = String(e.properties.callerCrate ?? '');
      const callee = String(e.properties.calleeCrate ?? '');
      const x1 = xOf.get(caller);
      const x2 = xOf.get(callee);
      if (x1 === undefined || x2 === undefined) return;
      const designed = isDesigned(ctx, e);
      const color = designed ? 'var(--warn)' : 'var(--text)';
      const marker = designed ? `${id}-w` : `${id}-f`;
      const selfCall = caller === callee;
      if (selfCall) {
        parts.push(`<path d="M ${x1 + 6} ${y} h 26 v 14 h -26" fill="none" stroke="${color}" stroke-width="1.2" marker-end="url(#${marker})"/>`);
      } else {
        const dir = x2 > x1 ? 1 : -1;
        parts.push(`<line x1="${x1}" y1="${y}" x2="${x2 - dir * 4}" y2="${y}" stroke="${color}" stroke-width="1.3" `
          + `${designed ? 'stroke-dasharray="6 4"' : ''} marker-end="url(#${marker})"/>`);
      }
      const label = `${edgeCalleeService(e)}(${trunc(edgeFileLine(e), 26)})`;
      const lx = selfCall ? x1 + 38 : (x1 + x2) / 2;
      parts.push(text(lx, y - 6, label, { 'font-size': 9, 'text-anchor': 'middle', fill: 'var(--text)' }));
      if (e.guardId) {
        parts.push(text(lx, y + 12, `[guard ${e.guardId}]`, { 'font-size': 9, 'text-anchor': 'middle', fill: 'var(--warn)' }));
      }
      if (!selfCall) {
        const arr = actStart.get(callee) ?? [];
        arr.push(i);
        actStart.set(callee, arr);
      }
    });

    // 激活条: 每条消息在 callee 上开一段, 到下一条同 callee 的消息为止
    for (const [callee, idxs] of actStart) {
      const x = xOf.get(callee);
      if (x === undefined) continue;
      idxs.forEach((i, k) => {
        const y1 = TOP + i * MSG_GAP - 12;
        const next = idxs[k + 1];
        const y2 = next !== undefined ? TOP + next * MSG_GAP - 12 : TOP + edges.length * MSG_GAP - 12;
        parts.push(`<rect x="${x - 4}" y="${y1}" width="8" height="${Math.max(16, y2 - y1)}" fill="var(--raised)" stroke="var(--text-2)" stroke-width="1"/>`);
      });
    }

    // alt/opt/loop/par 组合框 — 来自真实 Case
    const involved = new Set<string>();
    for (const e of edges) {
      if (e.sourceId) involved.add(e.sourceId);
      if (e.targetId) involved.add(e.targetId);
    }
    const cases = ctx.aux.cases.filter((c) => c.nodeIds.some((n) => involved.has(n)));
    let frameIdx = 0;
    for (const c of cases) {
      const kind = FRAME_MAP[c.kind];
      if (!kind) continue;
      const stepRows = c.steps.map((_, si) => si);
      const y0 = TOP + Math.max(0, frameIdx) * MSG_GAP - 22;
      const h = Math.max(46, (c.steps.length + 1) * 26);
      const x0 = 40;
      const x1 = Math.max(...crates.map((cc) => xOf.get(cc)!)) + 70;
      parts.push(frameCorner(x0, y0, x1 - x0, h, `${kind} [${c.caseId}] ${c.kind}`));
      parts.push(text(x0 + 8, y0 + 26, trunc(c.title, 70), { 'font-size': 10, fill: 'var(--info)' }));
      parts.push(text(x0 + 8, y0 + 39, `期望: ${trunc(c.expectedOutcome, 60)}`, { 'font-size': 9, fill: 'var(--text-2)' }));
      frameIdx += Math.max(2, Math.ceil(c.steps.length / 1));
    }
    if (cases.length) {
      notes.push(`组合框取自 ${cases.length} 条真实 4D 用例 (${cases.map((c) => c.caseId).join(', ')}), 与本图参与节点相交。`);
    } else {
      notes.push('本图参与节点在 models/cases.json 中没有对应用例, 故未绘制 alt/opt/loop/par 组合框 (不虚构)。');
    }
    const noGuard = edges.every((e) => !e.guardId);
    if (noGuard) notes.push('所有边的 guardId 均为 null — 采集模型未记录内联 guard, 因此消息上不绘制 guard 标签。');

    const canvasW = Math.max(760, Math.max(...crates.map((cc) => xOf.get(cc)!)) + 120);
    const canvasH = Math.max(320, bottom + 40 + frameIdx * MSG_GAP);

    return {
      svg: svgRoot(canvasW, canvasH, 'UML 时序图', parts.join('')),
      width: canvasW,
      height: canvasH,
      title: 'UML 时序图',
      summary: `${edges.length} 条消息 / ${crates.length} 条生命线 / ${cases.length} 个组合框`,
      notes,
    };
  },
};
