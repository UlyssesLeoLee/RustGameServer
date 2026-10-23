/**
 * diagrams/uml-deployment.ts — UML 部署图 (spec `2D.diagrams.UML_Deployment`)
 *
 * 专有符号: 3D 立方节点 (UML deployment node)、制品 (artifact)、
 *           通信路径 (communication path)、节点内部构件。
 *
 * 数据来源:
 *   - Service.properties.deployment (kind/namespace/replicas/manifest) — 真实 k8s 证据
 *   - 制品 = 该 crate 真实拥有的 Table + protoFile
 *   - 通信路径 = 真实 grpc-call 边
 */

import type { DiagramRenderer, DiagramContext, DiagramResult } from './types';
import type { GraphNode } from '../core/types';
import { esc, text, textLines, trunc, arrowDefs, svgRoot, uid } from './svg';
import { grid, bounds, type Box } from './layout';
import { allEdges, isDesigned, deploymentOf, deployedServices, tablesByCrate, crateOf } from './data';

const CUBE_W = 128;
const CUBE_H = 66;
const ART_W = 138;
const ART_H = 17;
const DOMAIN_ORDER = ['Core', 'Interaction', 'Data', 'Security', 'Infrastructure', 'External', 'Operations', 'Unknown'];

function cube(x: number, y: number, label: string, sub: string, sel: boolean): string {
  const stroke = sel ? 'var(--focus)' : 'var(--text-2)';
  const sw = sel ? 2 : 1.2;
  const d = 10;
  const parts: string[] = [];
  // 顶面
  parts.push(`<path d="M ${x} ${y + d} L ${x + d} ${y} L ${x + CUBE_W + d} ${y} L ${x + CUBE_W} ${y + d} Z" fill="var(--raised)" stroke="${stroke}" stroke-width="${sw}"/>`);
  // 右侧面
  parts.push(`<path d="M ${x + CUBE_W} ${y + d} L ${x + CUBE_W + d} ${y} L ${x + CUBE_W + d} ${y + CUBE_H - d} L ${x + CUBE_W} ${y + CUBE_H} Z" fill="var(--raised)" stroke="${stroke}" stroke-width="${sw}"/>`);
  // 正面
  parts.push(`<rect x="${x}" y="${y + d}" width="${CUBE_W}" height="${CUBE_H - d}" fill="var(--surface)" stroke="${stroke}" stroke-width="${sw}"/>`);
  parts.push(text(x + 8, y + d + 18, trunc(label, 20), { 'font-size': 10, 'font-weight': 600 }));
  parts.push(text(x + 8, y + d + 31, trunc(sub, 22), { 'font-size': 8, fill: 'var(--text-2)' }));
  return parts.join('');
}

