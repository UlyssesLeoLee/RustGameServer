/**
 * diagrams/uml-usecase.ts — UML 用例图 (spec `2D.diagrams.UML_UseCase`)
 *
 * 专有符号: 椭圆用例、火柴人主体 (actor)、关联线 (association, 无箭头)、系统边界。
 *
 * 数据来源:
 *   - 主体 = architecture.json 真实 grpc-call 边的调用方 crate
 *   - 用例 = 真实被调的 proto service (calleeService)
 *   - 关联线标签 = 真实 fileLine
 */

import type { DiagramRenderer, DiagramContext, DiagramResult } from './types';
import type { GraphEdge } from '../core/types';
import { esc, text, textLines, trunc, arrowDefs, svgRoot, uid } from './svg';
import { allEdges, isDesigned, edgeFileLine } from './data';

const EL_W = 168;
const EL_H = 56;
const ACTOR_H = 62;

function actor(x: number, y: number, name: string, sel: boolean): string {
  const stroke = sel ? 'var(--focus)' : 'var(--text-2)';
  const sw = sel ? 2 : 1.2;
  const p: string[] = [];
  p.push(`<circle cx="${x}" cy="${y}" r="9" fill="none" stroke="${stroke}" stroke-width="${sw}"/>`);
  p.push(`<line x1="${x}" y1="${y + 9}" x2="${x}" y2="${y + 30}" stroke="${stroke}" stroke-width="${sw}"/>`);
  p.push(`<line x1="${x - 14}" y1="${y + 18}" x2="${x + 14}" y2="${y + 18}" stroke="${stroke}" stroke-width="${sw}"/>`);
  p.push(`<line x1="${x}" y1="${y + 30}" x2="${x - 12}" y2="${y + 48}" stroke="${stroke}" stroke-width="${sw}"/>`);
  p.push(`<line x1="${x}" y1="${y + 30}" x2="${x + 12}" y2="${y + 48}" stroke="${stroke}" stroke-width="${sw}"/>`);
  p.push(text(x, y + ACTOR_H, trunc(name, 18), { 'font-size': 10, 'text-anchor': 'middle', 'font-weight': 600 }));
  return p.join('');
}

