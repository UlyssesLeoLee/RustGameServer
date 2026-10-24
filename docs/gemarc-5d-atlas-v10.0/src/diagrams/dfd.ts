/**
 * diagrams/dfd.ts — 数据流图 DFD (spec `2D.diagrams.DFD`)
 *
 * 专有符号: 开口矩形 (外部实体)、圆 (加工/加工)、开口平行线 (数据存储)、
 *           带标签的数据流。
 *
 * 数据来源:
 *   - 加工 = 被真实调用的 crate (Service 节点)
 *   - 外部实体 = 只作为调用方出现、从未被调用的 crate
 *   - 数据存储 = 真实 Table 节点
 *   - 数据流 = 真实 grpc-call 边 (标签 = calleeService + fileLine)
 */

import type { DiagramRenderer, DiagramContext, DiagramResult } from './types';
import { esc, text, textLines, trunc, arrowDefs, svgRoot, uid } from './svg';
import { allEdges, isDesigned, edgeFileLine, serviceNodes, tableNodes, crateOf, domainFacets } from './data';

const PROC_R = 40;
const STORE_W = 128;

export const dfd: DiagramRenderer = {
  type: 'DFD',
  label: '数据流图 DFD',
  symbols: ['开口矩形外部实体', '圆加工', '开口平行线数据存储', '带标签数据流'],
  dataSource: 'architecture.json: Service / Table 节点 + grpc-call 边 (fileLine)',
  render(ctx: DiagramContext): DiagramResult {
    const id = uid('dfd');
    const notes: string[] = [];
    const edges = allEdges(ctx);

    if (!edges.length) {
      return {
        svg: svgRoot(620, 130, '数据流图', text(20, 40, '模型中没有调用边, 无法生成数据流图。', { fill: 'var(--text-2)', 'font-size': 13 })),
        width: 620, height: 130, title: '数据流图', summary: '0 条数据流',
        notes: ['architecture.json 中没有 grpc-call 边。'],
      };
    }

    // 只被调用 / 只调用
    const called = new Set<string>();
    const calledBy = new Set<string>();
    for (const e of edges) {
      if (e.properties.calleeCrate) called.add(String(e.properties.calleeCrate));
      if (e.properties.callerCrate) calledBy.add(String(e.properties.callerCrate));
    }
    const externals = [...calledBy].filter((c) => !called.has(c)).sort();
    const procs = [...called].sort();
    const tables = tableNodes(ctx);
    const tablesByCrate = new Map<string, typeof tables>();
    for (const t of tables) {
      const c = crateOf(t);
      if (!tablesByCrate.has(c)) tablesByCrate.set(c, []);
      tablesByCrate.get(c)!.push(t);
    }

    const lanes = domainFacets(ctx).filter((f) => procs.some((p) => f.services.some((s) => crateOf(s) === p)));
    const laneOrder = lanes.length ? lanes.map((l) => l.name) : [...new Set(procs)];

    // 布局: 每条泳道一列加工
    const laneX = new Map<string, number>();
    laneOrder.forEach((l, i) => laneX.set(l, 250 + i * 210));
    const procPos = new Map<string, { x: number; y: number }>();
    let py = 90;
    for (const p of procs) {
      const svc = serviceNodes(ctx).find((s) => crateOf(s) === p);
      const lane = svc?.domainId ?? 'Unknown';
      const x = laneX.get(lane) ?? 250;
      const cur = procPos.get(p);
      if (!cur) procPos.set(p, { x, y: py });
      else procPos.set(p, { x: cur.x, y: py });
      py += 108;
    }

    const extPos = new Map<string, { x: number; y: number }>();
    externals.forEach((e, i) => extPos.set(e, { x: 70, y: 110 + i * 92 }));

    const storePos = new Map<string, { x: number; y: number }>();
    let sy = 96;
    for (const [c, list] of tablesByCrate) {
      if (!procs.includes(c) && !externals.includes(c)) continue;
      for (const t of list) {
        if (storePos.size > 14) break;
        storePos.set(t.id, { x: 250 + laneOrder.length * 210 + 30, y: sy });
        sy += 46;
      }
    }

    const parts: string[] = [arrowDefs(id)];
    parts.push(text(20, 22, `数据流图 DFD — ${procs.length} 个加工 / ${externals.length} 个外部实体 / ${storePos.size} 个数据存储`, {
      'font-size': 13, 'font-weight': 600,
    }));
    parts.push(text(20, 38, '外部实体 = 只调用不被调用的 crate; 加工 = 被调用的 crate; 数据存储 = 真实表; 数据流标签 = 真实 calleeService。', {
      'font-size': 10, fill: 'var(--text-2)',
    }));

    // 泳道标题
    laneOrder.forEach((l) => {
      const x = laneX.get(l)!;
      parts.push(`<line x1="${x - 96}" y1="56" x2="${x - 96}" y2="${Math.max(300, py + 20)}" stroke="var(--edge)" stroke-width="1" stroke-dasharray="4 4"/>`);
      parts.push(text(x - 100, 66, `«domain» ${l}`, { 'font-size': 10, fill: 'var(--text-2)' }));
    });

    // 数据流 (真实边)
    for (const e of edges) {
      const caller = String(e.properties.callerCrate ?? '');
      const callee = String(e.properties.calleeCrate ?? '');
      const cp = extPos.get(caller) ?? procPos.get(caller);
      const kp = procPos.get(callee);
      if (!cp || !kp) continue;
      const designed = isDesigned(ctx, e);
      parts.push(`<line x1="${cp.x}" y1="${cp.y}" x2="${kp.x}" y2="${kp.y}" stroke="${designed ? 'var(--warn)' : 'var(--text-2)'}" `
        + `stroke-width="1.3" ${designed ? 'stroke-dasharray="6 4"' : ''} marker-end="url(#${id}-f)"/>`);
      parts.push(text((cp.x + kp.x) / 2, (cp.y + kp.y) / 2 - 6, trunc(String(e.properties.calleeService ?? ''), 24), {
        'font-size': 9, 'text-anchor': 'middle', fill: 'var(--text)',
      }));
      parts.push(text((cp.x + kp.x) / 2, (cp.y + kp.y) / 2 + 6, trunc(edgeFileLine(e), 28), {
        'font-size': 7, 'text-anchor': 'middle', fill: 'var(--text-2)',
      }));
    }

    // 外部实体 (开口矩形)
    for (const [name, p] of extPos) {
      const w = 130;
      const h = 52;
      parts.push(`<path d="M ${p.x - w / 2} ${p.y - h / 2} H ${p.x + w / 2} V ${p.y + h / 2} H ${p.x - w / 2}" `
        + `fill="none" stroke="var(--text-2)" stroke-width="1.3"/>`);
      parts.push(text(p.x, p.y - h / 2 - 6, trunc(name, 22), { 'font-size': 10, 'text-anchor': 'middle', 'font-weight': 600 }));
    }

    // 加工 (圆)
    for (const [c, p] of procPos) {
      parts.push(`<circle cx="${p.x}" cy="${p.y}" r="${PROC_R}" fill="var(--surface)" stroke="var(--info)" stroke-width="1.3"/>`);
      parts.push(text(p.x, p.y - 3, trunc(c, 16), { 'font-size': 9, 'text-anchor': 'middle' }));
      parts.push(text(p.x, p.y + 10, 'process', { 'font-size': 7, 'text-anchor': 'middle', fill: 'var(--text-2)' }));
    }

    // 数据存储 (开口平行线)
    for (const [id2, p] of storePos) {
      const t = ctx.index.nodes.get(id2)!;
      const w = STORE_W;
      parts.push(`<line x1="${p.x}" y1="${p.y}" x2="${p.x + w}" y2="${p.y}" stroke="var(--text-2)" stroke-width="1.4"/>`);
      parts.push(`<line x1="${p.x}" y1="${p.y + 16}" x2="${p.x + w}" y2="${p.y + 16}" stroke="var(--text-2)" stroke-width="1.4"/>`);
      parts.push(text(p.x + 3, p.y + 12, trunc(t.name, 20), { 'font-size': 9 }));
      parts.push(text(p.x + 3, p.y + 28, `crate ${trunc(crateOf(t), 16)}`, { 'font-size': 7, fill: 'var(--text-2)' }));
    }

    const canvasW = Math.max(900, 250 + laneOrder.length * 210 + 30 + STORE_W + 40);
    const canvasH = Math.max(360, Math.max(py + 40, sy + 40));

    notes.push(`数据存储最多绘制 14 个 (共 ${tables.length} 张真实表); 被裁掉的不计入摘要, 避免夸大覆盖面。`);
    notes.push('外部实体判定 = 在真实调用图中"只作为调用方出现"的 crate; 工具 crate (rgs-testkit / gm-backend) 因此落在外部实体侧。');

    return {
      svg: svgRoot(canvasW, canvasH, '数据流图', parts.join('')),
      width: canvasW,
      height: canvasH,
      title: '数据流图',
      summary: `${procs.length} 个加工 / ${externals.length} 个外部实体 / ${storePos.size} 个数据存储 / ${edges.length} 条数据流`,
      notes,
    };
  },
};
