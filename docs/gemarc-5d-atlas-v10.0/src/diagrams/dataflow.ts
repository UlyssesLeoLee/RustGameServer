/**
 * diagrams/dataflow.ts — 数据流图 (DataFlow 变体, spec `2D.diagrams.DataFlow`)
 *
 * 与 DFD 的区别: DFD 关注加工与数据存储的角色; DataFlow 关注
 * **数据变换链** — 源表 → 加工 → 加工 → 目标表, 每条流标注真实表名与列数。
 *
 * 专有符号: 粗数据流箭头、加工圆、数据存储 (开口平行线)、变换链。
 *
 * 数据来源:
 *   - 链路由真实 grpc-call 边驱动 (caller crate → callee crate)
 *   - 表与列数来自真实 Table 节点
 */

import type { DiagramRenderer, DiagramContext, DiagramResult } from './types';
import type { GraphNode } from '../core/types';
import { esc, text, textLines, trunc, arrowDefs, svgRoot, uid } from './svg';
import { allEdges, isDesigned, columnsOf, crateOf, tableNodes } from './data';

const ROW_H = 132;
const SRC_X = 40;
const P1_X = 300;
const P2_X = 470;
const DST_X = 640;

export const dataflow: DiagramRenderer = {
  type: 'DataFlow',
  label: '数据流图 (变换链)',
  symbols: ['粗数据流', '加工圆', '数据存储平行线', '变换链'],
  dataSource: 'architecture.json: grpc-call 边 + Table 节点 (列数真实统计)',
  render(ctx: DiagramContext): DiagramResult {
    const id = uid('df');
    const notes: string[] = [];
    const edges = allEdges(ctx);

    if (!edges.length) {
      return {
        svg: svgRoot(620, 130, '数据流图', text(20, 40, '模型中没有调用边, 无法生成数据流图。', { fill: 'var(--text-2)', 'font-size': 13 })),
        width: 620, height: 130, title: '数据流图', summary: '0 条数据流',
        notes: ['architecture.json 中没有 grpc-call 边。'],
      };
    }

    const tables = tableNodes(ctx);
    const byCrate = new Map<string, GraphNode[]>();
    for (const t of tables) {
      const c = crateOf(t);
      if (!byCrate.has(c)) byCrate.set(c, []);
      byCrate.get(c)!.push(t);
    }
    const store = (crate: string) => (byCrate.get(crate) ?? []).slice(0, 3);

    // 每条边一行 (同 caller→callee 去重)
    const seen = new Set<string>();
    const rows: { caller: string; callee: string; edge: GraphNode extends never ? never : (typeof edges)[number] }[] = [];
    for (const e of edges) {
      const caller = String(e.properties.callerCrate ?? '');
      const callee = String(e.properties.calleeCrate ?? '');
      const key = `${caller}->${callee}`;
      if (!caller || !callee || seen.has(key)) continue;
      seen.add(key);
      rows.push({ caller, callee, edge: e });
    }

    const parts: string[] = [arrowDefs(id)];
    parts.push(text(20, 22, `数据流图 (变换链) — ${rows.length} 条链 / 源表与目标表取自真实 DDL`, {
      'font-size': 13, 'font-weight': 600,
    }));
    parts.push(text(20, 38, '每行 = 一条真实 gRPC 调用驱动的数据变换链: 源表 → 调用方加工 → 被调方加工 → 目标表。', {
      'font-size': 10, fill: 'var(--text-2)',
    }));

    const storeBlock = (x: number, y: number, list: GraphNode[], label: string): string => {
      const p: string[] = [];
      p.push(text(x, y - 10, label, { 'font-size': 9, fill: 'var(--text-2)' }));
      if (!list.length) {
        p.push(text(x, y + 10, '(该 crate 无表)', { 'font-size': 8, fill: 'var(--text-2)' }));
        return p.join('');
      }
      list.forEach((t, i) => {
        const ty = y + i * 26;
        const cols = columnsOf(t);
        p.push(`<line x1="${x}" y1="${ty}" x2="${x + 150}" y2="${ty}" stroke="var(--text-2)" stroke-width="1.3"/>`);
        p.push(`<line x1="${x}" y1="${ty + 14}" x2="${x + 150}" y2="${ty + 14}" stroke="var(--text-2)" stroke-width="1.3"/>`);
        p.push(text(x + 4, ty + 11, trunc(t.name, 18), { 'font-size': 9 }));
        p.push(text(x + 150, ty + 11, `${cols.length} 列`, { 'font-size': 8, 'text-anchor': 'end', fill: 'var(--text-2)' }));
      });
      return p.join('');
    };

    rows.forEach((r, i) => {
      const y = 70 + i * ROW_H;
      const designed = isDesigned(ctx, r.edge);
      const color = designed ? 'var(--warn)' : 'var(--text-2)';

      // 行标题
      parts.push(text(20, y + 16, `链 ${i + 1}: ${trunc(r.caller, 20)} → ${trunc(r.callee, 20)}`, {
        'font-size': 10, 'font-weight': 600, fill: 'var(--text)',
      }));
      parts.push(text(20, y + 30, trunc(String(r.edge.properties.fileLine ?? ''), 30), {
        'font-size': 8, fill: 'var(--text-2)',
      }));

      // 源表 → 加工1
      parts.push(storeBlock(SRC_X, y + 16, store(r.caller), `${r.caller} 的表`));
      parts.push(`<line x1="${SRC_X + 160}" y1="${y + 24}" x2="${P1_X - 30}" y2="${y + 24}" stroke="${color}" stroke-width="2" `
        + `${designed ? 'stroke-dasharray="7 4"' : ''} marker-end="url(#${id}-f)"/>`);
      parts.push(text(SRC_X + 166, y + 20, '数据', { 'font-size': 8, fill: 'var(--text-2)' }));

      // 加工1
      parts.push(`<circle cx="${P1_X}" cy="${y + 24}" r="24" fill="var(--surface)" stroke="var(--info)" stroke-width="1.3"/>`);
      parts.push(text(P1_X, y + 21, trunc(r.caller, 12), { 'font-size': 8, 'text-anchor': 'middle' }));
      parts.push(text(P1_X, y + 32, '加工', { 'font-size': 7, 'text-anchor': 'middle', fill: 'var(--text-2)' }));

      // 加工1 → 加工2
      parts.push(`<line x1="${P1_X + 24}" y1="${y + 24}" x2="${P2_X - 24}" y2="${y + 24}" stroke="${color}" stroke-width="2" `
        + `${designed ? 'stroke-dasharray="7 4"' : ''} marker-end="url(#${id}-f)"/>`);
      parts.push(text((P1_X + P2_X) / 2, y + 20, trunc(String(r.edge.properties.calleeService ?? ''), 20), {
        'font-size': 8, 'text-anchor': 'middle', fill: 'var(--text)',
      }));

      // 加工2
      parts.push(`<circle cx="${P2_X}" cy="${y + 24}" r="24" fill="var(--surface)" stroke="var(--info)" stroke-width="1.3"/>`);
      parts.push(text(P2_X, y + 21, trunc(r.callee, 12), { 'font-size': 8, 'text-anchor': 'middle' }));
      parts.push(text(P2_X, y + 32, '加工', { 'font-size': 7, 'text-anchor': 'middle', fill: 'var(--text-2)' }));

      // 加工2 → 目标表
      parts.push(`<line x1="${P2_X + 24}" y1="${y + 24}" x2="${DST_X - 10}" y2="${y + 24}" stroke="${color}" stroke-width="2" `
        + `${designed ? 'stroke-dasharray="7 4"' : ''} marker-end="url(#${id}-f)"/>`);
      parts.push(storeBlock(DST_X, y + 16, store(r.callee), `${r.callee} 的表`));
    });

    const canvasW = DST_X + 190;
    const canvasH = 70 + rows.length * ROW_H + 30;

    notes.push(`每条链最多展示源/目标各 3 张表 (真实 DDL 统计列数); 被省略的表不计入摘要。`);
    notes.push('同名 caller→callee 边已去重 (多调用点合并为一条链), 证据 fileLine 取该对的第一条真实调用点。');

    return {
      svg: svgRoot(canvasW, canvasH, '数据流图', parts.join('')),
      width: canvasW,
      height: canvasH,
      title: '数据流图',
      summary: `${rows.length} 条数据变换链`,
      notes,
    };
  },
};
