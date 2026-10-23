/**
 * diagrams/uml-component.ts — UML 组件图 (spec `2D.diagrams.UML_Component`)
 *
 * 专有符号: 组件 (左侧双 tab 矩形)、棒棒糖接口 (lollipop)、插槽 (socket)、
 *           组装连接器 (assembly connector, 虚线)。
 *
 * 数据来源:
 *   - Service 节点 (crate) → 组件
 *   - Api 节点 (proto service) → 提供的接口 (棒棒糖)
 *   - 真实 gRPC 调用边 (入边) → 需要的接口 (插槽 + 组装连接器)
 */

import type { DiagramRenderer, DiagramContext, DiagramResult } from './types';
import type { GraphNode } from '../core/types';
import { esc, text, textLines, trunc, arrowDefs, svgRoot, domainVar, uid } from './svg';
import { columnsBy, bounds, type Box } from './layout';
import {
  apisByCrate, crateOf, allEdges, isDesigned, inbound, scopeFor, serviceNodes,
} from './data';

const W = 190;
const H = 92;
const DOMAIN_ORDER = ['Core', 'Interaction', 'Data', 'Security', 'Infrastructure', 'External', 'Operations', 'Unknown'];

interface Comp {
  node: GraphNode;
  crate: string;
  apis: GraphNode[];
  sockets: { label: string; edgeId: string; y: number }[];
}