export const umlUseCase: DiagramRenderer = {
  type: 'UML_UseCase',
  label: 'UML 用例图',
  symbols: ['椭圆用例', '火柴人主体', '关联线', '系统边界'],
  dataSource: 'architecture.json: grpc-call 边 (调用方 crate / calleeService / fileLine)',
  render(ctx: DiagramContext): DiagramResult {
    const id = uid('uc');
    const notes: string[] = [];

    const selEdges = ctx.selection.edgeIds
      .map((eid) => ctx.index.edges.get(eid) ?? ctx.index.designedEdges.get(eid))
      .filter((e): e is GraphEdge => !!e);
    const edges = selEdges.length ? selEdges : allEdges(ctx);

    if (!edges.length) {
      return {
        svg: svgRoot(600, 120, 'UML 用例图', text(20, 40, '模型中没有调用边, 无法生成用例图。', { fill: 'var(--text-2)', 'font-size': 13 })),
        width: 600, height: 120, title: 'UML 用例图', summary: '0 个用例',
        notes: ['architecture.json 中没有 grpc-call 边。'],
      };
    }

    // 主体 = 调用方 crate; 用例 = 被调的 proto service
    const actors: string[] = [];
    const useCases: { name: string; crate: string }[] = [];
    const links: { actor: string; uc: string; edge: GraphEdge }[] = [];

    for (const e of edges) {
      const a = String(e.properties.callerCrate ?? '');
      const uc = String(e.properties.calleeService ?? '');
      const ucCrate = String(e.properties.calleeCrate ?? '');
      if (!a || !uc) continue;
      if (!actors.includes(a)) actors.push(a);
      if (!useCases.some((u) => u.name === uc)) useCases.push({ name: uc, crate: ucCrate });
      links.push({ actor: a, uc, edge: e });
    }

    actors.sort();
    useCases.sort((a, b) => a.name.localeCompare(b.name));
    if (actors.length > 5) actors.length = 5;
    if (useCases.length > 8) useCases.length = 8;

    const actorX = 90;
    const boundaryX = 250;
    const boundaryW = Math.max(420, useCases.length * (EL_W + 30) + 40);
    const useCaseY = useCases.map((_, i) => 110 + i * (EL_H + 26));
    const actorY = actors.map((_, i) => 110 + i * (ACTOR_H + 30));

    const parts: string[] = [arrowDefs(id)];
    parts.push(text(20, 22, `UML 用例图 — ${actors.length} 个主体 / ${useCases.length} 个用例 / ${links.length} 条关联`, {
      'font-size': 13, 'font-weight': 600,
    }));
    parts.push(text(20, 38, '主体 = 真实调用方 crate; 用例 = 真实被调的 proto service; 关联标签 = 真实 fileLine。', {
      'font-size': 10, fill: 'var(--text-2)',
    }));

    // 系统边界
    const boundaryH = 80 + useCases.length * (EL_H + 26);
    parts.push(`<rect x="${boundaryX}" y="70" width="${boundaryW}" height="${boundaryH}" rx="6" fill="none" stroke="var(--text-2)" stroke-width="1.3"/>`);
    parts.push(text(boundaryX + 10, 88, '«system» RGS gRPC 服务面', { 'font-size': 11, 'font-weight': 600 }));

    // 关联线
    for (const l of links) {
      const ai = actors.indexOf(l.actor);
      const ui = useCases.findIndex((u) => u.name === l.uc);
      if (ai < 0 || ui < 0) continue;
      const x1 = actorX + 14;
      const y1 = actorY[ai] + 22;
      const x2 = boundaryX + 8;
      const y2 = useCaseY[ui] + EL_H / 2;
      const designed = isDesigned(ctx, l.edge);
      parts.push(`<line x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}" stroke="${designed ? 'var(--warn)' : 'var(--text-2)'}" `
        + `stroke-width="1.1" ${designed ? 'stroke-dasharray="6 4"' : ''}/>`);
    }

    // 用例椭圆
    useCases.forEach((u, i) => {
      const x = boundaryX + 40;
      const y = useCaseY[i];
      const sel = ctx.selection.nodeIds.length > 0;
      parts.push(`<ellipse cx="${x + EL_W / 2}" cy="${y + EL_H / 2}" rx="${EL_W / 2}" ry="${EL_H / 2}" `
        + `fill="var(--surface)" stroke="${sel ? 'var(--focus)' : 'var(--info)'}" stroke-width="${sel ? 2 : 1.3}"/>`);
      parts.push(text(x + EL_W / 2, y + EL_H / 2 - 2, trunc(u.name, 24), { 'font-size': 10, 'text-anchor': 'middle' }));
      parts.push(text(x + EL_W / 2, y + EL_H / 2 + 12, trunc(u.crate, 26), { 'font-size': 8, 'text-anchor': 'middle', fill: 'var(--text-2)' }));
      const ls = links.filter((l) => l.uc === u.name).slice(0, 2);
      ls.forEach((l, k) => {
        parts.push(text(x + EL_W / 2 + 6, y + EL_H / 2 + 24 + k * 10, trunc(edgeFileLine(l.edge), 34), {
          'font-size': 7, fill: 'var(--text-2)',
        }));
      });
    });

    // 主体
    actors.forEach((a, i) => {
      parts.push(actor(actorX, actorY[i], a, ctx.selection.nodeIds.length > 0));
    });

    const canvasW = boundaryX + boundaryW + 40;
    const canvasH = Math.max(360, 70 + boundaryH + 40);

    notes.push(`用例图由 ${links.length} 条真实 grpc-call 边推导; 若边数超出上限则按 crate 名排序截断并在下方声明。`);
    notes.push('actor 集合上限 5 / 用例集合上限 8 — 超出部分未绘制, 避免不可读。');

    return {
      svg: svgRoot(canvasW, canvasH, 'UML 用例图', parts.join('')),
      width: canvasW,
      height: canvasH,
      title: 'UML 用例图',
      summary: `${actors.length} 个主体 / ${useCases.length} 个用例 / ${links.length} 条关联`,
      notes,
    };
  },
};
