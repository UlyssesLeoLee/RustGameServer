/**
 * d1.ts — 1D Prompt 工作区 (spec `immutable_dimensions.1D`)
 *
 * 1D = 提示词而非图。动作: 生成 / 编辑 / 校验 / 复制 / 导出 / 版本差分 / 链接证据。
 * schemas: promptId, version, objective, requirementIds, nodeEdgeIds, caseIds,
 *          inputs, constraints, outputs, acceptance, evidenceIds
 */

import type { DimensionModule, DimensionContext } from '../core/app';
import { esc } from '../core/app';
import { bus } from '../core/bus';
import type { PromptRecord } from '../core/types';

interface PromptState {
  records: PromptRecord[];
  currentId: string | null;
  drafts: Map<string, string>;   // promptId -> 序列化正文 (用于版本差分)
  baseline: Map<string, string>;
}

let state: PromptState = { records: [], currentId: null, drafts: new Map(), baseline: new Map() };

function serialize(p: PromptRecord): string {
  return [
    `# ${p.promptId}  (v${p.version})`,
    '',
    `## objective`,
    p.objective,
    '',
    `## requirementIds`,
    p.requirementIds.map((x) => `- ${x}`).join('\n') || '- (无)',
    '',
    `## nodeEdgeIds`,
    p.nodeEdgeIds.map((x) => `- ${x}`).join('\n') || '- (无)',
    '',
    `## caseIds`,
    p.caseIds.map((x) => `- ${x}`).join('\n') || '- (无)',
    '',
    `## inputs`,
    p.inputs.map((x) => `- ${x}`).join('\n') || '- (无)',
    '',
    `## constraints`,
    p.constraints.map((x) => `- ${x}`).join('\n') || '- (无)',
    '',
    `## outputs`,
    p.outputs.map((x) => `- ${x}`).join('\n') || '- (无)',
    '',
    `## acceptance`,
    p.acceptance.map((x) => `- ${x}`).join('\n') || '- (无)',
    '',
    `## evidenceIds`,
    p.evidenceIds.map((x) => `- ${x}`).join('\n') || '- (无)',
  ].join('\n');
}

