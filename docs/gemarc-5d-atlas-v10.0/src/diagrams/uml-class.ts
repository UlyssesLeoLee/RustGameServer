/**
 * diagrams/uml-class.ts — UML 类图 (spec `2D.diagrams.UML_Class`)
 *
 * 专有符号: 三 compartment (类名 / 属性 / 方法)、可见性符号 (+ - # ~)、
 *           抽象/接口类名斜体、构造型 «...»、实现/关联虚线。
 *
 * 数据来源:
 *   - Api 节点 (gRPC service 定义) → «interface» 类, 类名斜体 (UML 惯例)
 *   - Table 节点 (真实 DDL) → «entity» 类, 属性 = 真实列
 *   - ports (按 crate 归属) → 操��
 *   - edges (真实 gRPC 调用) → 实现关系
 */

import type { DiagramRenderer, DiagramContext, DiagramResult } from './types';
import type { GraphNode } from '../core/types';
import { esc, text, textLines, wrapText, trunc, arrowDefs, svgRoot, domainVar, uid } from './svg';
import { grid, bounds, centerOffset, type Box } from './layout';
import {
  apiNodes, columnsOf, crateOf, fkTarget, scopeFor,
  apisByCrate, portsByCrate, type Column,
} from './data';

const W = 240;
const ATTR_MAX = 6;
const OP_MAX = 4;

/** 可见性映射规则 (写入图注, 不隐藏推导) */
const VIS_RULE =
  '属性可见性映射 (模型无显式 visibility 字段, 按列属性推导): '
  + '+ = 主键列 (pk) / # = 外键列 (fk) / - = NOT NULL 且非主键非外键 / ~ = 可空且非主键非外键。';

interface ClassDef {
  node: GraphNode;
  stereotype: string;
  name: string;
  italic: boolean;
  attrs: string[];
  ops: string[];
  attrOverflow: number;
  opOverflow: number;
}

function vis(col: Column): string {
  if (col.pk) return '+';
  if (col.fk) return '#';
  if (col.notNull) return '-';
  return '~';
}

function buildClasses(ctx: DiagramContext): ClassDef[] {
  const scope = scopeFor(ctx, (n) => n.kind === 'Api' || n.kind === 'Table', 28);
  const apisByC = apisByCrate(ctx);
  const portsByC = portsByCrate(ctx);
  const out: ClassDef[] = [];

  for (const n of scope.nodes) {
    if (n.kind === 'Api') {
      const crate = crateOf(n);
      const apiCount = (apisByC.get(crate) ?? []).length;
      const ports = portsByC.get(crate) ?? [];
      const reqs: string[] = [];
      const resps: string[] = [];
      for (const p of ports) {
        if (p.requestType && !reqs.includes(p.requestType)) reqs.push(p.requestType);
        if (p.responseType && !resps.includes(p.responseType)) resps.push(p.responseType);
      }
      const attrs: string[] = [];
      for (const r of reqs.slice(0, 3)) attrs.push(`req: ${trunc(r, 30)}`);
      for (const r of resps.slice(0, 2)) attrs.push(`res: ${trunc(r, 30)}`);

      const ops: string[] = [];
      let opOverflow = 0;
      if (apiCount === 1) {
        for (const p of ports.slice(0, OP_MAX)) {
          ops.push(`+ ${p.name}(${trunc(p.requestType, 18)}) : ${trunc(p.responseType, 18)}`);
        }
        opOverflow = Math.max(0, ports.length - OP_MAX);
      } else {
        attrs.push(`[端口归属不可判定: 该 crate 有 ${apiCount} 个 proto service, 端口 apiId 全为 null]`);
      }
      out.push({
        node: n,
        stereotype: 'interface',
        name: n.name,
        italic: true,
        attrs: attrs.slice(0, ATTR_MAX),
        ops,
        attrOverflow: Math.max(0, attrs.length - ATTR_MAX),
        opOverflow,
      });
      continue;
    }

    // Table → «entity»
    const cols = columnsOf(n);
    const attrs: string[] = [];
    let attrOverflow = 0;
    for (const c of cols.slice(0, ATTR_MAX)) {
      attrs.push(`${vis(c)} ${c.name} : ${trunc(c.type.replace(/PRIMARY KEY|REFERENCES.*$/, '').trim(), 24)}`);
    }
    attrOverflow = Math.max(0, cols.length - ATTR_MAX);

    const ops: string[] = [];
    let opOverflow = 0;
    for (const c of cols.filter((x) => x.fk)) {
      if (ops.length >= OP_MAX) { opOverflow += 1; continue; }
      const t = fkTarget(ctx, n, c.fk!);
      ops.push(`+ ${c.name}() : ${t ? t.name : c.fk!}`);
    }
    out.push({
      node: n,
      stereotype: 'entity',
      name: n.name,
      italic: false,
      attrs,
      ops,
      attrOverflow,
      opOverflow,
    });
  }
  return out;
}

