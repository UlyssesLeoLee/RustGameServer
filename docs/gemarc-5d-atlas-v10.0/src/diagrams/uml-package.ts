/**
 * diagrams/uml-package.ts — UML 包图 (spec `2D.diagrams.UML_Package`)
 *
 * 专有符号: 折叠包 (左上门 tab)、嵌套包。
 *
 * 数据来源:
 *   - containers: System (RustGameServer) → Floor (业务域) → Room (crate)
 *   - Room.properties.crate → 归属节点 (Api / Table)
 *   - 全部来自 architecture.json, 与 spatial.json 同源
 */

import type { DiagramRenderer, DiagramContext, DiagramResult } from './types';
import type { Container, GraphNode } from '../core/types';
import { esc, text, textLines, trunc, arrowDefs, svgRoot, domainVar, uid } from './svg';
import { serviceNodes, apiNodes, tableNodes, crateOf } from './data';

const CRATE_W = 214;
const ITEM_H = 13;
const HDR = 22;

interface CratePkg {
  name: string;
  domain: string;
  items: { label: string; kind: string }[];
  x: number; y: number; w: number; h: number;
}
interface DomainPkg {
  name: string;
  crates: CratePkg[];
  x: number; y: number; w: number; h: number;
}

export const umlPackage: DiagramRenderer = {
  type: 'UML_Package',
  label: 'UML 包图',
  symbols: ['折叠包 tab', '嵌套包', '包内成员'],
  dataSource: 'architecture.json: containers (System/Building/Floor/Room) + Service/Api/Table 节点',
  render(ctx: DiagramContext): DiagramResult {
    const id = uid('pkg');
    const notes: string[] = [];
    const services = serviceNodes(ctx);
    const apis = apiNodes(ctx);
    const tables = tableNodes(ctx);

    const floors = [...ctx.index.containers.values()]
      .filter((c) => c.kind === 'Floor')
      .sort((a, b) => a.name.localeCompare(b.name));

    const domPkgs: DomainPkg[] = [];
    for (const f of floors) {
      const rooms = f.childrenIds
        .map((id2) => ctx.index.containers.get(id2))
        .filter((c): c is Container => !!c && c.kind === 'Room')
        .sort((a, b) => a.name.localeCompare(b.name));
      const crates: CratePkg[] = [];
      for (const room of rooms) {
        const crate = String(room.properties?.crate ?? room.name);
        const svc = services.find((s) => crateOf(s) === crate);
        const items: { label: string; kind: string }[] = [];
        for (const a of apis.filter((x) => crateOf(x) === crate)) {
          items.push({ label: `«interface» ${a.name}`, kind: 'Api' });
        }
        for (const t of tables.filter((x) => crateOf(x) === crate)) {
          items.push({ label: `«table» ${t.name}`, kind: 'Table' });
        }
        if (!items.length && svc) items.push({ label: `«crate» ${svc.name}`, kind: 'Service' });
        const h = HDR + 6 + items.length * ITEM_H + 8;
        crates.push({ name: crate, domain: f.name, items, x: 0, y: 0, w: CRATE_W, h });
      }
      if (!crates.length) continue;
      const w = Math.max(...crates.map((c) => c.w)) + 22;
      const h = HDR + 8 + crates.reduce((n, c) => n + c.h + 8, 0);
      domPkgs.push({ name: f.name, crates, x: 0, y: 0, w, h });
    }

    if (!domPkgs.length) {
      return {
        svg: svgRoot(600, 120, 'UML 包图', text(20, 40, '容器层级为空。', { fill: 'var(--text-2)', 'font-size': 13 })),
        width: 600, height: 120, title: 'UML 包图', summary: '0 个包',
        notes: ['architecture.json 的 containers 中没有 Floor / Room 层级。'],
      };
    }

    // 域包按 3 列排布
    const COLS = 3;
    const colX: number[] = [];
    const colY: number[] = [];
    const colH: number[] = [];
    domPkgs.forEach((d, i) => {
      const c = i % COLS;
      if (colX.length <= c) { colX.push(0); colY.push(0); colH.push(0); }
      colX[c] = Math.max(colX[c], d.w);
      colH[c] += d.h + 10;
    });
    const colW = colX.map((w) => w + 26);
    const colOff: number[] = [];
    let acc = 0;
    for (const w of colW) { colOff.push(acc); acc += w; }

    domPkgs.forEach((d, i) => {
      const c = i % COLS;
      let y = 0;
      for (let j = 0; j < i; j++) {
        if (j % COLS === c) y += domPkgs[j].h + 10;
      }
      d.x = colOff[c] + 14;
      d.y = y;
      let cy = d.y + HDR + 8;
      for (const cr of d.crates) {
        cr.x = d.x + 12;
        cr.y = cy;
        cy += cr.h + 8;
      }
    });

    const sysW = Math.max(...domPkgs.map((d) => d.x + d.w)) + 20;
    const sysH = Math.max(...domPkgs.map((d) => d.y + d.h)) + 20;
    const OX = 20;
    const OY = 52;

    const parts: string[] = [arrowDefs(id)];
    parts.push(text(20, 22, `UML 包图 — ${domPkgs.length} 个业务域包 / ${domPkgs.reduce((n, d) => n + d.crates.length, 0)} 个 crate 包`, {
      'font-size': 13, 'font-weight': 600,
    }));
    parts.push(text(20, 38, '层级 = containers 的 System → Floor(业务域) → Room(crate) → 成员 (Api / Table)。', {
      'font-size': 10, fill: 'var(--text-2)',
    }));

    // System 包 (最外层)
    parts.push(`<path d="M ${OX} ${OY} H ${OX + 70} V ${OY - 12} H ${OX + 120} V ${OY}" fill="none" stroke="var(--text-2)" stroke-width="1.4"/>`);
    parts.push(`<rect x="${OX}" y="${OY}" width="${sysW}" height="${sysH}" fill="none" stroke="var(--text-2)" stroke-width="1.4"/>`);
    parts.push(text(OX + 8, OY + 16, '«system» RustGameServer', { 'font-size': 12, 'font-weight': 600 }));

    for (const d of domPkgs) {
      const x = OX + d.x;
      const y = OY + d.y;
      parts.push(`<path d="M ${x} ${y} H ${x + 56} V ${y - 10} H ${x + 100} V ${y}" fill="none" stroke="${domainVar(d.name)}" stroke-width="1.2"/>`);
      parts.push(`<rect x="${x}" y="${y}" width="${d.w}" height="${d.h}" fill="none" stroke="${domainVar(d.name)}" stroke-width="1.2"/>`);
      parts.push(text(x + 7, y + 15, `«domain» ${d.name}`, { 'font-size': 11, 'font-weight': 600, fill: domainVar(d.name) }));

      for (const cr of d.crates) {
        const cx = OX + cr.x;
        const cy = OY + cr.y;
        parts.push(`<path d="M ${cx} ${cy} H ${cx + 50} V ${cy - 9} H ${cx + 92} V ${cy}" fill="none" stroke="var(--edge)" stroke-width="1.1"/>`);
        parts.push(`<rect x="${cx}" y="${cy}" width="${cr.w}" height="${cr.h}" fill="var(--surface)" stroke="var(--edge)" stroke-width="1.1"/>`);
        parts.push(text(cx + 7, cy + 15, trunc(cr.name, 30), { 'font-size': 10, 'font-weight': 600 }));
        cr.items.forEach((it, i) => {
          parts.push(text(cx + 9, cy + HDR + 4 + i * ITEM_H, it.label, {
            'font-size': 9, fill: it.kind === 'Api' ? 'var(--info)' : 'var(--text-2)',
            'font-style': it.kind === 'Api' ? 'italic' : 'normal',
          }));
        });
        if (!cr.items.length) {
          parts.push(text(cx + 9, cy + HDR + 4, '(无成员)', { 'font-size': 9, fill: 'var(--text-2)' }));
        }
      }
    }

    const canvasW = OX + sysW + 20;
    const canvasH = OY + sysH + 20;
    notes.push(`包图覆盖 ${domPkgs.length} 个业务域 / ${domPkgs.reduce((n, d) => n + d.crates.length, 0)} 个 crate (全部来自 containers, 无抽样)。`);

    return {
      svg: svgRoot(canvasW, canvasH, 'UML 包图', parts.join('')),
      width: canvasW,
      height: canvasH,
      title: 'UML 包图',
      summary: `${domPkgs.length} 个域包 / ${domPkgs.reduce((n, d) => n + d.crates.length, 0)} 个 crate 包`,
      notes,
    };
  },
};