export const umlDeployment: DiagramRenderer = {
  type: 'UML_Deployment',
  label: 'UML 部署图',
  symbols: ['3D 立方节点', '制品 artifact', '通信路径', '命名空间边界'],
  dataSource: 'architecture.json: Service.properties.deployment (k8s 证据) + Table 节点 + grpc-call 边',
  render(ctx: DiagramContext): DiagramResult {
    const id = uid('dep');
    const notes: string[] = [];
    const deployed = deployedServices(ctx);
    const focus = ctx.selection.focusId;

    if (!deployed.length) {
      return {
        svg: svgRoot(600, 120, 'UML 部署图', text(20, 40, '模型中没有带 deployment 证据的服务。', { fill: 'var(--text-2)', 'font-size': 13 })),
        width: 600, height: 120, title: 'UML 部署图', summary: '0 个部署节点',
        notes: ['Service.properties.deployed 全为 false — 部署状态为 Unknown, 不虚构节点。'],
      };
    }

    const list = focus && deployed.some((n) => n.id === focus)
      ? deployed.filter((n) => n.id === focus || deployed.some((m) => m.id === focus))
      : deployed;

    const tablesByC = tablesByCrate(ctx);
    const ordered = [...list].sort((a, b) => {
      const da = DOMAIN_ORDER.indexOf(a.domainId);
      const db = DOMAIN_ORDER.indexOf(b.domainId);
      return (da < 0 ? 99 : da) - (db < 0 ? 99 : db) || a.name.localeCompare(b.name);
    });

    const cols = 3;
    const artifactsOf = (n: GraphNode) => {
      const out: string[] = [];
      for (const t of tablesByC.get(crateOf(n)) ?? []) out.push(`«table» ${t.name}`);
      const api = [...ctx.index.nodes.values()].find((x) => x.kind === 'Api' && crateOf(x) === crateOf(n));
      if (api) out.push(`«proto» ${String(api.properties.protoFile ?? '')}`);
      return out;
    };

    const artCounts = ordered.map((n) => artifactsOf(n).length);
    const maxArt = Math.max(1, ...artCounts);
    const rowH = CUBE_H + 24 + maxArt * (ART_H + 3) + 20;
    const boxMap = grid(ordered, {
      cols, w: Math.max(CUBE_W + 12, ART_W), h: rowH, gapX: 40, gapY: 26, x0: 40, y0: 80,
    });

    const parts: string[] = [arrowDefs(id)];
    parts.push(text(20, 22, `UML 部署图 — ${ordered.length} 个有 k8s 证据的部署节点`, { 'font-size': 13, 'font-weight': 600 }));
    const ns = deployed.map((n) => deploymentOf(n)!.namespace).filter(Boolean);
    const nsText = [...new Set(ns)].join(', ') || '—';
    parts.push(text(20, 38, `命名空间: ${nsText}  (来自 Service.properties.deployment 真实证据)`, { 'font-size': 10, fill: 'var(--text-2)' }));

    // 命名空间边界
    const boxes = [...boxMap.values()];
    const bb = bounds(boxes, 30);
    parts.push(`<rect x="22" y="58" width="${bb.w}" height="${bb.h - 20}" rx="6" fill="none" stroke="var(--edge)" stroke-width="1.2" stroke-dasharray="8 5"/>`);
    parts.push(text(28, 54, `«node» ${nsText}`, { 'font-size': 10, fill: 'var(--text-2)' }));

    // 通信路径
    const byCrate = new Map(ordered.map((n) => [crateOf(n), n]));
    for (const e of allEdges(ctx)) {
      const caller = byCrate.get(String(e.properties.callerCrate ?? ''));
      const callee = byCrate.get(String(e.properties.calleeCrate ?? ''));
      if (!caller || !callee) continue;
      const a = boxMap.get(caller)!;
      const b = boxMap.get(callee)!;
      const x1 = a.x + CUBE_W / 2;
      const y1 = a.y + CUBE_H / 2;
      const x2 = b.x + CUBE_W / 2;
      const y2 = b.y + CUBE_H / 2;
      const designed = isDesigned(ctx, e);
      parts.push(`<path d="M ${x1} ${y1} C ${x1 + 40} ${y1}, ${x2 - 40} ${y2}, ${x2} ${y2}" fill="none" `
        + `stroke="${designed ? 'var(--warn)' : 'var(--info)'}" stroke-width="1.3" `
        + `${designed ? 'stroke-dasharray="6 4"' : ''} marker-end="url(#${id}-f)"/>`);
      parts.push(text((x1 + x2) / 2, (y1 + y2) / 2 - 5, trunc(String(e.properties.calleeService ?? ''), 22), {
        'font-size': 9, 'text-anchor': 'middle', fill: 'var(--text-2)',
      }));
    }

    // 节点 + 制品
    for (const n of ordered) {
      const b = boxMap.get(n)!;
      const dep = deploymentOf(n)!;
      const sel = ctx.selection.nodeIds.includes(n.id);
      parts.push(cube(b.x, b.y, n.name, `replicas=${dep.replicas} · ${dep.namespace}`, sel));
      parts.push(text(b.x, b.y + CUBE_H + 16, trunc(dep.manifest, 30), { 'font-size': 8, fill: 'var(--text-2)' }));
      artifactsOf(n).forEach((a, i) => {
        const ay = b.y + CUBE_H + 24 + i * (ART_H + 3);
        parts.push(`<rect x="${b.x + 6}" y="${ay}" width="${ART_W - 12}" height="${ART_H}" rx="2" fill="var(--surface)" stroke="var(--edge)" stroke-width="1" stroke-dasharray="3 2"/>`);
        parts.push(text(b.x + 11, ay + 12, trunc(a, 24), { 'font-size': 8, fill: 'var(--text-2)' }));
      });
    }

    const canvasW = Math.max(760, bb.w + 60);
    const canvasH = Math.max(360, bb.h + 60);
    notes.push(`${ordered.length} 个部署节点全部来自真实 k8s 证据 (properties.deployment.manifest); 未部署的服务 (deployed=false) 不出现在本图。`);
    notes.push(`制品 = 该 crate 真实拥有的表 + proto 文件路径; 通信路径 = 真实 grpc-call 边, 黄色虚线 = Inferred。`);

    return {
      svg: svgRoot(canvasW, canvasH, 'UML 部署图', parts.join('')),
      width: canvasW,
      height: canvasH,
      title: 'UML 部署图',
      summary: `${ordered.length} 个部署节点 / ${artCounts.reduce((a, b) => a + b, 0)} 个制品`,
      notes,
    };
  },
};