export const umlComponent: DiagramRenderer = {
  type: 'UML_Component',
  label: 'UML 组件图',
  symbols: ['组件双 tab 矩形', '棒棒糖 lollipop', '插槽 socket', '组装连接器'],
  dataSource: 'architecture.json: Service 节点 (crate) + Api 节点 (提供接口) + grpc-call 入边 (需要接口)',
  render(ctx: DiagramContext): DiagramResult {
    const id = uid('cmp');
    const scope = scopeFor(ctx, (n) => n.kind === 'Service', 24);
    const notes: string[] = [];

    if (!scope.nodes.length) {
      return {
        svg: svgRoot(600, 120, 'UML 组件图', text(20, 40, '当前选择下没有可绘制的组件。', { fill: 'var(--text-2)', 'font-size': 13 })),
        width: 600, height: 120, title: 'UML 组件图', summary: '0 个组件',
        notes: ['当前选择域内没有 Service 节点。'],
      };
    }

    const apisByC = apisByCrate(ctx);
    const comps: Comp[] = scope.nodes.map((n) => {
      const crate = crateOf(n);
      const ins = inbound(ctx, n.id).filter((e) => e.sourceId !== n.id);
      return {
        node: n,
        crate,
        apis: apisByC.get(crate) ?? [],
        sockets: ins.map((e, i) => ({
          label: String(e.properties.calleeService ?? e.id),
          edgeId: e.id,
          y: i,
        })),
      };
    });

    const boxMap = columnsBy(comps, (c) => c.node.domainId, {
      w: W, h: H, gapX: 96, gapY: 40, x0: 40, y0: 60, order: DOMAIN_ORDER,
    });

    const parts: string[] = [arrowDefs(id)];
    parts.push(text(20, 22, `UML 组件图 — ${comps.length} 个 crate 组件 (装配关系来自真实 gRPC 调用点)`, {
      'font-size': 13, 'font-weight': 600,
    }));
    parts.push(text(20, 38, '棒棒糖 = 提供的 proto service 接口;  半圆插槽 = 被真实调用方需要的接口;  虚线 = 组装连接器。', {
      'font-size': 10, fill: 'var(--text-2)',
    }));

    // 组装连接器: 调用方组件 → 被调方插槽
    const compByCrate = new Map(comps.map((c) => [c.crate, c]));
    for (const e of allEdges(ctx)) {
      const caller = compByCrate.get(String(e.properties.callerCrate ?? ''));
      const callee = compByCrate.get(String(e.properties.calleeCrate ?? ''));
      if (!caller || !callee) continue;
      const cb = boxMap.get(caller)!;
      const kb = boxMap.get(callee)!;
      const designed = isDesigned(ctx, e);
      const x1 = cb.x + cb.w;
      const y1 = cb.y + H / 2;
      const x2 = kb.x;
      const y2 = kb.y + 24 + Math.min(3, (callee.sockets.findIndex((s) => s.edgeId === e.id) + 1)) * 18;
      parts.push(`<path d="M ${x1} ${y1} C ${x1 + 50} ${y1}, ${x2 - 50} ${y2}, ${x2 - 8} ${y2}" `
        + `fill="none" stroke="${designed ? 'var(--warn)' : 'var(--text-2)'}" stroke-width="1.2" `
        + `stroke-dasharray="6 4" ${designed ? '' : 'marker-end="url(#' + id + '-o)"'}/>`);
      parts.push(text((x1 + x2) / 2, (y1 + y2) / 2 - 4, trunc(String(e.properties.calleeService ?? ''), 24), {
        'font-size': 9, fill: 'var(--text-2)', 'text-anchor': 'middle',
      }));
    }

    // 组件本体 + 棒棒糖 + 插槽
    for (const c of comps) {
      const b = boxMap.get(c)!;
      const stroke = ctx.selection.nodeIds.includes(c.node.id) ? 'var(--focus)' : 'var(--text-2)';
      const sw = ctx.selection.nodeIds.includes(c.node.id) ? 2 : 1.2;
      // 双 tab 组件符号
      parts.push(`<path d="M ${b.x} ${b.y} H ${b.x + 18} V ${b.y - 6} H ${b.x + 40} V ${b.y} `
        + `M ${b.x} ${b.y} H ${b.x + 18} V ${b.y + 14} H ${b.x + 40} V ${b.y}" fill="none" stroke="${stroke}" stroke-width="${sw}"/>`);
      parts.push(`<rect x="${b.x}" y="${b.y}" width="${b.w}" height="${H}" fill="var(--surface)" stroke="${stroke}" stroke-width="${sw}" rx="2"/>`);
      parts.push(text(b.x + 10, b.y + 20, `«crate»`, { 'font-size': 9, fill: 'var(--text-2)' }));
      parts.push(text(b.x + 10, b.y + 35, trunc(c.crate, 26), { 'font-size': 12, 'font-weight': 600 }));
      parts.push(text(b.x + 10, b.y + 50, `域 ${c.node.domainId}`, { 'font-size': 9, fill: 'var(--text-2)' }));
      parts.push(text(b.x + 10, b.y + 63, `v${String(c.node.properties.version ?? '—')}`, { 'font-size': 9, fill: 'var(--text-2)' }));
      parts.push(text(b.x + 10, b.y + 76, `deployed=${c.node.properties.deployed === true}`, { 'font-size': 9, fill: 'var(--text-2)' }));

      // 棒棒糖 (提供的接口) — 右侧
      c.apis.slice(0, 4).forEach((api, i) => {
        const ly = b.y + 16 + i * 18;
        parts.push(`<line x1="${b.x + b.w}" y1="${ly}" x2="${b.x + b.w + 26}" y2="${ly}" stroke="${stroke}" stroke-width="1.2"/>`);
        parts.push(`<circle cx="${b.x + b.w + 32}" cy="${ly}" r="5" fill="var(--surface)" stroke="${stroke}" stroke-width="1.4"/>`);
        parts.push(text(b.x + b.w + 41, ly + 3, trunc(api.name, 20), { 'font-size': 9, fill: 'var(--text-2)' }));
      });
      if (c.apis.length > 4) {
        parts.push(text(b.x + b.w + 41, b.y + 16 + 4 * 18 + 3, `+${c.apis.length - 4} 个接口`, { 'font-size': 9, fill: 'var(--text-2)' }));
      }

      // 插槽 (需要的接口) — 左侧
      c.sockets.slice(0, 4).forEach((s, i) => {
        const ly = b.y + 16 + i * 18;
        parts.push(`<path d="M ${b.x} ${ly} A 7 7 0 0 1 ${b.x} ${ly + 14}" fill="var(--bg)" stroke="${stroke}" stroke-width="1.3"/>`);
        parts.push(text(b.x - 10, ly + 10, trunc(s.label, 16), { 'font-size': 9, fill: 'var(--text-2)', 'text-anchor': 'end' }));
      });
    }

    const all = [...boxMap.values()];
    const bb = bounds(all, 120);
    const canvasW = Math.max(760, bb.w);
    const canvasH = Math.max(320, bb.h);

    return {
      svg: svgRoot(canvasW, canvasH, 'UML 组件图', parts.join('')),
      width: canvasW,
      height: canvasH,
      title: 'UML 组件图',
      summary: `${comps.length} 个组件 / ${comps.reduce((n, c) => n + c.apis.length, 0)} 个提供接口 / ${comps.reduce((n, c) => n + c.sockets.length, 0)} 个需要接口`,
      notes: [
        '组件 = Rust crate (Service 节点); 棒棒糖 = crate 提供的 proto service; 插槽 = 真实调用方依赖的接口。',
        '装配连接器来自 architecture.json 的真实 grpc-call 边; 黄色虚线 = Inferred (仅注释提及)。',
        ...(scope.truncated ? [scope.note] : []),
      ],
    };
  },
};
