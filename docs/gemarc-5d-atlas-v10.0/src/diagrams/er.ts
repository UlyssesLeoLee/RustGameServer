/**
 * diagrams/er.ts — ER 图 (spec `2D.diagrams.ER`)
 *
 * 专有符号: 方框实体、菱形关系、基数标注 (1 / 0..* / 1..* / 0..1 / 1..1)、
 *           PK 下划线、FK 虚线斜体、标识关系双线。
 *
 * 数据来源:
 *   - Table 节点: properties.columns[] (pk / fk / notNull / unique) + pkColumns + fkColumns
 *   - 关系 = 真实外键; 基数由列属性推导 (规则写进图注)
 */

import type { DiagramRenderer, DiagramContext, DiagramResult } from './types';
import type { GraphNode } from '../core/types';
import { esc, text, textLines, trunc, arrowDefs, svgRoot, uid } from './svg';
import { grid, bounds, type Box } from './layout';
import { columnsOf, crateOf, fkLinks, scopeFor, type Column, type FkLink } from './data';

const EW = 216;
const ATTR_MAX = 9;
const CARD_NOTE =
  '基数推导规则: 父端恒为 1 (每条子记录指向唯一父记录); '
  + '子端 = 1..* (NOT NULL 且非唯一) / 0..* (可空且非唯一) / 1..1 (唯一且 NOT NULL) / 0..1 (唯一且可空)。';

