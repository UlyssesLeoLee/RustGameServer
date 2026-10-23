/**
 * d4.ts — 4D 動的 Case
 *
 * spec `immutable_dimensions.4D`:
 *   "在**同一个 3D** Graph/Scene 上，以逻辑时间驱动 正常/异常/边界/并发/补偿/恢复 Case"
 *
 * したがって 4D は独自の 3D を持たない。**3D と同じステージ** (src/core/scene-host.ts) に
 * ノード状態を適用し、下部のタイムラインで Event Trace を再生する。
 * アニメーションは Trace の再生であり、FPS は結果を決めない (spec 4D.engine)。
 *
 * 鎖:
 *   3D の焦点 (bus.focus3d) → 4D の対象 Case 選定 → エンジン実行 → Trace を bus へ公開
 *   → 5D が A/B 比較に使う
 */

import type { DimensionModule, DimensionContext } from '../core/app';
import { esc } from '../core/app';
import { bus } from '../core/bus';
import { getSceneHost } from '../core/scene-host';
import { RING_STATE_COLOR } from '../core/tokens';
import { runSimulation, type ScenarioGraph, type SimNode, type SimEdge, type RunResult } from '../sim/engine';
import type { CaseRecord, CausalRule } from '../core/types';

interface Models { cases?: CaseRecord[]; rules?: CausalRule[] }
const models = (): Models => (window as unknown as { __ATLAS_MODELS?: Models }).__ATLAS_MODELS ?? {};

