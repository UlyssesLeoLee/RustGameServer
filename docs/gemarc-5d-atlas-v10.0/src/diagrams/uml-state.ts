/**
 * diagrams/uml-state.ts — UML 状态图 (spec `2D.diagrams.UML_State`)
 *
 * 专有符号: 圆角矩形状态、实心圆初始状态、双圆终止状态、带 guard 的转换。
 *
 * 数据来源:
 *   - models/causal-rules.json 的真实 when / then / else / retry / compensation
 *   - models/cases.json 的真实 CaseKind + expectedOutcome (无规则时的回退)
 *   - 状态的命名与转换标签 100% 来自真实规则, 不编造状态名
 */

import type { DiagramRenderer, DiagramContext, DiagramResult } from './types';
import { esc, text, textLines, trunc, arrowDefs, svgRoot, roundRect, uid } from './svg';
import { serviceNodes } from './data';

const SW = 210;
const SH = 54;

interface StateDef {
  key: string;
  label: string;
  sub: string;
  initial?: boolean;
  final?: boolean;
  x: number; y: number; w: number; h: number;
}

export const umlState: DiagramRenderer = {
  type: 'UML_State',
  label: 'UML 状态图',
  symbols: ['圆角矩形状态', '实心圆初始', '双圆终止', 'guard 转换'],
  dataSource: 'models/causal-rules.json (when/then/else) + models/cases.json (CaseKind/expectedOutcome)',
  render(ctx: DiagramContext): DiagramResult {
    const id = uid('st');
    const notes: string[] = [];

    // 作用域: 焦点服务 > 规则作用域 > 用例作用域
    const focus = ctx.selection.focusId;
    let scopeIds = new Set<string>();
    if (focus && ctx.index.nodes.has(focus)) scopeIds.add(focus);
    if (!scopeIds.size) {
      for (const r of ctx.aux.rules) for (const n of r.scopeNodeIds) scopeIds.add(n);
    }
    if (!scopeIds.size) {
      for (const c of ctx.aux.cases) for (const n of c.nodeIds) scopeIds.add(n);
    }

    const rules = ctx.aux.rules.filter((r) => r.scopeNodeIds.some((n) => scopeIds.has(n)));
    const cases = ctx.aux.cases.filter((c) => c.nodeIds.some((n) => scopeIds.has(n)));

    const states: StateDef[] = [];
    const transitions: { from: string; to: string; label: string; dashed?: boolean; note?: string }[] = [];

    const scopeName = (() => {
      const n = [...scopeIds].map((id2) => ctx.index.nodes.get(id2)).find(Boolean);
      return n ? n!.name : '全域';
    })();

    if (rules.length) {
      states.push({ key: 'start', label: '«start»', sub: '', initial: true, x: 0, y: 0, w: 26, h: 26 });
      for (const r of rules) {
        states.push({ key: r.ruleId, label: r.name, sub: r.ruleId, x: 0, y: 0, w: SW, h: SH });
        transitions.push({ from: 'start', to: r.ruleId, label: r.when });
        transitions.push({ from: r.ruleId, to: 'end', label: r.then });
        if (r.else) transitions.push({ from: r.ruleId, to: 'end', label: r.else, dashed: true });
      }
      states.push({ key: 'end', label: '«end»', sub: '', final: true, x: 0, y: 0, w: 30, h: 30 });
      notes.push(`状态名与 guard 全部来自 ${rules.length} 条真实因果规则 (${rules.map((r) => r.ruleId).join(', ')})。`);
      notes.push('重试/超时等真实参数写在转换注释: '
        + rules.map((r) => {
          const bits: string[] = [];
          if (r.retry) bits.push(`retry.max=${r.retry.max} backoff=${r.retry.backoffMs}ms`);
          if (r.timeoutMs) bits.push(`timeout=${r.timeoutMs}ms`);
          if (r.compensation) bits.push(`compensation=${trunc(r.compensation, 28)}`);
          return bits.length ? `${r.ruleId}: ${bits.join(', ')}` : '';
        }).filter(Boolean).join(' | ') || '(无)');
    } else if (cases.length) {
      states.push({ key: 'start', label: '«start»', sub: '', initial: true, x: 0, y: 0, w: 26, h: 26 });
      let prev = 'start';
      for (const c of cases) {
        states.push({ key: c.caseId, label: c.title, sub: `${c.caseId} · ${c.kind}`, x: 0, y: 0, w: SW, h: SH });
        transitions.push({ from: prev, to: c.caseId, label: c.expectedOutcome });
        prev = c.caseId;
      }
      states.push({ key: 'end', label: '«end»', sub: '', final: true, x: 0, y: 0, w: 30, h: 30 });
      notes.push(`该作用域无因果规则, 回退为 ${cases.length} 条真实 4D 用例状态 (${cases.map((c) => c.caseId).join(', ')})。`);
    } else {
      notes.push('作用域内既无因果规则也无 4D 用例 — 仅绘制初始/终止状态, 不虚构中间状态。');
      states.push({ key: 'start', label: '«start»', sub: '', initial: true, x: 0, y: 0, w: 26, h: 26 });
      states.push({ key: 'end', label: '«end»', sub: '', final: true, x: 0, y: 0, w: 30, h: 30 });
      transitions.push({ from: 'start', to: 'end', label: '(无真实转换定义)' });
    }

    // 纵向布局
    const CX = 240;
    let y = 40;
    for (const s of states) {
      if (s.initial) { s.x = CX - 13; s.y = y; y += 56; }
      else if (s.final) { s.x = CX - 15; s.y = y; y += 46; }
      else { s.x = CX - SW / 2; s.y = y; y += SH + 44; }
    }
    const canvasH = y + 20;
    const canvasW = 620;

    const parts: string[] = [arrowDefs(id)];
    parts.push(text(20, 22, `UML 状态图 — 作用域: ${scopeName}`, { 'font-size': 13, 'font-weight': 600 }));
    parts.push(text(20, 38, '状态名 / guard / then / else 100% 来自 models/causal-rules.json 与 models/cases.json。', {
      'font-size': 10, fill: 'var(--text-2)',
    }));

    const byKey = new Map(states.map((s) => [s.key, s]));

    // 转换 (先画, 状态压在上面)
    for (const t of transitions) {
      const a = byKey.get(t.from);
      const b = byKey.get(t.to);
      if (!a || !b) continue;
      const x1 = a.x + a.w / 2;
      const y1 = a.y + a.h;
      const x2 = b.x + b.w / 2;
      const y2 = b.y;
      parts.push(`<line x1="${x1}" y1="${y1}" x2="${x2}" y2="${y2}" stroke="var(--text-2)" stroke-width="1.2" `
        + `${t.dashed ? 'stroke-dasharray="5 4"' : ''} marker-end="url(#${id}-f)"/>`);
      parts.push(text((x1 + x2) / 2 + 8, (y1 + y2) / 2, `[${trunc(t.label, 42)}]`, {
        'font-size': 9, fill: t.dashed ? 'var(--warn)' : 'var(--text-2)',
      }));
    }

    // 状态
    for (const s of states) {
      if (s.initial) {
        parts.push(`<circle cx="${s.x + 13}" cy="${s.y + 13}" r="11" fill="var(--text)" stroke="var(--text)" stroke-width="1.2"/>`);
        continue;
      }
      if (s.final) {
        parts.push(`<circle cx="${s.x + 15}" cy="${s.y + 15}" r="13" fill="var(--surface)" stroke="var(--text)" stroke-width="1.4"/>`);
        parts.push(`<circle cx="${s.x + 15}" cy="${s.y + 15}" r="9" fill="none" stroke="var(--text)" stroke-width="1.2"/>`);
        continue;
      }
      parts.push(`<path d="${roundRect(s.x, s.y, s.w, s.h, 12)}" fill="var(--surface)" stroke="var(--text-2)" stroke-width="1.3"/>`);
      parts.push(text(s.x + s.w / 2, s.y + 21, trunc(s.label, 30), { 'font-size': 11, 'text-anchor': 'middle', 'font-weight': 600 }));
      if (s.sub) parts.push(text(s.x + s.w / 2, s.y + 37, s.sub, { 'font-size': 9, 'text-anchor': 'middle', fill: 'var(--text-2)' }));
    }

    return {
      svg: svgRoot(canvasW, canvasH, 'UML 状态图', parts.join('')),
      width: canvasW,
      height: canvasH,
      title: 'UML 状态图',
      summary: `${states.length} 个状态 / ${transitions.length} 条转换 (${rules.length} 规则 / ${cases.length} 用例)`,
      notes,
    };
  },
};
