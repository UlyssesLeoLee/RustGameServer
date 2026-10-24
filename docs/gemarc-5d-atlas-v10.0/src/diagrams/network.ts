/**
 * diagrams/network.ts — 网络拓扑 (spec `2D.diagrams.Network`)
 *
 * 专有符号: 设备 (服务器符号: 矩形 + 横向分隔线)、集群节点 (交叉线)、
 *           链路 (实线/虚线)、链路标签。
 *
 * 数据来源:
 *   - 设备 = 有真实 k8s 部署证据的 Service (properties.deployment)
 *   - 集群 = containers 的 Building (k3s / rust-game-server)
 *   - 链路 = 真实 grpc-call 边
 */

import type { DiagramRenderer, DiagramContext, DiagramResult } from './types';
import { esc, text, textLines, trunc, arrowDefs, svgRoot, uid } from './svg';
import { grid, bounds, type Box } from './layout';
import { allEdges, isDesigned, deploymentOf, deployedServices, crateOf } from './data';

const DW = 116;
const DH = 62;

function server(x: number, y: number, label: string, sub: string, sel: boolean): string {
  const stroke = sel ? 'var(--focus)' : 'var(--text-2)';
  const sw = sel ? 2 : 1.2;
  const p: string[] = [];
  p.push(`<rect x="${x}" y="${y}" width="${DW}" height="${DH}" rx="3" fill="var(--surface)" stroke="${stroke}" stroke-width="${sw}"/>`);
  p.push(`<line x1="${x}" y1="${y + 15}" x2="${x + DW}" y2="${y + 15}" stroke="${stroke}" stroke-width="1"/>`);
  p.push(`<line x1="${x}" y1="${y + DH - 15}" x2="${x + DW}" y2="${y + DH - 15}" stroke="${stroke}" stroke-width="1"/>`);
  p.push(`<circle cx="${x + DW - 10}" cy="${y + 7.5}" r="2.5" fill="${stroke}"/>`);
  p.push(`<circle cx="${x + DW - 19}" cy="${y + 7.5}" r="2.5" fill="${stroke}"/>`);
  p.push(text(x + 8, y + 30, trunc(label, 18), { 'font-size': 10, 'font-weight': 600 }));
  p.push(text(x + 8, y + 43, trunc(sub, 18), { 'font-size': 8, fill: 'var(--text-2)' }));
  p.push(text(x + 8, y + 55, trunc(label, 18), { 'font-size': 7, fill: 'var(--text-2)' }));
  return p.join('');
}

export const network: DiagramRenderer = {
  type: 'Network',
  label: '网络拓扑',
  symbols: ['服务器设备符号', '集群节点', '链路', '链路标签'],
  dataSource: 'architecture.json: Service.properties.deployment (k8s 证据) + containers Building + grpc-call 边',
  render(ctx: DiagramContext): DiagramResult {
    const id = uid('net');
    const notes: string[] = [];
    const devices = deployedServices(ctx);
    const building = [...ctx.index.containers.values()].find((c) => c.kind === 'Building');

    if (!devices.length) {
      return {
        svg: svgRoot(620, 130, '网络拓扑', text(20, 40, '模型中没有带部署证据的设备。', { fill: 'var(--text-2)', 'font-size': 13 })),
        width: 620, height: 130, title: '网络拓扑', summary: '0 台设备',
        notes: ['Service.properties.deployed 全为 false — 不虚构网络设备。'],
      };
    }

    const list = ctx.selection.focusId && devices.some((n) => n.id === ctx.selection.focusId)
      ? devices.filter((n) => n.id === ctx.selection.focusId || devices.some((m) => m.id === ctx.selection.focusId))
      : devices;

    const boxMap = grid(list, { cols: 4, w: DW + 34, h: DH + 44, gapX: 26, gapY: 24, x0: 60, y0: 116 });
    const boxes = [...boxMap.values()];
    const bb = bounds(boxes, 60);
    const canvasW = Math.max(860, bb.w + 60);
    const canvasH = Math.max(420, bb.h + 130);

    const parts: string[] = [arrowDefs(id)];
    parts.push(text(20, 22, `网络拓扑 — ${list.length} 台设备 (全部有真实 k8s 部署证据)`, { 'font-size': 13, 'font-weight': 600 }));
    parts.push(text(20, 38, `集群: ${building?.name ?? '—'} · 命名空间: ${[...new Set(devices.map((n) => deploymentOf(n)!.namespace))].join(', ')}`, {
      'font-size': 10, fill: 'var(--text-2)',
    }));

    // 集群边界
    parts.push(`<rect x="34" y="74" width="${canvasW - 60}" height="${canvasH - 110}" rx="10" fill="none" `
      + `stroke="var(--info)" stroke-width="1.3" stroke-dasharray="9 6"/>`);
    parts.push(text(44, 66, `«cluster» ${building?.name ?? 'k3s'}`, { 'font-size': 10, fill: 'var(--info)' }));

    // 链路
    const byCrate = new Map(list.map((n) => [crateOf(n), n]));
    for (const e of allEdges(ctx)) {
      const caller = byCrate.get(String(e.properties.callerCrate ?? ''));
      const callee = byCrate.get(String(e.properties.calleeCrate ?? ''));
      if (!caller || !callee) continue;
      const a = boxMap.get(caller)!;
      const b = boxMap.get(callee)!;
      const x1 = a.x + DW / 2;
      const y1 = a.y + DH / 2;
      const x2 = b.x + DW / 2;
      const y2 = b.y + DH / 2;
      const designed = isDesigned(ctx, e);
      parts.push(`<line x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}" stroke="${designed ? 'var(--warn)' : 'var(--info)'}" `
        + `stroke-width="1.4" ${designed ? 'stroke-dasharray="7 4"' : ''} marker-end="url(#${id}-f)"/>`);
      parts.push(text((x1 + x2) / 2, (y1 + y2) / 2 - 5, trunc(String(e.properties.calleeService ?? ''), 22), {
        'font-size': 9, 'text-anchor': 'middle',
      }));
      parts.push(text((x1 + x2) / 2, (y1 + y2) / 2 + 7, trunc(String(e.properties.fileLine ?? ''), 28), {
        'font-size': 7, 'text-anchor': 'middle', fill: 'var(--text-2)',
      }));
    }

    // 设备
    for (const n of list) {
      const b = boxMap.get(n)!;
      const dep = deploymentOf(n)!;
      const sel = ctx.selection.nodeIds.includes(n.id);
      parts.push(server(b.x, b.y, n.name, `replicas=${dep.replicas}`, sel));
      parts.push(text(b.x, b.y + DH + 13, trunc(dep.manifest, 34), { 'font-size': 7, fill: 'var(--text-2)' }));
    }

    notes.push(`仅绘制有 properties.deployment 证据的服务 (共 ${devices.length} 个 / 全模型 31 个 crate); 其余 crate 无部署证据, 不臆造设备。`);
    notes.push('链路 = 真实 grpc-call 边; 黄色虚线 = Inferred (仅注释提及, 实际未接入网络)。');

    return {
      svg: svgRoot(canvasW, canvasH, '网络拓扑', parts.join('')),
      width: canvasW,
      height: canvasH,
      title: '网络拓扑',
      summary: `${list.length} 台设备 / ${allEdges(ctx).filter((e) => {
        const c = byCrate.get(String(e.properties.callerCrate ?? ''));
        const k = byCrate.get(String(e.properties.calleeCrate ?? ''));
        return c && k;
      }).length} 条链路`,
      notes,
    };
  },
};