export const d4: DimensionModule = {
  id: '4D',
  title: '4D 動的 Case',
  usesTimeline: true,
  mount(ctx: DimensionContext) {
    const { host, toolbar, sidebar } = ctx;
    const cases = models().cases ?? [];
    const rules = models().rules ?? [];

    if (!cases.length) {
      host.innerHTML = '<div class="panel"><h3>4D 無 Case データ</h3><p class="empty-hint">状態: NOT_TESTED</p></div>';
      return;
    }

    /* ── 共有ステージを 4D に取り付ける (3D と同一の canvas) ── */
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

    /* 4D 独自の状態投影 (WebGL 不可時のフォールバック) */
    const fallback = document.createElement('div');
    fallback.style.cssText = 'position:absolute;inset:0;display:none;';
    host.appendChild(fallback);

    /* ── グラフ (FactGraph) ─────────────────────────────────── */
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

    let currentCase = cases[0];
    let lastRun: RunResult | null = null;
    let unsubscribe = () => { };

    /* ── 3D の焦点から Case を選ぶ (3D → 4D の鎖) ───────────── */
    const pickCaseFromFocus = (): CaseRecord | null => {
      const focus = bus.snapshot.focus3d;
      if (!focus || !focus.size) return null;
      const hit = cases.find((c) => c.nodeIds.some((n) => focus.has(n)));
      return hit ?? null;
    };

    const run = () => {
      const r = runSimulation({
        caseId: currentCase.caseId,
        caseTitle: currentCase.title,
        steps: currentCase.steps,
        rules,
        interventions: [],
        graph,
        seed: 20261021,
        modelRevision: ctx.index.model.meta.revision,
      });
      lastRun = r;
      bus.publishTrace({
        traceId: r.traceId, caseId: r.caseId, caseTitle: r.caseTitle,
        events: r.events, finalStates: r.finalStates,
      });
      ctx.status(`4D — ${r.caseId} 実行完了: ${r.outcome}, ${r.events.length} イベント${r.truncated ? ' (截断)' : ''}`);
      renderHost();
      renderTimeline();
    };

    /* ── 状態適用 (3D ステージ or フォールバック SVG) ─────────── */
    const applyStates = (states: Record<string, string>) => {
      if (has3D) { stage.applyStates(states); return; }
      // フォールバック: 2D 状態投影
      const s = bus.snapshot;
      const byDomain: Record<string, string[]> = {};
      for (const [id, st] of Object.entries(states)) {
        const n = ctx.index.nodes.get(id);
        const d = n?.domainId ?? 'Unknown';
        (byDomain[d] ??= []).push(id);
      }
      let svg = `<svg viewBox="0 0 ${Math.max(600, Object.keys(byDomain).length * 190)} 200" width="100%" height="200" role="img" aria-label="4D 状態投影 (DEGRADED)">`;
      Object.entries(byDomain).forEach(([d, ids], di) => {
        const x = 30 + di * 190;
        svg += `<text x="${x}" y="20" fill="var(--text-2)" font-size="11">${esc(d)}</text>`;
        ids.forEach((id, ni) => {
          const n = ctx.index.nodes.get(id);
          const st = states[id];
          svg += `<g data-node="${esc(id)}" style="cursor:pointer">
            <rect x="${x}" y="${30 + ni * 50}" width="170" height="42" rx="8" fill="var(--surface)" stroke="${RING_STATE_COLOR[st] ?? '#81909F'}" stroke-width="1.5"/>
            <text x="${x + 10}" y="${50 + ni * 50}" fill="var(--text)" font-size="12">${esc(n?.name ?? id)}</text>
            <text x="${x + 10}" y="${64 + ni * 50}" fill="var(--text-2)" font-size="10">${esc(st)}</text></g>`;
        });
      });
      svg += '</svg>';
      fallback.innerHTML = svg;
      fallback.style.display = 'block';
      mountEl.style.display = 'none';
      fallback.querySelectorAll('[data-node]').forEach((g) => {
        g.addEventListener('click', () => ctx.inspect({ __kind: 'node', id: (g as HTMLElement).dataset.node! }));
      });
    };

    /* ── ツールバー ─────────────────────────────────────────── */
    const renderToolbar = () => {
      toolbar.innerHTML = `
        <button class="tool-btn" data-act="play">▶ Play</button>
        <button class="tool-btn" data-act="pause">⏸ Pause</button>
        <button class="tool-btn" data-act="stepF" title="StepForward">⏭ 前</button>
        <button class="tool-btn" data-act="stepB" title="StepBack">⏮ 後</button>
        <button class="tool-btn" data-act="reset">⟲ Reset</button>
        <button class="tool-btn" data-act="replay">↺ Replay</button>
        <button class="tool-btn" data-act="snapshot">⛶ Snapshot</button>
        <span class="tool-sep"></span>
        <label class="tool-label" for="d4-speed">Speed</label>
        <select id="d4-speed" class="tool-btn" style="padding:0 6px">
          ${[0.25, 0.5, 1, 2, 4].map((s) => `<option value="${s}" ${s === bus.snapshot.speed ? 'selected' : ''}>${s}x</option>`).join('')}
        </select>
        <span class="tool-sep"></span>
        <span class="tool-label" id="d4-seek-label">seek 0/0</span>
        ${has3D ? '' : '<span class="badge badge--FAIL">DEGRADED (2D 投影)</span>'}`;
    };

    /* ── サイドバー ─────────────────────────────────────────── */
    const renderSidebar = () => {
      const upstream = bus.snapshot;
      sidebar.innerHTML = `
        ${upstream.focus3d?.size ? `<div class="panel" style="padding:8px">
          <div class="sidebar-title" style="margin-bottom:6px">上流: 3D 焦点</div>
          <p class="empty-hint">${upstream.focus3d.size} ノードに焦点中。対象 Case を自動選定した。</p>
        </div>` : `<div class="panel" style="padding:8px">
          <div class="sidebar-title" style="margin-bottom:6px">上流: 3D 焦点</div>
          <p class="empty-hint">3D でノードを選択すると、その Shot が対象 Case として自動選択される。</p>
        </div>`}
        <div class="panel" style="padding:8px">
          <div class="sidebar-title" style="margin-bottom:6px">CASE (${cases.length})</div>
          ${cases.map((c) => `
            <button class="tool-btn" data-case="${c.caseId}"
              style="display:block;width:100%;text-align:left;margin-bottom:3px;height:auto;min-height:30px;padding:5px 8px;white-space:normal;line-height:1.35;
                     ${c.caseId === currentCase.caseId ? 'border-color:var(--focus);' : ''}">
              <span class="badge badge--${c.origin === 'HypotheticalExperiment' ? 'Inferred' : 'Designed'}">${esc(c.kind)}</span>
              <code style="font-size:10px">${esc(c.caseId)}</code><br>
              <span style="font-size:11px;color:var(--text-2)">${esc(c.title)}</span>
            </button>`).join('')}
        </div>
        <div class="panel" style="padding:8px">
          <div class="sidebar-title" style="margin-bottom:6px">エンジン</div>
          <p class="empty-hint">決定論的離散イベントエンジン (src/sim/engine.ts)。Three.js から独立。
            描画 FPS は結果を決めない。</p>
          <p class="empty-hint">seed=20261021 / ${esc(ctx.index.model.meta.shortRevision)}</p>
        </div>`;
      sidebar.querySelectorAll('[data-case]').forEach((b) => {
        b.addEventListener('click', () => {
          const id = (b as HTMLElement).dataset.case!;
          const c = cases.find((x) => x.caseId === id);
          if (c) { currentCase = c; run(); renderSidebar(); }
        });
      });
    };

    /* ── 中央パネル ─────────────────────────────────────────── */
    const renderHost = () => {
      // 前回描画した情報パネルを消してから差し替える (積み上げ防止)
      host.querySelectorAll('.d4-info').forEach((e) => e.remove());
      const c = currentCase;
      const info = document.createElement('div');
      info.className = 'panel';
      info.style.cssText = 'position:absolute;left:12px;top:12px;right:12px;z-index:5;max-width:760px;';
      info.innerHTML = `
        <h3>${esc(c.title)}
          <span class="badge badge--${c.origin === 'HypotheticalExperiment' ? 'Inferred' : 'Designed'}">${esc(c.origin)}</span>
          ${lastRun?.truncated ? '<span class="badge badge--FAIL">TRUNCATED</span>' : ''}
        </h3>
        <p class="empty-hint">${esc(c.description)}</p>
        <p class="empty-hint"><strong>预期</strong>: ${esc(c.expectedOutcome)}
          ${c.origin === 'HypotheticalExperiment' ? '<br>⚠️ 本 Case 明确标记为假设性实验, 不得当作已实现行为。' : ''}</p>
        ${has3D ? '<p class="empty-hint">▶ この Case は <strong>3D と同じステージ</strong>上で再生される (4D は 3D の上に立つ)。</p>' : ''}`;
      host.appendChild(info);
    };

    /* ── タイムライン ───────────────────────────────────────── */
    const renderTimeline = () => {
      const el = document.getElementById('timeline')!;
      const s = bus.snapshot;
      el.innerHTML = `
        <div class="timeline-head">
          <span class="tool-label">Trace</span>
          <code style="font-size:10px">${esc(s.traceId ?? '—')}</code>
          <span class="tool-label">seed ${lastRun?.seed ?? '—'}</span>
          <span class="tool-label">outcome <strong>${esc(lastRun?.outcome ?? '—')}</strong></span>
          <span class="tool-label">events ${s.events.length}</span>
          ${lastRun?.truncated ? '<span class="badge badge--FAIL">安全限额截断</span>' : ''}
        </div>
        <div class="timeline-track">
          ${s.events.map((e) => `
            <button class="tl-event" data-seq="${e.sequence}"
              aria-current="${e.sequence === s.playhead + 1}"
              title="${esc(e.ruleId ?? '')}">
              <span class="tl-seq">${e.sequence}</span>
              <span class="tl-kind" style="color:${RING_STATE_COLOR[e.nextState ?? 'Idle']}">${esc(e.nextState ?? '')}</span>
              <span>${esc(ctx.index.nodes.get(e.sourceNodeId ?? '')?.name ?? e.sourceNodeId ?? '')}</span>
            </button>`).join('')}
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
    toolbar.querySelector('[data-act="play"]')?.addEventListener('click', () => bus.play());
    toolbar.querySelector('[data-act="pause"]')?.addEventListener('click', () => bus.pause());
    toolbar.querySelector('[data-act="stepF"]')?.addEventListener('click', () => { bus.pause(); bus.step(1); });
    toolbar.querySelector('[data-act="stepB"]')?.addEventListener('click', () => { bus.pause(); bus.step(-1); });
    toolbar.querySelector('[data-act="reset"]')?.addEventListener('click', () => bus.reset());
    toolbar.querySelector('[data-act="replay"]')?.addEventListener('click', () => { bus.reset(); bus.play(); });
    toolbar.querySelector('[data-act="snapshot"]')?.addEventListener('click', () => {
      const s = bus.snapshot;
      const payload = {
        traceId: s.traceId, caseId: s.caseId, seed: lastRun?.seed,
        modelRevision: lastRun?.modelRevision, playhead: s.playhead,
        states: s.states, capturedAt: new Date().toISOString(),
      };
      const blob = new Blob([JSON.stringify(payload, null, 2)], { type: 'application/json' });
      const a = document.createElement('a');
      a.href = URL.createObjectURL(blob);
      a.download = `snapshot-${s.caseId}-${s.playhead}.json`;
      a.click();
      setTimeout(() => URL.revokeObjectURL(a.href), 1000);
      ctx.status(`已导出快照 (playhead=${s.playhead})`);
    });
    (toolbar.querySelector('#d4-speed') as HTMLSelectElement).addEventListener('change', (e) => {
      bus.setSpeed(Number((e.target as HTMLSelectElement).value));
    });

    /* ── bus 購読: 再生位置 → ステージ状態適用 ───────────────── */
    unsubscribe = bus.subscribe((s) => {
      applyStates(s.states);
      const sl = toolbar.querySelector('#d4-seek-label');
      if (sl) sl.textContent = `seek ${s.playhead + 1}/${s.events.length}`;
      renderTimeline();
    });

    /* 3D の焦点が変わったら対象 Case を切り替える (3D → 4D の鎖) */
    const unsubFocus = bus.subscribe(() => {
      const hit = pickCaseFromFocus();
      if (hit && hit.caseId !== currentCase.caseId) {
        currentCase = hit;
        run();
        renderSidebar();
        ctx.status(`3D の焦点を受け ${hit.caseId} に切替`);
      }
    });

    run();

    return () => {
      unsubscribe();
      unsubFocus();
      bus.pause();
      stage.detach();
    };
  },
};