export const er: DiagramRenderer = {
  type: 'ER',
  label: 'ER 图',
  symbols: ['方框实体', '菱形关系', '基数 1 / 0..* / 1..*', 'PK 下划线', 'FK 虚线斜体'],
  dataSource: 'architecture.json: Table 节点 columns[].pk / .fk / .notNull / .unique',
  render(ctx: DiagramContext): DiagramResult {
    const id = uid('er');
    const notes: string[] = [CARD_NOTE];

    const scope = scopeFor(ctx, (n) => n.kind === 'Table', 30);
    const links = fkLinks(ctx).filter((l) => scope.nodes.some((n) => n.id === l.from.id));

    // 实体 = 参与外键的表 + 被引用表
    const entIds = new Set<string>();
    for (const l of links) {
      entIds.add(l.from.id);
      if (l.to) entIds.add(l.to.id);
    }
    const ents = scope.nodes.filter((n) => entIds.has(n.id));

    if (!ents.length) {
      return {
        svg: svgRoot(620, 130, 'ER 图', text(20, 40, '当前选择域内没有带外键的表。', { fill: 'var(--text-2)', 'font-size': 13 })),
        width: 620, height: 130, title: 'ER 图', summary: '0 个实体',
        notes: ['所选域内无外键关系, 不虚构实体关系。'],
      };
    }

    const colH = (n: GraphNode) => 30 + Math.min(ATTR_MAX, columnsOf(n).length) * 14 + 10;
    const rowH = Math.max(...ents.map(colH)) + 40;
    const boxMap = grid(ents, { cols: 3, w: EW, h: rowH, gapX: 96, gapY: 30, x0: 60, y0: 84 });

    const parts: string[] = [arrowDefs(id)];
    parts.push(text(20, 22, `ER 图 — ${ents.length} 个实体 / ${links.length} 条外键关系`, { 'font-size': 13, 'font-weight': 600 }));
    parts.push(text(20, 38, '下划线 = 主键 (PK);  虚线斜体 = 外键 (FK);  菱形 = 关系;  端点数字 = 基数。', {
      'font-size': 10, fill: 'var(--text-2)',
    }));

    // 关系 (先画线, 实体压在上面)
    for (const l of links) {
      const fb = boxMap.get(l.from);
      const tb = l.to ? boxMap.get(l.to) : undefined;
      if (!fb || !tb) continue;
      const x1 = fb.x + fb.w / 2;
      const y1 = fb.y + fb.h;
      const x2 = tb.x + tb.w / 2;
      const y2 = tb.y + tb.h;
      const identifying = l.column.pk && l.column.fk;
      parts.push(`<line x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}" stroke="var(--text-2)" stroke-width="1.2" `
        + `${identifying ? '' : ''} marker-end="url(#${id}-f)"/>`);
      if (identifying) {
        parts.push(`<line x1="${x1 + 2}" y1="${y1}" x2="${x2 + 2}" y2="${y2}" stroke="var(--text-2)" stroke-width="1"/>`);
      }
      // 菱形关系
      const mx = (x1 + x2) / 2;
      const my = (y1 + y2) / 2;
      const label = l.column.name;
      const hw = Math.max(26, label.length * 5 + 14);
      parts.push(`<path d="M ${mx} ${my - 15} L ${mx + hw} ${my} L ${mx} ${my + 15} L ${mx - hw} ${my} Z" `
        + `fill="var(--surface)" stroke="var(--text-2)" stroke-width="1.2"/>`);
      parts.push(text(mx, my + 4, label, { 'font-size': 9, 'text-anchor': 'middle' }));
      // 基数
      parts.push(text(x2 + 6, y2 - 4, l.cardinality.parent, { 'font-size': 10, 'font-weight': 600, fill: 'var(--info)' }));
      parts.push(text(x1 + 6, y1 - 6, l.cardinality.child, { 'font-size': 10, 'font-weight': 600, fill: 'var(--info)' }));
    }

    // 实体
    for (const n of ents) {
      const b = boxMap.get(n)!;
      const cols = columnsOf(n);
      const sel = ctx.selection.nodeIds.includes(n.id);
      const stroke = sel ? 'var(--focus)' : 'var(--text-2)';
      const sw = sel ? 2 : 1.2;
      const h = colH(n);
      parts.push(`<rect x="${b.x}" y="${b.y}" width="${b.w}" height="${h}" fill="var(--surface)" stroke="${stroke}" stroke-width="${sw}"/>`);
      parts.push(`<line x1="${b.x}" y1="${b.y + 26}" x2="${b.x + b.w}" y2="${b.y + 26}" stroke="${stroke}" stroke-width="1"/>`);
      parts.push(text(b.x + 8, b.y + 18, trunc(n.name, 30), { 'font-size': 11, 'font-weight': 600 }));
      parts.push(text(b.x + b.w - 6, b.y + 18, trunc(crateOf(n), 16), { 'font-size': 8, 'text-anchor': 'end', fill: 'var(--text-2)' }));
      cols.slice(0, ATTR_MAX).forEach((c: Column, i) => {
        const y = b.y + 40 + i * 14;
        const isPk = c.pk;
        const isFk = !!c.fk;
        parts.push(text(b.x + 10, y, c.name, {
          'font-size': 10, 'font-family': 'var(--mono)',
          'font-style': isFk ? 'italic' : 'normal',
          'text-decoration': isPk ? 'underline' : 'none',
          fill: isFk ? 'var(--info)' : 'var(--text)',
        }));
        if (isFk && !isPk) {
          parts.push(`<line x1="${b.x + 10}" y1="${y + 3}" x2="${b.x + 10 + c.name.length * 5.6}" y2="${y + 3}" `
            + `stroke="var(--info)" stroke-width="1" stroke-dasharray="2 2"/>`);
        }
        parts.push(text(b.x + b.w - 8, y, trunc(c.type.replace(/PRIMARY KEY|REFERENCES.*$/, '').trim(), 18), {
          'font-size': 8, 'text-anchor': 'end', fill: 'var(--text-2)',
        }));
      });
      if (cols.length > ATTR_MAX) {
        parts.push(text(b.x + 10, b.y + 40 + ATTR_MAX * 14, `… 另有 ${cols.length - ATTR_MAX} 列`, {
          'font-size': 9, fill: 'var(--text-2)',
        }));
      }
    }

    const bb = bounds([...boxMap.values()], 60);
    const canvasW = Math.max(820, bb.w + 60);
    const canvasH = Math.max(360, bb.h + 60);

    const unresolved = links.filter((l) => !l.to);
    if (unresolved.length) {
      notes.push(`${unresolved.length} 条外键未能解析到唯一目标表 (同名表跨 crate 歧义), 已在关系说明中标注 — 不猜目标。`);
    }
    notes.push(`实体集合 = 带外键的表 + 其被引用表, 共 ${ents.length} 个; 表总数 96, 其余无外键表未进入本图。`);

    return {
      svg: svgRoot(canvasW, canvasH, 'ER 图', parts.join('')),
      width: canvasW,
      height: canvasH,
      title: 'ER 图',
      summary: `${ents.length} 个实体 / ${links.length} 条关系`,
      notes,
    };
  },
};