export const d1: DimensionModule = {
  id: '1D',
  title: '1D Prompt',
  mount(ctx: DimensionContext) {
    const { app, host, toolbar, sidebar } = ctx;
    // prompts 由 main.ts 在构建期内联并挂到 window
    state.records = (window as unknown as { __ATLAS_PROMPTS?: PromptRecord[] }).__ATLAS_PROMPTS ?? [];
    state.currentId = state.records[0]?.promptId ?? null;
    for (const p of state.records) {
      const s = serialize(p);
      state.baseline.set(p.promptId, s);
      state.drafts.set(p.promptId, s);
    }

    if (!state.records.length) {
      host.innerHTML = `<div class="panel"><h3>1D 无 Prompt 数据</h3>
        <p class="empty-hint">未找到 models/prompts.json 的内联数据。状态: NOT_TESTED。</p></div>`;
      return;
    }

    /* ── 工具条 ─────────────────────────────────────────────── */
    toolbar.innerHTML = `
      <button class="tool-btn" data-act="copy">复制</button>
      <button class="tool-btn" data-act="export">导出 .md</button>
      <button class="tool-btn" data-act="diff">版本差分</button>
      <span class="tool-sep"></span>
      <button class="tool-btn" data-act="validate">校验</button>
      <button class="tool-btn" data-act="evidence">链接证据</button>
      <span class="tool-sep"></span>
      <span class="tool-label" id="p1d-status">${state.records.length} 条 Prompt</span>`;

    /* ── 侧栏: Prompt 列表 ──────────────────────────────────── */
    sidebar.innerHTML = `
      <div class="panel" style="padding:8px">
        <input id="p1d-filter" type="search" placeholder="筛选 prompt / objective…"
               style="width:100%;height:28px;background:var(--raised);color:var(--text);
                      border:1px solid var(--edge);border-radius:6px;padding:0 8px;font-size:12px">
        <div id="p1d-list" style="margin-top:8px"></div>
      </div>`;

    const listEl = sidebar.querySelector('#p1d-list') as HTMLElement;
    const renderList = (q = '') => {
      const items = state.records.filter((p) =>
        !q || p.promptId.toLowerCase().includes(q.toLowerCase()) || p.objective.toLowerCase().includes(q.toLowerCase()));
      listEl.innerHTML = items.map((p) => `
        <button class="tool-btn" data-pid="${p.promptId}"
          style="display:block;width:100%;text-align:left;margin-bottom:3px;
                 ${p.promptId === state.currentId ? 'border-color:var(--focus);' : ''}">
          <code>${esc(p.promptId)}</code><br>
          <span style="font-size:11px;color:var(--text-2)">${esc(p.objective.slice(0, 42))}</span>
        </button>`).join('');
      listEl.querySelectorAll('[data-pid]').forEach((b) => {
        b.addEventListener('click', () => { state.currentId = (b as HTMLElement).dataset.pid!; render(); });
      });
    };

    const editor = document.createElement('div');
    editor.id = 'p1d-editor';
    editor.style.cssText = 'padding:12px;height:100%;';
    host.appendChild(editor);

    const render = () => {
      const p = state.records.find((x) => x.promptId === state.currentId);
      if (!p) return;
      /* 1D → 2D の鎖: 選択中の Prompt が定義するスコープを公開する。
       * 2D はこのスコープで「何を描くか」を決める。 */
      bus.setPrompt({
        id: p.promptId, title: p.objective,
        nodeIds: p.nodeEdgeIds, edgeIds: [],
      });
      const cur = state.drafts.get(p.promptId) ?? '';
      const dirty = cur !== state.baseline.get(p.promptId);
      editor.innerHTML = `
        <div class="panel">
          <h3><code>${esc(p.promptId)}</code> v${esc(p.version)}
            ${dirty ? '<span class="badge badge--Designed">已修改</span>' : '<span class="badge badge--Present">与基线一致</span>'}
          </h3>
          <p class="empty-hint">1D 是提示词不是图。下方正文可直接编辑, 证据 ID 指向仓库 fileLine。</p>
        </div>
        <textarea id="p1d-text" spellcheck="false"
          style="width:100%;min-height:calc(100% - 140px);background:var(--raised);color:var(--text);
                 border:1px solid var(--edge);border-radius:10px;padding:12px;
                 font:13px/20px var(--mono);resize:vertical">${esc(cur)}</textarea>`;
      const ta = editor.querySelector('#p1d-text') as HTMLTextAreaElement;
      ta.addEventListener('input', () => {
        state.drafts.set(p.promptId, ta.value);
        const b = editor.querySelector('.badge') as HTMLElement;
        const isDirty = ta.value !== state.baseline.get(p.promptId);
        b.textContent = isDirty ? '已修改' : '与基线一致';
        b.className = `badge badge--${isDirty ? 'Designed' : 'Present'}`;
      });
      renderList((sidebar.querySelector('#p1d-filter') as HTMLInputElement).value);
    };

    (sidebar.querySelector('#p1d-filter') as HTMLInputElement)
      .addEventListener('input', (e) => renderList((e.target as HTMLInputElement).value));

    /* ── 动作 ───────────────────────────────────────────────── */
    const current = () => state.records.find((x) => x.promptId === state.currentId);
    toolbar.querySelector('[data-act="copy"]')?.addEventListener('click', async () => {
      const p = current(); if (!p) return;
      const text = state.drafts.get(p.promptId) ?? '';
      try {
        await navigator.clipboard.writeText(text);
        ctx.status(`已复制 ${p.promptId} (${text.length} 字符)`);
      } catch {
        // file:// 下 clipboard API 可能不可用 → 回退到选中文本
        const ta = editor.querySelector('#p1d-text') as HTMLTextAreaElement;
        ta.select();
        ctx.status('剪贴板 API 不可用 (file:// 限制), 已选中全文, 请手动 Ctrl+C');
      }
    });

    toolbar.querySelector('[data-act="export"]')?.addEventListener('click', () => {
      const p = current(); if (!p) return;
      const blob = new Blob([state.drafts.get(p.promptId) ?? ''], { type: 'text/markdown;charset=utf-8' });
      const a = document.createElement('a');
      a.href = URL.createObjectURL(blob);
      a.download = `${p.promptId}.md`;
      a.click();
      setTimeout(() => URL.revokeObjectURL(a.href), 1000);
      ctx.status(`已导出 ${p.promptId}.md`);
    });

    toolbar.querySelector('[data-act="diff"]')?.addEventListener('click', () => {
      const p = current(); if (!p) return;
      const base = state.baseline.get(p.promptId)!.split('\n');
      const now = (state.drafts.get(p.promptId) ?? '').split('\n');
      const rows: string[] = [];
      const max = Math.max(base.length, now.length);
      for (let i = 0; i < max; i++) {
        const b = base[i] ?? '', n = now[i] ?? '';
        if (b === n) { if (b.trim()) rows.push(`  ${esc(b)}`); continue; }
        rows.push(`<div style="background:color-mix(in srgb,var(--fail) 18%,transparent);margin:0 -8px;padding:1px 8px">- ${esc(b)}</div>`);
        rows.push(`<div style="background:color-mix(in srgb,var(--success) 18%,transparent);margin:0 -8px;padding:1px 8px">+ ${esc(n)}</div>`);
      }
      const root = document.getElementById('modal-root')!;
      root.hidden = false;
      root.innerHTML = `
        <div class="modal" role="dialog" aria-modal="true" aria-label="版本差分">
          <div class="modal-head"><strong>${esc(p.promptId)} 版本差分</strong>
            <button class="icon-btn" data-close aria-label="关闭">✕</button></div>
          <div class="modal-body"><pre style="margin:0;font:12px/18px var(--mono);white-space:pre-wrap">${rows.join('\n')}</pre></div>
        </div>`;
      root.querySelector('[data-close]')?.addEventListener('click', () => { root.hidden = true; root.innerHTML = ''; });
    });

    toolbar.querySelector('[data-act="validate"]')?.addEventListener('click', () => {
      const p = current(); if (!p) return;
      const problems: string[] = [];
      if (!p.objective.trim()) problems.push('objective 为空');
      if (!p.requirementIds.length) problems.push('无 requirementIds');
      if (!p.acceptance.length) problems.push('无 acceptance (不可验证)');
      if (!p.evidenceIds.length) problems.push('无 evidenceIds (未链接真实来源)');
      const orphan = p.evidenceIds.filter((id) => !app.evidence.has(id));
      if (orphan.length) problems.push(`证据 ID 不存在: ${orphan.join(', ')}`);
      const root = document.getElementById('modal-root')!;
      root.hidden = false;
      root.innerHTML = `
        <div class="modal" role="dialog" aria-modal="true" aria-label="校验结果">
          <div class="modal-head"><strong>${esc(p.promptId)} 校验</strong>
            <button class="icon-btn" data-close aria-label="关闭">✕</button></div>
          <div class="modal-body">
            ${problems.length
              ? `<ul>${problems.map((x) => `<li style="color:var(--warn)">${esc(x)}</li>`).join('')}</ul>`
              : `<p style="color:var(--success)">✓ 通过 — 字段完整且证据 ID 全部可解析。</p>`}
          </div>
        </div>`;
      root.querySelector('[data-close]')?.addEventListener('click', () => { root.hidden = true; root.innerHTML = ''; });
      ctx.status(problems.length ? `校验: ${problems.length} 个问题` : '校验: 通过');
    });

    toolbar.querySelector('[data-act="evidence"]')?.addEventListener('click', () => {
      const p = current(); if (!p) return;
      const rows = p.evidenceIds.map((id) => {
        const e = app.evidence.get(id);
        if (!e) return `<tr><td><code>${esc(id)}</code></td><td colspan="2" style="color:var(--fail)">证据缺失</td></tr>`;
        return `<tr><td><code>${esc(e.id)}</code></td><td>${esc(e.kind)}</td>
          <td><code>${esc(e.fileLine)}</code> <span class="empty-hint">rev ${esc(e.revision)}</span></td></tr>`;
      }).join('');
      const root = document.getElementById('modal-root')!;
      root.hidden = false;
      root.innerHTML = `
        <div class="modal" role="dialog" aria-modal="true" aria-label="证据链接">
          <div class="modal-head"><strong>${esc(p.promptId)} 证据 (${p.evidenceIds.length})</strong>
            <button class="icon-btn" data-close aria-label="关闭">✕</button></div>
          <div class="modal-body">
            <table class="data"><thead><tr><th>ID</th><th>Kind</th><th>fileLine (rev)</th></tr></thead>
            <tbody>${rows || '<tr><td colspan="3">无</td></tr>'}</tbody></table>
          </div>
        </div>`;
      root.querySelector('[data-close]')?.addEventListener('click', () => { root.hidden = true; root.innerHTML = ''; });
    });

    render();
    ctx.status(`1D — ${state.records.length} 条 Prompt (由 models/prompts.json 派生)`);
  },
};