function classHeight(c: ClassDef): number {
  return 34 + Math.max(1, c.attrs.length) * 13 + 8 + Math.max(1, c.ops.length) * 13 + 8;
}

function classSvg(b: Box, c: ClassDef, sel: boolean): string {
  const stroke = sel ? 'var(--focus)' : 'var(--edge)';
  const sw = sel ? 2 : 1;
  const h = classHeight(c);
  const parts: string[] = [];
  parts.push(`<rect x="${b.x}" y="${b.y}" width="${b.w}" height="${h}" rx="4" fill="var(--surface)" stroke="${stroke}" stroke-width="${sw}"/>`);
  // compartment 1 — 类名
  parts.push(text(b.x + 8, b.y + 14, `«${c.stereotype}»`, { 'font-size': 10, fill: 'var(--text-2)' }));
  parts.push(text(b.x + 8, b.y + 29, c.name, {
    'font-size': 12, 'font-weight': 600, fill: 'var(--text)',
    'font-style': c.italic ? 'italic' : 'normal',
  }));
  const h1 = 34;
  parts.push(`<line x1="${b.x}" y1="${b.y + h1}" x2="${b.x + b.w}" y2="${b.y + h1}" stroke="${stroke}" stroke-width="1"/>`);
  // compartment 2 — 属性
  const aTop = b.y + h1 + 13;
  parts.push(text(b.x + 8, b.y + h1 + 12, 'attributes', { 'font-size': 9, fill: 'var(--text-2)' }));
  c.attrs.forEach((line, i) => {
    parts.push(text(b.x + 8, aTop + 11 + i * 13, line, {
      'font-size': 10, 'font-family': 'var(--mono)',
      'font-style': c.attrs[i]?.includes('#') ? 'italic' : 'normal',
    }));
  });
  if (c.attrOverflow) {
    parts.push(text(b.x + 8, aTop + 11 + c.attrs.length * 13, `… 另有 ${c.attrOverflow} 列`, { 'font-size': 10, fill: 'var(--text-2)' }));
  }
  const h2 = h1 + Math.max(1, c.attrs.length) * 13 + 8 + (c.attrOverflow ? 13 : 0);
  parts.push(`<line x1="${b.x}" y1="${b.y + h2}" x2="${b.x + b.w}" y2="${b.y + h2}" stroke="${stroke}" stroke-width="1"/>`);
  // compartment 3 — 方法
  const oTop = b.y + h2 + 13;
  parts.push(text(b.x + 8, b.y + h2 + 12, 'operations', { 'font-size': 9, fill: 'var(--text-2)' }));
  if (!c.ops.length) {
    parts.push(text(b.x + 8, oTop + 11, '(无)', { 'font-size': 10, fill: 'var(--text-2)' }));
  }
  c.ops.forEach((line, i) => {
    parts.push(text(b.x + 8, oTop + 11 + i * 13, line, { 'font-size': 10, 'font-family': 'var(--mono)' }));
  });
  if (c.opOverflow) {
    parts.push(text(b.x + 8, oTop + 11 + c.ops.length * 13, `… 另有 ${c.opOverflow} 个`, { 'font-size': 10, fill: 'var(--text-2)' }));
  }
  return parts.join('');
}

