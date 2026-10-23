/**
 * d5.ts — 5D 因果実験台
 *
 * spec `immutable_dimensions.5D`:
 *   "可干预且可新增互动元素的因果 4D，**必须复用 4D 引擎与 3D 场景**"
 *
 * したがって 5D は:
 *   - エンジン: src/sim/engine.ts (4D と同じ)
 *   - ステージ: src/core/scene-host.ts (3D/4D と同じ 1 個の WebGL キャンバス)
 *   - ベースライン: bus 上の 4D Trace (同一 Case・同一 seed)
 * を使う。独自の世界を持たない。
 *
 * 因果提示 (Overlay / SplitSynchronized3D) は**受動的な badge ではなく実描画**:
 *   - Overlay : 分岐ノードをリング色で強調
 *   - Split   : 1 キャンバスを左右に区切り、左=ベースライン / 右=分岐を別状態で 2 パス描画
 */

import type { DimensionModule, DimensionContext } from '../core/app';
import { esc } from '../core/app';
import { bus } from '../core/bus';
import { getSceneHost } from '../core/scene-host';
import { runSimulation, applyInterventions, type ScenarioGraph, type SimNode, type SimEdge } from '../sim/engine';
import { runBranch } from '../sim/causal';
import { RING_STATE_COLOR, DOMAIN_COLOR } from '../core/tokens';
import type { CaseRecord, CausalRule, Intervention } from '../core/types';

interface Models { cases?: CaseRecord[]; rules?: CausalRule[] }
const models = (): Models => (window as unknown as { __ATLAS_MODELS?: Models }).__ATLAS_MODELS ?? {};

type CausalMode = 'none' | 'overlay' | 'split';

