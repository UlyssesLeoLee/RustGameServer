/**
 * d3.ts — 3D 次元 (ステージのビュー)
 *
 * シーンそのものは `src/core/scene-host.ts` が**プロセス内で 1 個だけ**保持し、
 * 3D / 4D / 5D が同じステージを共有する。
 * このファイルは 12 View・12 ナビゲーション・ラベルモード・ラベル/ピッキングの UI に徹する。
 *
 * 上流 (2D) からの連携: 2D で選択したノードは bus.selection2d に入り、
 * 这里是 3D でそのノードにフォーカスする (2D → 3D の鎖)。
 */

import type { DimensionModule, DimensionContext } from '../core/app';
import { esc } from '../core/app';
import { bus } from '../core/bus';
import { getSceneHost } from '../core/scene-host';
import { VIEWS, viewById, type ViewId } from '../three/views';
import { DOMAIN_COLOR } from '../core/tokens';

interface Models {
  spatial?: { building: { floors: { id: string; name: string; y: number }[] } };
  rules?: never[];
  cases?: never[];
}
const models = (): Models => (window as unknown as { __ATLAS_MODELS?: Models }).__ATLAS_MODELS ?? {};

export const d3: DimensionModule = {
  id: '3D',
  title: '3D 空間グラフ',
  mount(ctx: DimensionContext) {
    const { host, toolbar, sidebar } = ctx;

    const stage = getSceneHost(
      ctx.index.model, ctx.index as never,
      models().spatial as never,
      (window as unknown as { __ATLAS_MODELS?: { rules?: never[]; cases?: never[] } }).__ATLAS_MODELS?.rules ?? [],
      (window as unknown as { __ATLAS_MODELS?: { rules?: never[]; cases?: never[] } }).__ATLAS_MODELS?.cases ?? [],
    );

    /* ステージをこの次元のホストに取り付ける (canvas 自体は破棄しない) */
    const mountEl = document.createElement('div');
    mountEl.style.cssText = 'position:absolute;inset:0;';
    host.appendChild(mountEl);
    if (!stage.attachTo(mountEl)) {
      host.innerHTML = `<div class="panel" style="margin:16px">
        <h3><span class="badge badge--FAIL">DEGRADED</span> 3D を描画できません</h3>
        <p class="empty-hint">WebGL が利用できません。CSS 3D 透視や画像で 3D を偽装しない。</p></div>`;
      return () => { /* noop */ };
    }

    let view: ViewId = stage.view;
    let unsubscribe = () => { };

    /* ── ツールバー ─────────────────────────────────────────── */
    const renderToolbar = () => {
      toolbar.innerHTML = `
        <select id="d3-view" class="tool-btn" style="padding:0 6px" aria-label="3D View">
          ${VIEWS.map((v) => `<option value="${v.id}" ${v.id === view ? 'selected' : ''}>${esc(v.label)}</option>`).join('')}
        </select>
        <select id="d3-label" class="tool-btn" style="padding:0 6px" aria-label="ラベルモード">
          <option value="Smart" ${stage.labelMode === 'Smart' ? 'selected' : ''}>Smart</option>
          <option value="All" ${stage.labelMode === 'All' ? 'selected' : ''}>All</option>
          <option value="RelatedOnly" ${stage.labelMode === 'RelatedOnly' ? 'selected' : ''}>RelatedOnly</option>
        </select>
        <span class="tool-sep"></span>
        <select id="d3-floor" class="tool-btn" style="padding:0 6px" aria-label="フロア切替">
          <option value="">Floor: (全)</option>
          ${stage.atlas.floors.map((f) => `<option value="${esc(f.id)}" ${f.id === bus.snapshot.activeFloorId ? 'selected' : ''}>${esc(f.name)}</option>`).join('')}
        </select>
        <select id="d3-room" class="tool-btn" style="padding:0 6px" aria-label="ルーム切替">
          <option value="">Room: (全)</option>
          ${stage.atlas.rooms.map((r) => `<option value="${esc(r.id)}" ${r.id === bus.snapshot.activeRoomId ? 'selected' : ''}>${esc(r.name)}</option>`).join('')}
        </select>
        <span class="tool-sep"></span>
        <button class="tool-btn" data-act="reset">⟲ Reset</button>
        <button class="tool-btn" data-act="fit">⤢ Fit</button>
        <button class="tool-btn" data-act="gizmo" aria-pressed="${stage.isGizmoVisible()}">軸</button>
        <span class="tool-label" id="d3-info">${stage.atlas.nodes.size} nodes · ${stage.atlas.edges.length} edges · ${stage.atlas.rooms.length} rooms</span>`;
    };

    /* ── サイドバー ─────────────────────────────────────────── */
    const renderSidebar = () => {
      const sel = stage.selected;
      const node = sel ? stage.atlas.nodes.get(sel.id) : undefined;
      const upstream = bus.snapshot;
      sidebar.innerHTML = `
        <div class="panel" style="padding:8px">
          <div class="sidebar-title" style="margin-bottom:6px">投影の性質</div>
          <p class="empty-hint">⚠️ この空間は<strong>論理投影</strong>であり、物理的な机房トポロジではない。
            実際の k8s Deployment 情報は各ノードの deployment 属性に保持される (未定義なら Unknown)。</p>
        </div>
        ${upstream.promptId ? `<div class="panel" style="padding:8px">
          <div class="sidebar-title" style="margin-bottom:6px">上流: 1D Prompt</div>
          <p class="empty-hint"><code>${esc(upstream.promptId)}</code> ${esc(upstream.promptTitle ?? '')}</p>
          <p class="empty-hint">スコープ ${upstream.promptScopeNodeIds.size} ノード / ${upstream.promptScopeEdgeIds.size} エッジ</p>
          <button class="tool-btn" data-act="apply-prompt">1D スコープを 3D に適用</button>
        </div>` : ''}
        ${upstream.selection2d.id ? `<div class="panel" style="padding:8px">
          <div class="sidebar-title" style="margin-bottom:6px">上流: 2D 選択</div>
          <p class="empty-hint"><code>${esc(upstream.selection2d.id)}</code> (${esc(upstream.selection2d.kind ?? '')})</p>
        </div>` : ''}
        <div class="panel" style="padding:8px">
          <div class="sidebar-title" style="margin-bottom:6px">宝石ロール (shape = 機能ロール)</div>
          ${(['Actor', 'Input', 'Processor', 'Decision', 'Storage', 'Message', 'Interface', 'Security', 'Agent', 'Composition', 'Output'] as const)
            .map((r) => `<div style="font-size:11px;padding:1px 0">${esc(r)}</div>`).join('')}
          <p class="empty-hint" style="margin-top:6px">color = 業務ドメイン / ring = 実行状態 / facet = ロール派生</p>
        </div>
        <div class="panel" style="padding:8px">
          <div class="sidebar-title" style="margin-bottom:6px">ドメイン色</div>
          ${Object.entries(DOMAIN_COLOR).map(([k, v]) =>
            `<div style="display:flex;align-items:center;gap:6px;font-size:11px;padding:1px 0">
               <span style="width:10px;height:10px;border-radius:2px;background:${v};display:inline-block"></span>${esc(k)}</div>`).join('')}
        </div>
        <div class="panel" style="padding:8px">
          <div class="sidebar-title" style="margin-bottom:6px">線種</div>
          <div style="font-size:11px">実線 = Implemented (実使用点あり)</div>
          <div style="font-size:11px">破線 = Inferred (注释/文字列内のみ)</div>
        </div>
        ${node ? `<div class="panel" style="padding:8px">
          <div class="sidebar-title" style="margin-bottom:6px">選択</div>
          <dl class="kv">
            <dt>名前</dt><dd>${esc(node.display)}</dd>
            <dt>ロール</dt><dd>${esc(node.role)}</dd>
            <dt>ドメイン</dt><dd>${esc(node.node.domainId)}</dd>
            <dt>リング</dt><dd>${esc(node.ring)}</dd>
            <dt>指標</dt><dd>${esc(node.metricLabel)} = ${esc(node.metricValue)}</dd>
            <dt>配備</dt><dd>${node.deployment ? esc(String(node.deployment.manifest ?? '')) : 'Unknown'}</dd>
          </dl>
        </div>` : ''}`;
    };

        /* ツールバー描画は**イベント配線より前**に行う。
       先に innerHTML を書いてから querySelector しないと null になる。 */
    renderToolbar();
    renderSidebar();

/* ── イベント配線 ───────────────────────────────────────── */
    (toolbar.querySelector('#d3-view') as HTMLSelectElement).addEventListener('change', (e) => {
      view = (e.target as HTMLSelectElement).value as ViewId;
      stage.setView(view);
      ctx.status(`3D View: ${viewById(view).label} — ${viewById(view).desc}`);
    });
    (toolbar.querySelector('#d3-label') as HTMLSelectElement).addEventListener('change', (e) => {
      stage.setLabelMode((e.target as HTMLSelectElement).value as 'Smart' | 'All' | 'RelatedOnly');
    });
    (toolbar.querySelector('#d3-floor') as HTMLSelectElement).addEventListener('change', (e) => {
      const v = (e.target as HTMLSelectElement).value || null;
      bus.setActiveFloor(v);
      stage.applyViewVisibility(); stage.requestRender();
    });
    (toolbar.querySelector('#d3-room') as HTMLSelectElement).addEventListener('change', (e) => {
      const v = (e.target as HTMLSelectElement).value || null;
      bus.setActiveRoom(v);
      stage.applyViewVisibility(); stage.requestRender();
    });
    toolbar.querySelector('[data-act="reset"]')?.addEventListener('click', () => {
      stage.resetCamera();
      bus.setActiveFloor(null); bus.setActiveRoom(null);
      renderToolbar(); renderSidebar();
    });
    toolbar.querySelector('[data-act="fit"]')?.addEventListener('click', () => stage.resetCamera());
    toolbar.querySelector('[data-act="gizmo"]')?.addEventListener('click', (e) => {
      const on = stage.toggleGizmo();
      (e.currentTarget as HTMLElement).setAttribute('aria-pressed', String(on));
    });
    sidebar.querySelector('[data-act="apply-prompt"]')?.addEventListener('click', () => {
      const scope = bus.snapshot.promptScopeNodeIds;
      if (!scope.size) { ctx.status('1D Prompt にスコープが未設定'); return; }
      stage.applyFocus(new Set(scope));
      bus.setFocus3d(new Set(scope));
      stage.applyViewVisibility(); stage.requestRender();
      ctx.status(`1D スコープ ${scope.size} ノードを 3D に適用`);
    });

    /* ポインタ操作 (回転 / パン / ズーム / ピッキング) */
    const canvas = stage.canvasElement();
    if (canvas) {
      let dragging = false, lastX = 0, lastY = 0, moved = 0;
      canvas.addEventListener('pointerdown', (ev) => {
        dragging = true; moved = 0; lastX = ev.clientX; lastY = ev.clientY;
        canvas.setPointerCapture(ev.pointerId);
      });
      canvas.addEventListener('pointermove', (ev) => {
        if (!dragging) return;
        const dx = ev.clientX - lastX, dy = ev.clientY - lastY;
        moved += Math.abs(dx) + Math.abs(dy);
        lastX = ev.clientX; lastY = ev.clientY;
        if (ev.shiftKey) stage.pan(dx, dy); else stage.orbit(-dx * 0.006, dy * 0.006);
      });
      canvas.addEventListener('pointerup', (ev) => {
        dragging = false;
        try { canvas.releasePointerCapture(ev.pointerId); } catch { /* noop */ }
        if (moved > 4) return;
        const hit = stage.pick(ev.clientX, ev.clientY);
        if (!hit) {
          stage.selected = null; stage.applyFocus(null);
          bus.setSelection2d({ kind: null, id: null, origin: '3D' });
          bus.setFocus3d(null);
          renderSidebar();
          return;
        }
        stage.selected = { kind: hit.kind, id: hit.id };
        if (hit.kind === 'node') {
          ctx.inspect({ __kind: 'node', id: hit.id });
          stage.focusNode(hit.id);
        } else if (hit.kind === 'edge') {
          ctx.inspect({ __kind: 'edge', id: hit.id });
        } else if (hit.kind === 'port') {
          ctx.inspect({ __kind: 'port', id: hit.id });
        }
        // 3D → 4D へ焦点を送る
        if (hit.kind === 'node') {
          const n = stage.atlas.nodes.get(hit.id);
          const related = new Set<string>([hit.id]);
          if (n) for (const e of stage.atlas.edges) {
            if (e.sourceId === hit.id) related.add(e.targetId);
            if (e.targetId === hit.id) related.add(e.sourceId);
          }
          stage.applyFocus(related);
          bus.setFocus3d(related);
        }
        renderSidebar();
      });
      canvas.addEventListener('wheel', (ev) => {
        ev.preventDefault();
        stage.zoom(ev.deltaY > 0 ? 1.12 : 0.89);
      }, { passive: false });
    }

    const onKey = (ev: KeyboardEvent) => {
      const t = ev.target as HTMLElement | null;
      if (t && ['INPUT', 'TEXTAREA', 'SELECT'].includes(t.tagName)) return;
      let handled = true;
      switch (ev.key) {
        case 'ArrowLeft': stage.orbit(-0.14, 0); break;
        case 'ArrowRight': stage.orbit(0.14, 0); break;
        case 'ArrowUp': stage.orbit(0, 0.12); break;
        case 'ArrowDown': stage.orbit(0, -0.12); break;
        case '+': case '=': stage.zoom(0.86); break;
        case '-': case '_': stage.zoom(1.16); break;
        case 'Escape': stage.selected = null; stage.applyFocus(null); bus.setFocus3d(null); renderSidebar(); break;
        default: handled = false;
      }
      if (handled) ev.preventDefault();
    };
    document.addEventListener('keydown', onKey);

    /* 上流 (2D) からの選択に追従 */
    unsubscribe = bus.subscribe((s) => {
      const id = s.selection2d.id;
      if (id && s.selection2d.kind === 'node' && stage.selected?.id !== id) {
        stage.selected = { kind: 'node', id };
        const related = new Set<string>([id]);
        for (const e of stage.atlas.edges) {
          if (e.sourceId === id) related.add(e.targetId);
          if (e.targetId === id) related.add(e.sourceId);
        }
        stage.applyFocus(related);
        stage.focusNode(id);
        renderSidebar();
      }
    });

    stage.setView(view);
    ctx.status(`3D — ${stage.atlas.nodes.size} ノード / ${stage.atlas.edges.length} エッジ / ${stage.atlas.rooms.length} ルーム (論理投影)`);

    return () => {
      unsubscribe();
      document.removeEventListener('keydown', onKey);
      stage.detach();
    };
  },
};