export const umlClass: DiagramRenderer = {
  type: 'UML_Class',
  label: 'UML 类图',
  symbols: ['三 compartment', '可见性符号 + - # ~', '接口类名斜体', '«stereotype»'],
  dataSource: 'architecture.json: Api 节点 (proto service) + Table 节点 (DDL 列) + ports + grpc-call 边',
  render(ctx: DiagramContext): DiagramResult {
    const id = uid('cls');
    const classes = buildClasses(ctx);
    const notes: string[] = [VIS_RULE];

    if (!classes.length) {
      return {
        svg: svgRoot(600, 120, 'UML 类图', `<text x="20" y="40" fill="var(--text-2)" font-size="13">当前选择下没有可绘制的类。</text>`),
        width: 600, height: 120,
        title: 'UML 类图',
        summary: '0 个类',
        notes: ['当前选择域内没有 Api / Table 节点。'],
      };
    }

    const byKind = (k: 'Api' | 'Table') => classes.filter((c) => c.node.kind === k);
    const ifaces = byKind('Api');
    const ents = byKind('Table');

    const boxes = new Map<ClassDef, Box>();
    let y = 30;
    let maxCols = 0;

    const pack = (list: ClassDef[], y0: number) => {
      if (!list.length) return y0;
      const cols = Math.min(4, Math.max(1, list.length));
      maxCols = Math.max(maxCols, cols);
      list.forEach((c, i) => {
        const cc = i % cols;
        const rr = Math.floor(i / cols);
        const colH = Math.max(
          classHeight(c),
          ...list.filter((_, j) => j % cols === cc).map(classHeight),
        );
        boxes.set(c, { x: 20 + cc * (W + 40), y: y0 + rr * (colH + 46), w: W, h: classHeight(c) });
      });
      const rows = Math.ceil(list.length / cols);
      const colHs: number[] = [];
      for (let cc = 0; cc < cols; cc++) {
        const inCol = list.filter((_, j) => j % cols === cc);
        colHs.push(Math.max(0, ...inCol.map(classHeight)));
      }
      const maxH = Math.max(0, ...colHs);
      return y0 + rows * (maxH + 46);
    };

    y = pack(ifaces, y);
    if (ifaces.length && ents.length) y += 26;
    y = pack(ents, y);

    const all = [...boxes.values()];
    const canvasW = Math.max(760, 20 + maxCols * (W + 40) + 20);
    const canvasH = y + 20;

    const parts: string[] = [arrowDefs(id)];
    parts.push(text(20, 20, `UML 类图 — ${ifaces.length} 个接口 / ${ents.length} 个实体 (数据: models/architecture.json)`, {
      'font-size': 13, 'font-weight': 600,
    }));
    parts.push(text(20, 36, VIS_RULE, { 'font-size': 10, fill: 'var(--text-2)' }));

    // 关系: 真实 gRPC 调用 → 实现 (虚线三角箭头)
    const nodeById = new Map(classes.map((c) => [c.node.id, c]));
    for (const e of ctx.index.edges.values()) {
      const s = nodeById.get(e.sourceId ?? '');
      const t = nodeById.get(e.targetId ?? '');
      if (!s || !t) continue;
      const sb = boxes.get(s)!;
      const tb = boxes.get(t)!;
      const x1 = sb.x + sb.w;
      const y1 = sb.y + 20;
      const x2 = tb.x;
      const y2 = tb.y + 20;
      parts.push(`<path d="M ${x1} ${y1} C ${x1 + 40} ${y1}, ${x2 - 40} ${y2}, ${x2} ${y2}" `
        + `fill="none" stroke="var(--text-2)" stroke-width="1" stroke-dasharray="5 3" marker-end="url(#${id}-t)"/>`);
    }

    for (const [c, b] of boxes) parts.push(classSvg(b, c, ctx.selection.nodeIds.includes(c.node.id)));
    for (const c of classes) {
      const b = boxes.get(c)!;
      parts.push(text(b.x + b.w - 6, b.y + 29, c.node.verification, {
        'font-size': 9, fill: 'var(--text-2)', 'text-anchor': 'end',
      }));
    }

    return {
      svg: svgRoot(canvasW, canvasH, 'UML 类图', parts.join('')),
      width: canvasW,
      height: canvasH,
      title: 'UML 类图',
      summary: `${ifaces.length} 个 «interface» 类 + ${ents.length} 个 «entity» 类`,
      notes,
    };
  },
};