export const d5: DimensionModule = {
  id: '5D',
  title: '5D 因果実験',
  usesTimeline: true,
  mount(ctx: DimensionContext) {
    const { host, toolbar, sidebar } = ctx;
    const cases = models().cases ?? [];
    const rules = models().rules ?? [];

    if (!cases.length) {
      host.innerHTML = '<div class="panel"><h3>5D 無 Case データ</h3><p class="empty-hint">状態: NOT_TESTED</p></div>';
      return;
    }

    /* ── 共有ステージ (3D/4D と同一) ───────────────────────── */
    const mountEl = document.createElement('div');
    mountEl.style.cssText = 'position:absolute;inset:0;';
    host.appendChild(mountEl);
    const stage = getSceneHost(
      ctx.index.model, ctx.index as never,
      (window as unknown as { __ATLAS_MODELS?: { spatial?: never } }).__ATLAS_MODELS?.spatial as never,
      (window as unknown as { __ATLAS_MODELS?: { rules?: never[] } }).__ATLAS_MODELS?.rules ?? [],
      (window as unknown as { __ATLAS_MODELS?: { cases?: never[] } }).__ATLAS_MODELS?.cases ?? [],
    );
    const has3D = stage.attachTo(mountEl);

    /* ── グラフ ─────────────────────────────────────────────── */
    const nodes = new Map<string, SimNode>();
    const edges = new Map<string, SimEdge>();
    for (const n of ctx.index.nodes.values()) {
      nodes.set(n.id, { id: n.id, name: n.name, kind: n.kind, domainId: n.domainId, evidenceIds: n.evidenceIds });
    }
    for (const e of [...ctx.index.edges.values(), ...ctx.index.designedEdges.values()]) {
      if (!e.sourceId || !e.targetId) continue;
      edges.set(e.id, { id: e.id, sourceId: e.sourceId, targetId: e.targetId, type: e.type, directed: e.directed, guardId: e.guardId, verification: e.verification });
    }
    const graph: ScenarioGraph = { nodes, edges, baseRevision: ctx.index.model.meta.revision };

    let currentCase = cases.find((c) => c.kind === 'FailurePropagation') ?? cases[0];
    let interventions: Intervention[] = [];
    let rewireTarget = '';
    let mode: CausalMode = 'none';
    let result: ReturnType<typeof runBranch> | null = null;

    const nodeOptions = [...ctx.index.nodes.values()]
      .filter((n) => n.kind === 'Service')
      .map((n) => `<option value="${esc(n.id)}">${esc(n.name)}</option>`).join('');

    /* ── A/B 実行 ──────────────────────────────────────────── */
    const runAB = () => {
      const base = {
        caseId: currentCase.caseId, caseTitle: currentCase.title,
        steps: currentCase.steps, rules, graph, seed: 20261021,
        modelRevision: ctx.index.model.meta.revision,
      };
      result = runBranch(base, interventions, ctx.index);
      bus.publishBranch({
        traceId: result.branch.traceId,
        events: result.branch.events,
        finalStates: result.branch.finalStates,
      });
      // ステージへ因果提示を適用 (overlay / split は実描画)
      const diverged = Object.keys(result.explanation.terminalDiff);
      stage.setCausalPresentation({
        mode, baseStates: result.baseline.finalStates,
        branchStates: result.branch.finalStates, divergedNodeIds: diverged,
      });
      const v = toolbar.querySelector('#d5-verdict');
      if (v) {
        v.textContent = `${result.explanation.conclusion} · A=${result.baseline.outcome} B=${result.branch.outcome}`;
        (v as HTMLElement).style.color =
          result.explanation.conclusion === 'DIFFERENT' ? 'var(--warn)'
            : result.explanation.conclusion === 'NO_DIFFERENCE' ? 'var(--success)' : 'var(--text-2)';
      }
      ctx.status(`5D — A/B 重算完了: ${result.explanation.conclusion}`);
      renderHost();
      renderTimeline();
    };

    /* ── ツールバー ─────────────────────────────────────────── */
    const renderToolbar = () => {
      toolbar.innerHTML = `
        <select id="d5-mode" class="tool-btn" style="padding:0 6px" aria-label="5D 因果表示モード">
          <option value="none" ${mode === 'none' ? 'selected' : ''}>3D のみ (none)</option>
          <option value="overlay" ${mode === 'overlay' ? 'selected' : ''}>Overlay3D</option>
          <option value="split" ${mode === 'split' ? 'selected' : ''}>SplitSynchronized3D</option>
        </select>
        <span class="tool-sep"></span>
        <button class="tool-btn" data-act="run">⟳ 重算 A/B</button>
        <button class="tool-btn" data-act="clear">清空干渉</button>
        <span class="tool-sep"></span>
        <span class="tool-label" id="d5-verdict">—</span>
        ${has3D ? '' : '<span class="badge badge--FAIL">DEGRADED (3D 不可)</span>'}`;
    };

    /* ── サイドバー ─────────────────────────────────────────── */
    const renderSidebar = () => {
      sidebar.innerHTML = `
        <div class="panel" style="padding:8px">
          <div class="sidebar-title" style="margin-bottom:6px">干渉操作 (spec 5D.operations)</div>
          <div style="display:flex;flex-direction:column;gap:4px">
            <select id="d5-op" class="tool-btn" style="padding:0 6px" aria-label="干渉種別">
              <option value="DisableNode">DisableNode — 無効化ノード</option>
              <option value="DisableEdge">DisableEdge — 無効化辺</option>
              <option value="BypassNode">BypassNode — 迂回</option>
              <option value="ChangeGuard">ChangeGuard — 護衛変更</option>
              <option value="ChangeTimeout">ChangeTimeout — タイムアウト変更</option>
              <option value="ChangeRetry">ChangeRetry — 再試行変更</option>
              <option value="DelayMessage">DelayMessage — 遅延注入</option>
              <option value="InjectFault">InjectFault — 故障注入</option>
              <option value="AddInteractiveElement">AddInteractiveElement — 挿入要素</option>
              <option value="RewireGraph">RewireGraph — 配線変更</option>
            </select>
            <select id="d5-target" class="tool-btn" style="padding:0 6px" aria-label="干渉対象">${nodeOptions}</select>
            <div id="d5-rewire" style="display:none">
              <select id="d5-rewire-target" class="tool-btn" style="padding:0 6px;width:100%;margin-top:4px" aria-label="改線後の新接続点"></select>
              <div class="empty-hint" style="font-size:10px">改線後の新接続点 (必須。無いと削除のみ)</div>
            </div>
            <input id="d5-value" type="text" placeholder="値 (timeoutms / 再試行回数 / 護衛テキスト)"
                   style="height:28px;background:var(--raised);color:var(--text);border:1px solid var(--edge);border-radius:6px;padding:0 8px;font-size:12px">
            <button class="tool-btn" data-act="add">+ 干渉を追加</button>
          </div>
        </div>
        <div class="panel" style="padding:8px">
          <div class="sidebar-title" style="margin-bottom:6px">適用済み (${interventions.length})</div>
          <div id="d5-list"></div>
          ${interventions.length ? '<button class="tool-btn" data-act="clear" style="width:100%;margin-top:6px">清空</button>' : ''}
        </div>
        <div class="panel" style="padding:8px">
          <div class="sidebar-title" style="margin-bottom:6px">CASE</div>
          <select id="d5-case" class="tool-btn" style="padding:0 6px;width:100%" aria-label="Case 選択">
            ${cases.map((c) => `<option value="${c.caseId}" ${c.caseId === currentCase.caseId ? 'selected' : ''}>${esc(c.kind)} — ${esc(c.title)}</option>`).join('')}
          </select>
        </div>
        <div class="panel" style="padding:8px">
          <div class="sidebar-title" style="margin-bottom:6px">上流</div>
          <p class="empty-hint">4D Trace: <code>${esc(bus.snapshot.traceId ?? '—')}</code></p>
          <p class="empty-hint">3D 焦点: ${bus.snapshot.focus3d?.size ?? 0} ノード</p>
          <p class="empty-hint">1D Prompt: <code>${esc(bus.snapshot.promptId ?? '—')}</code></p>
        </div>`;

      const list = sidebar.querySelector('#d5-list') as HTMLElement;
      list.innerHTML = interventions.length
        ? interventions.map((i) => `<div style="font-size:11px;padding:3px 0;border-bottom:1px solid var(--edge)">
             <code>${esc(i.kind)}</code> → ${esc(i.targetId)}
             ${i.value !== undefined ? `<br><span class="empty-hint">値: ${esc(String(i.value))}</span>` : ''}
           </div>`).join('')
        : '<p class="empty-hint">干渉なし → A/B は必ず NO_DIFFERENCE (対照基準)</p>';

      (sidebar.querySelector('#d5-case') as HTMLSelectElement).addEventListener('change', (e) => {
        const c = cases.find((x) => x.caseId === (e.target as HTMLSelectElement).value);
        if (c) { currentCase = c; runAB(); }
      });

      (sidebar.querySelector('#d5-op') as HTMLSelectElement).addEventListener('change', () => {
        const op = (sidebar.querySelector('#d5-op') as HTMLSelectElement).value as Intervention['kind'];
        const target = sidebar.querySelector('#d5-target') as HTMLSelectElement;
        if (op === 'DisableEdge' || op === 'RewireGraph') {
          target.innerHTML = [...ctx.index.edges.values()]
            .map((e) => `<option value="${esc(e.id)}">${esc(e.id)} (${esc(String(e.properties.callerCrate ?? ''))}→${esc(String(e.properties.calleeCrate ?? ''))})</option>`).join('');
        } else if (op === 'ChangeGuard' || op === 'ChangeTimeout' || op === 'ChangeRetry') {
          target.innerHTML = rules.map((r) => `<option value="${esc(r.ruleId)}">${esc(r.ruleId)} — ${esc(r.name)}</option>`).join('');
        } else {
          target.innerHTML = nodeOptions;
        }
        const rewireBox = sidebar.querySelector('#d5-rewire');
        if (op === 'RewireGraph') {
          rewireBox!.style.display = 'block';
          const sel = sidebar.querySelector('#d5-rewire-target') as HTMLSelectElement;
          if (!sel.options.length) { sel.innerHTML = nodeOptions; rewireTarget = sel.value; }
        } else {
          rewireBox!.style.display = 'none';
        }
      });

      (sidebar.querySelector('#d5-rewire-target') as HTMLSelectElement | null)?.addEventListener('change', (e) => {
        rewireTarget = (e.target as HTMLSelectElement).value;
      });

      sidebar.querySelector('[data-act="add"]')?.addEventListener('click', () => {
        const op = (sidebar.querySelector('#d5-op') as HTMLSelectElement).value as Intervention['kind'];
        const target = (sidebar.querySelector('#d5-target') as HTMLSelectElement).value;
        const raw = (sidebar.querySelector('#d5-value') as HTMLInputElement).value;
        const it: Intervention = {
          interventionId: `INT-${interventions.length + 1}`,
          kind: op, targetId: target, label: `${op}@${target}`,
          origin: 'HypotheticalExperiment',
          value: op.startsWith('Change') ? target : undefined,
          payload: buildPayload(op, target, raw),
        };
        interventions = [...interventions, it];
        renderSidebar();
        runAB();
      });

      sidebar.querySelector('[data-act="clear"]')?.addEventListener('click', () => {
        interventions = []; renderSidebar(); runAB();
      });
    };

    const buildPayload = (op: string, target: string, raw: string): Record<string, unknown> => {
      switch (op) {
        case 'ChangeGuard': return { guard: raw || 'state == Failed' };
        case 'ChangeTimeout': return { timeoutMs: Number(raw) || 1000 };
        case 'ChangeRetry': return { max: Number(raw) || 1 };
        case 'DelayMessage': return { delayMs: Number(raw) || 500 };
        case 'AddInteractiveElement': return { kind: 'Processor', name: '挿入された要素' };
        case 'RewireGraph': {
          const edge = ctx.index.edges.get(target);
          return { edgeId: target, from: edge?.sourceId, to: rewireTarget || undefined };
        }
        default: return {};
      }
    };

    /* ── 中央パネル ─────────────────────────────────────────── */
    const renderHost = () => {
      // 前回描画した情報パネルを消してから差し替える (積み上げ防止)
      host.querySelectorAll('.d5-info').forEach((e) => e.remove());
      const el = document.createElement('div');
      el.className = 'd5-info';
      el.style.cssText = 'position:absolute;left:12px;top:12px;right:12px;z-index:5;overflow:auto;max-height:calc(100% - 24px);';
      if (!result) { el.innerHTML = '<div class="panel"><h3>5D 未実行</h3></div>'; host.appendChild(el); return; }
      const { baseline, branch, explanation } = result;
      const verdictColor = explanation.conclusion === 'DIFFERENT' ? 'var(--warn)'
        : explanation.conclusion === 'NO_DIFFERENCE' ? 'var(--success)' : 'var(--text-2)';

      const diffRows = Object.entries(explanation.terminalDiff).map(([id, d]) => `
        <tr><td>${esc(ctx.index.nodes.get(id)?.name ?? id)}</td><td>${esc(d.baseline)}</td><td>${esc(d.branch)}</td></tr>`).join('');

      const delta = applyInterventions(graph, interventions);

      el.innerHTML = `
        <div class="panel">
          <h3>因果結論
            <span class="badge" style="color:${verdictColor}">${esc(explanation.conclusion)}</span>
            <span class="badge badge--Designed">${esc(mode)}</span>
          </h3>
          ${explanation.conclusion === 'NO_DIFFERENCE'
            ? '<p style="color:var(--success)">干渉が可観測な差を生まなかった → NO_DIFFERENCE。分岐は捏造しない。</p>' : ''}
          ${explanation.conclusion === 'UNKNOWN'
            ? '<p style="color:var(--text-2)">差を説明するルールが 1 つも実行されていない → 因果は不明 (UNKNOWN)。</p>' : ''}
          ${mode === 'split' ? '<p class="empty-hint">左 = ベースライン / 右 = 分岐 (同一ステージを 2 パス描画)。</p>' : ''}
          ${mode === 'overlay' ? '<p class="empty-hint">分岐ノードをリング色で強調。ベースラインは左下の凡例を参照。</p>' : ''}
        </div>

        <div class="panel">
          <h3>初回分岐 (FirstDivergence)</h3>
          ${explanation.firstDivergence
            ? `<dl class="kv">
                 <dt>論理時間</dt><dd>${explanation.firstDivergence.logicalTime}</dd>
                 <dt>Branch A イベント</dt><dd><code>${esc(explanation.firstDivergence.eventA)}</code></dd>
                 <dt>Branch B イベント</dt><dd><code>${esc(explanation.firstDivergence.eventB)}</code></dd>
                 ${explanation.firstDivergenceDetail
              ? `<dt>A 状態遷移</dt><dd><code>${esc(explanation.firstDivergenceDetail.a)}</code></dd>
                     <dt>B 状態遷移</dt><dd><code>${esc(explanation.firstDivergenceDetail.b)}</code></dd>`
              : ''}
               </dl>`
            : '<p class="empty-hint">分岐なし (NO_DIFFERENCE)。</p>'}
        </div>

        <div class="panel">
          <h3>図増分 (ScenarioGraphDelta)</h3>
          <p class="empty-hint">干渉が**有効グラフ**に与えた実際の変更。正式事実グラフは不変。</p>
          <dl class="kv">
            <dt>追加ノード</dt><dd>${delta.addedNodes.length ? esc(delta.addedNodes.map((n) => n.name).join(', ')) : '—'}</dd>
            <dt>除去辺</dt><dd>${delta.removedEdges.length ? esc(delta.removedEdges.join(', ')) : '—'}</dd>
            <dt>改写</dt><dd>${delta.rewired.length
              ? esc(delta.rewired.map((r) => `${r.from} → ${r.newEdgeId} → ${r.to} (旧 ${r.replacedEdgeId})`).join('; '))
              : '—'}</dd>
            <dt>無効化ノード</dt><dd>${delta.disabledNodes.length ? esc(delta.disabledNodes.join(', ')) : '—'}</dd>
            <dt>無効化辺</dt><dd>${delta.disabledEdges.length ? esc(delta.disabledEdges.join(', ')) : '—'}</dd>
          </dl>
        </div>

        <div class="panel">
          <h3>影響サブグラフ (Propagation)</h3>
          <p class="empty-hint">ノード ${explanation.affectedNodeIds.length} · 辺 ${explanation.affectedEdgeIds.length}</p>
          <div style="display:flex;flex-wrap:wrap;gap:4px">
            ${explanation.affectedNodeIds.slice(0, 40).map((id) => {
              const n = ctx.index.nodes.get(id);
              return `<span class="badge" style="color:${DOMAIN_COLOR[n?.domainId ?? 'Unknown']}">${esc(n?.name ?? id)}</span>`;
            }).join('')}
          </div>
        </div>

        <div class="panel">
          <h3>終端差分 (OutcomeDiff)</h3>
          <table class="data">
            <thead><tr><th>ノード</th><th>Branch A</th><th>Branch B</th></tr></thead>
            <tbody>${diffRows || '<tr><td colspan="3">差なし</td></tr>'}</tbody>
          </table>
          ${explanation.structuralOnlyNodeIds?.length
            ? `<p class="empty-hint">片側のみに存在するノード (構造差分): ${esc(explanation.structuralOnlyNodeIds.join(', '))}</p>` : ''}
        </div>

        <div class="panel">
          <h3>復旧 / 補償 (Recovery / Compensation)</h3>
          <dl class="kv">
            <dt>Recovery</dt><dd>${esc(explanation.recovery ?? '—')}</dd>
            <dt>Compensation</dt><dd>${esc(explanation.compensation ?? '—')}</dd>
            <dt>ルール</dt><dd>${esc([...new Set([...baseline.executedRuleIds, ...branch.executedRuleIds])].join(', ') || '—')}</dd>
          </dl>
        </div>

        <div class="panel">
          <h3>仮定と境界 (Assumptions)</h3>
          <ul>${explanation.assumptions.map((a) => `<li>${esc(a)}</li>`).join('')}</ul>
        </div>

        <div class="panel">
          <h3>分岐トレース対照</h3>
          <table class="data">
            <thead><tr><th>項目</th><th>Branch A (基準)</th><th>Branch B (干渉)</th></tr></thead>
            <tbody>
              <tr><td>traceId</td><td><code>${esc(baseline.traceId)}</code></td><td><code>${esc(branch.traceId)}</code></td></tr>
              <tr><td>outcome</td><td>${esc(baseline.outcome)}</td><td>${esc(branch.outcome)}</td></tr>
              <tr><td>イベント数</td><td>${baseline.events.length}</td><td>${branch.events.length}</td></tr>
              <tr><td>截断</td><td>${baseline.truncated ? 'YES' : 'no'}</td><td>${branch.truncated ? 'YES' : 'no'}</td></tr>
              <tr><td>干渉</td><td>(なし)</td><td>${esc(branch.appliedInterventions.join(', ') || '(なし)')}</td></tr>
            </tbody>
          </table>
        </div>

        <div class="panel">
          <h3>診断 (Diagnostics)</h3>
          <ul>${[...baseline.diagnostics, ...branch.diagnostics].map((d) => `<li>${esc(d)}</li>`).join('') || '<li>なし</li>'}</ul>
        </div>`;
      host.appendChild(el);
    };

    /* ── タイムライン: A/B 並置 ─────────────────────────────── */
    const renderTimeline = () => {
      const el = document.getElementById('timeline')!;
      const s = bus.snapshot;
      const rows = (evs: typeof s.events, label: string) =>
        evs.map((e) => `
          <button class="tl-event" data-seq="${e.sequence}" aria-current="${e.sequence === s.playhead + 1}"
            title="${esc(e.ruleId ?? '')}">
            <span class="tl-seq">${label}${e.sequence}</span>
            <span class="tl-kind" style="color:${RING_STATE_COLOR[e.nextState ?? 'Idle']}">${esc(e.nextState ?? '')}</span>
            <span>${esc(ctx.index.nodes.get(e.sourceNodeId ?? '')?.name ?? e.sourceNodeId ?? '')}</span>
          </button>`).join('');
      el.innerHTML = `
        <div class="timeline-head">
          <span class="tool-label">A/B 分岐タイムライン</span>
          <span class="tool-label">A: ${esc(s.traceId ?? '—')}</span>
          <span class="tool-label">B: ${esc(s.branchTraceId ?? '—')}</span>
        </div>
        <div class="timeline-track">
          <div style="margin-bottom:4px"><strong style="font-size:11px">Branch A</strong> ${rows(s.events, 'A')}</div>
          <div><strong style="font-size:11px">Branch B</strong> ${rows(s.branchEvents, 'B')}</div>
        </div>`;
      el.querySelectorAll('[data-seq]').forEach((b) => {
        b.addEventListener('click', () => bus.seek(Number((b as HTMLElement).dataset.seq) - 1));
      });
    };

        /* ツールバー描画は**イベント配線より前**に行う。
       先に innerHTML を書いてから querySelector しないと null になる。 */
    renderToolbar();
    renderSidebar();

/* ── 制御 ───────────────────────────────────────────────── */
    (toolbar.querySelector('#d5-mode') as HTMLSelectElement).addEventListener('change', (e) => {
      mode = (e.target as HTMLSelectElement).value as CausalMode;
      bus.setCausalMode(mode);
      runAB();
    });
    toolbar.querySelector('[data-act="run"]')?.addEventListener('click', () => { runAB(); renderSidebar(); });
    toolbar.querySelector('[data-act="clear"]')?.addEventListener('click', () => {
      interventions = []; renderSidebar(); runAB();
    });

    const unsubscribe = bus.subscribe(() => { renderTimeline(); });

    runAB();

    return () => {
      unsubscribe();
      bus.pause();
      stage.detach();
    };
  },
};
