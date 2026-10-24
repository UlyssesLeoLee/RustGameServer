/**
 * diagrams/security.ts — 安全视图 (spec `2D.diagrams.Security`)
 *
 * 专有符号: 信任边界 (虚线圆角框)、主体 (subject)、客体 (object)、
 *           跨边界访问 (带箭头)。
 *
 * 数据来源:
 *   - 外层信任边界 = 真实 k8s 命名空间 (Service.properties.deployment.namespace)
 *   - 内层信任边界 = 真实业务域 (Security 域, 来自 containers Floor)
 *   - 主体 = 真实跨边界调用方 crate
 *   - 客体 = Security 域内的真实服务与表
 */

import type { DiagramRenderer, DiagramContext, DiagramResult } from './types';
import { esc, text, textLines, trunc, arrowDefs, svgRoot, uid } from './svg';
import { allEdges, isDesigned, deploymentOf, deployedServices, serviceNodes, tableNodes, crateOf, domainFacets } from './data';

export const security: DiagramRenderer = {
  type: 'Security',
  label: '安全视图',
  symbols: ['信任边界虚线框', '主体', '客体', '跨边界访问'],
  dataSource: 'architecture.json: deployment.namespace + 业务域 containers + grpc-call 边 + Table 节点',
  render(ctx: DiagramContext): DiagramResult {
    const id = uid('sec');
    const notes: string[] = [];

    const deployed = deployedServices(ctx);
    const namespaces = [...new Set(deployed.map((n) => deploymentOf(n)!.namespace).filter(Boolean))];
    const nsText = namespaces.join(', ') || 'rust-game-server';

    // 内层边界 = Security 域
    const secFacet = domainFacets(ctx).find((f) => f.name === 'Security');
    const secCrates = new Set((secFacet?.services ?? []).map((s) => crateOf(s)));
    const secServices = serviceNodes(ctx).filter((s) => secCrates.has(crateOf(s)));
    const secTables = tableNodes(ctx).filter((t) => secCrates.has(crateOf(t)));

    // 跨边界调用: 调用方不在 Security 域, 被调方在 Security 域
    const crossings = allEdges(ctx).filter((e) => {
      const c = String(e.properties.calleeCrate ?? '');
      const caller = String(e.properties.callerCrate ?? '');
      return secCrates.has(c) && !secCrates.has(caller);
    });

    const subjects = [...new Set(crossings.map((e) => String(e.properties.callerCrate ?? '')))].sort();

    const SUB_X = 70;
    const BOUND_X = 300;
    const BOUND_W = 460;
    const objH = Math.max(secServices.length, Math.min(secTables.length, 4));
    const BOUND_H = 70 + (objH + 1) * 44;

    const parts: string[] = [arrowDefs(id)];
    parts.push(text(20, 22, `安全视图 — ${subjects.length} 个跨边界主体 / ${secServices.length} 个 Security 域服务`, {
      'font-size': 13, 'font-weight': 600,
    }));
    parts.push(text(20, 38, '信任边界来自真实 k8s 命名空间与业务域划分; 跨边界访问 = 真实 grpc-call 边。', {
      'font-size': 10, fill: 'var(--text-2)',
    }));

    // 外层边界: 命名空间
    parts.push(`<rect x="24" y="52" width="${BOUND_X + BOUND_W - 24 + 40}" height="${BOUND_H + 80}" rx="12" `
      + `fill="none" stroke="var(--info)" stroke-width="1.4" stroke-dasharray="10 6"/>`);
    parts.push(text(34, 46, `«trust boundary» k8s namespace: ${nsText}`, { 'font-size': 10, fill: 'var(--info)' }));

    // 内层边界: Security 域
    parts.push(`<rect x="${BOUND_X}" y="88" width="${BOUND_W}" height="${BOUND_H}" rx="10" `
      + `fill="none" stroke="var(--warn)" stroke-width="1.4" stroke-dasharray="8 5"/>`);
    parts.push(text(BOUND_X + 10, 106, '«trust boundary» 业务域 Security (crates + tables)', {
      'font-size': 10, fill: 'var(--warn)',
    }));

    // 客体
    secServices.slice(0, 6).forEach((s, i) => {
      const y = 126 + i * 44;
      parts.push(`<rect x="${BOUND_X + 20}" y="${y}" width="${BOUND_W - 40}" height="34" rx="3" fill="var(--surface)" stroke="var(--text-2)" stroke-width="1.1"/>`);
      parts.push(text(BOUND_X + 30, y + 15, `«object» ${trunc(s.name, 30)}`, { 'font-size': 10, 'font-weight': 600 }));
      parts.push(text(BOUND_X + 30, y + 27, `crate=${trunc(crateOf(s), 24)} · deployed=${s.properties.deployed === true}`, {
        'font-size': 8, fill: 'var(--text-2)',
      }));
    });
    secTables.slice(0, 4).forEach((t, i) => {
      const y = 126 + (secServices.length + i) * 44;
      if (y > 88 + BOUND_H - 10) return;
      parts.push(`<rect x="${BOUND_X + 20}" y="${y}" width="${BOUND_W - 40}" height="34" rx="3" fill="var(--surface)" stroke="var(--edge)" stroke-width="1" stroke-dasharray="3 2"/>`);
      parts.push(text(BOUND_X + 30, y + 15, `«data object» ${trunc(t.name, 30)}`, { 'font-size': 10 }));
      parts.push(text(BOUND_X + 30, y + 27, `crate=${trunc(crateOf(t), 24)}`, { 'font-size': 8, fill: 'var(--text-2)' }));
    });

    // 主体
    subjects.slice(0, 6).forEach((s, i) => {
      const y = 120 + i * 52;
      parts.push(`<rect x="${SUB_X - 56}" y="${y}" width="112" height="30" rx="15" fill="var(--surface)" stroke="var(--text-2)" stroke-width="1.2"/>`);
      parts.push(text(SUB_X, y + 19, `«subject» ${trunc(s, 16)}`, { 'font-size': 9, 'text-anchor': 'middle' }));
    });

    // 跨边界访问
    for (const e of crossings) {
      const caller = String(e.properties.callerCrate ?? '');
      const callee = String(e.properties.calleeCrate ?? '');
      const si = subjects.indexOf(caller);
      const oi = secServices.findIndex((s) => crateOf(s) === callee);
      if (si < 0 || oi < 0) continue;
      const y1 = 120 + si * 52 + 15;
      const y2 = 126 + oi * 44 + 17;
      const designed = isDesigned(ctx, e);
      parts.push(`<line x1="${SUB_X + 56}" y1="${y1}" x2="${BOUND_X + 20}" y2="${y2}" `
        + `stroke="${designed ? 'var(--warn)' : 'var(--fail)'}" stroke-width="1.3" `
        + `${designed ? 'stroke-dasharray="6 4"' : ''} marker-end="url(#${id}-f)"/>`);
      parts.push(text((SUB_X + BOUND_X) / 2, (y1 + y2) / 2 - 4, trunc(String(e.properties.calleeService ?? ''), 20), {
        'font-size': 8, 'text-anchor': 'middle', fill: 'var(--text)',
      }));
      parts.push(text((SUB_X + BOUND_X) / 2, (y1 + y2) / 2 + 8, trunc(String(e.properties.fileLine ?? ''), 30), {
        'font-size': 7, 'text-anchor': 'middle', fill: 'var(--text-2)',
      }));
    }

    const canvasW = Math.max(880, BOUND_X + BOUND_W + 80);
    const canvasH = Math.max(380, BOUND_H + 170);

    notes.push(`Security 域含 ${secServices.length} 个 crate / ${secTables.length} 张表 (真实 containers + Table 节点); 未部署的 Security 服务仍计入边界 (它们存在, 只是无 k8s 证据)。`);
    notes.push('本图不推断认证/授权机制 — 采集模型未含 mTLS/证书字段, 故只画边界与真实调用关系。');
    if (!crossings.length) notes.push('当前模型中没有跨 Security 域的真实调用边。');

    return {
      svg: svgRoot(canvasW, canvasH, '安全视图', parts.join('')),
      width: canvasW,
      height: canvasH,
      title: '安全视图',
      summary: `${subjects.length} 个跨边界主体 / ${secServices.length} 个客体服务 / ${crossings.length} 条跨边界访问`,
      notes,
    };
  },
};
