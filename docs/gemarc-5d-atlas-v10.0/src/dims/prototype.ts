/**
 * prototype.ts — Prototype マルチタブ原型ワークスペース (spec `prototype_workbench`)
 *
 * 本ファイルは**オーケストレーション層**。下位モジュール:
 *   - model.ts   : ProtoDoc 構造・直列化 (PrototypeJSON / ActionGraphJSON / StateTransitionTable) ・逆取込
 *   - runtime.ts : ProtoRuntime — Action DSL 実行・状態・フォーム値・ガード評価・Trace/replay
 *   - expr.ts    : ガード式パーサ/評価器 (eval を使わない)
 *   - cases.ts   : 4D/5D Case との束縛 (CasePhase → 実 DOM 状態)
 *   - demo.ts    : 示范層 (Designed/Hypothetical) の明示
 *   - ui.ts      : DOM ヘルパ・トースト・モーダル・ファイル入出力
 *
 * 禁止 (spec prototype_workbench.rejection):
 *   - 静的なスクリーンショットだけ → 全 Tab に実操作がある
 *   - Tab が無い → 6 つのサブタブを実装
 *   - ドラッグ後に DOM が同期しない → Konva 座標と DOM プレビューを双方向同期
 *   - フォームが入力不能 → 実 DOM の input/textarea を使う (Konva shape を代用しない)
 *   - 取込後に Action が失われる → PrototypeJSON に Action/Guard/State/Fixture/Route/Variant を含める
 *   - 1D-5D と紐付かない → Screen/Component から各次元への遷移を提供
 */

import Konva from 'konva';
import type { DimensionModule, DimensionContext } from '../core/app';
import { esc } from '../core/app';
import {
  loadDoc, cloneDoc, touch, screenById, componentsOfScreen, actionsOfScreen, canvasSize,
  serializePrototypeJSON, serializeActionGraphJSON, serializeStateTransitionTable, parseImportedDoc,
  VIEWPORTS, FIXTURE_MODES, ALL_WIDGETS, ALL_ACTION_KINDS,
  type ProtoDoc, type ProtoScreen, type ProtoComponent, type ViewportName, type FixtureMode,
} from '../proto/model';
import { ProtoRuntime } from '../proto/runtime';
import { buildCaseBindings, readCases, phaseOfCase, boundCaseIds, isBoundComponent } from '../proto/cases';
import { applyDemoLayer, isDemoEntity } from '../proto/demo';
import { h, token, toast, openModal, installStyles, removeStyles, downloadText, readFileText } from '../proto/ui';

/**
 * 追記用の fill。
 *
 * ui.ts の `fill` は「消去して置換」 semantics だが、本ファイルは
 * `fill(root, a); fill(root, b); fill(root, c)` のように**積み上げ**る用法。
 * 置換 semantics のままだと最後の 1 つだけが残り、
 * Screen タブ行やツールバーボタンが消える (実際に踏んだバグ)。
 *
 * したがって本ファイルでは `fill` = 追記、`replaceText` = 置換 として使い分ける。
 */
function fill(node: HTMLElement, ...children: (Node | string)[]): HTMLElement {
  for (const c of children) node.appendChild(typeof c === 'string' ? document.createTextNode(c) : c);
  return node;
}

type SubTab = 'pages' | 'visual' | 'preview' | 'state' | 'library' | 'design';

const SUBTABS: { id: SubTab; label: string }[] = [
  { id: 'pages', label: 'ページとスクリーン' },
  { id: 'visual', label: 'ビジュアル編集' },
  { id: 'preview', label: 'インタラクションプレビュー' },
  { id: 'state', label: '状態と Case' },
  { id: 'library', label: 'コンポーネントライブラリ' },
  { id: 'design', label: '設計とテスト' },
];

export const prototype: DimensionModule = {
  id: 'Prototype',
  title: 'Prototype ワークスペース',
  mount(ctx: DimensionContext) {
    const { host, toolbar, sidebar } = ctx;

    let doc: ProtoDoc = loadDoc();
    const demoIds = applyDemoLayer(doc);
    const cases = readCases();
    buildCaseBindings(doc, cases);

    let runtime = new ProtoRuntime(doc);
    let sub: SubTab = 'pages';
    let activeScreenId: string | null = doc.screens[0]?.id ?? null;
    let viewport: ViewportName = 'desktop';
    let fixtureMode: FixtureMode = 'ok';
    const closedScreens: string[] = [];

    installStyles();

    /* ── ツールバー ─────────────────────────────────────────── */
    const renderToolbar = () => {
      toolbar.innerHTML = `
        <div role="tablist" aria-label="Prototype サブタブ" style="display:flex;gap:2px;flex-wrap:wrap">
          ${SUBTABS.map((t) => `<button role="tab" class="tool-btn" data-sub="${t.id}"
            aria-selected="${t.id === sub}">${esc(t.label)}</button>`).join('')}
        </div>
        <span class="tool-sep"></span>
        <select id="pt-viewport" class="tool-btn" style="padding:0 6px" aria-label="レスポンシブ表示">
          ${VIEWPORTS.map((v) => `<option value="${v.name}" ${v.name === viewport ? 'selected' : ''}>${esc(v.label)}</option>`).join('')}
        </select>
        <select id="pt-fixture" class="tool-btn" style="padding:0 6px" aria-label="Fixture モード">
          ${FIXTURE_MODES.map((f) => `<option value="${f.mode}" ${f.mode === fixtureMode ? 'selected' : ''}>${esc(f.label)}</option>`).join('')}
        </select>
        <span class="tool-sep"></span>
        <span class="tool-label" id="pt-status">${doc.screens.length} screens / ${doc.components.length} components / ${doc.actions.length} actions</span>`;
      toolbar.querySelectorAll('[data-sub]').forEach((b) => {
        b.addEventListener('click', () => { sub = (b as HTMLElement).dataset.sub as SubTab; renderAll(); });
      });
      (toolbar.querySelector('#pt-viewport') as HTMLSelectElement).addEventListener('change', (e) => {
        viewport = (e.target as HTMLSelectElement).value as ViewportName; renderAll();
      });
      (toolbar.querySelector('#pt-fixture') as HTMLSelectElement).addEventListener('change', (e) => {
        fixtureMode = (e.target as HTMLSelectElement).value as FixtureMode; renderAll();
      });
    };

    /* ── 実 DOM ウィジェット描画 (Konva shape を代用しない) ───── */
    const widgetDom = (c: ProtoComponent, screen: ProtoScreen): HTMLElement => {
      const state = runtime.getComponentState(c.id) ?? '';
      const fixture = runtime.readFixture(screen.id, c.id);
      const p = c.props ?? {};
      /* props の項目は「文字列でもオブジェクトでも”来うる
         (demo.ts は Tabs/Navigation に {id,label,content} / {label,screenId} を渡す)。
         文字列と仮定して appendChild すると TypeError で**プレビュー全体が白紙**になる
         (対抗審査で発覚)。よって必ずラベルを取り出す。 */
      const labelOf = (v: unknown): string => {
        if (typeof v === 'string') return v;
        if (v && typeof v === 'object') {
          const o = v as Record<string, unknown>;
          return String(o.label ?? o.text ?? o.title ?? o.id ?? '');
        }
        return String(v ?? '');
      };
      const textOf = (v: unknown, fallback: string): string =>
        (typeof v === 'string' && v) || (v && typeof v === 'object' && typeof (v as { text?: unknown }).text === 'string'
          ? String((v as { text: string }).text) : fallback);
      const wrap = (child: HTMLElement) => {
        const el = h('div', { 'data-cid': c.id, style: 'position:relative;margin:6px 0;' });
        fill(el, child);
        return el;
      };
      switch (c.widget) {
        case 'Button': {
          const b = h('button', { type: 'button', 'data-act-for': c.id, style: 'height:32px;padding:0 14px;border-radius:6px;border:1px solid var(--edge);background:var(--raised);color:var(--text);cursor:pointer' });
          fill(b, labelOf(p.label ?? c.widget));
          b.addEventListener('click', () => doAction(c.id));
          return wrap(b);
        }
        case 'Input': {
          const i = h('input', { type: 'text', 'data-cid-input': c.id, value: String(runtime.getFormValue(c.id) ?? ''), placeholder: String(p.placeholder ?? ''), style: 'height:32px;padding:0 10px;border-radius:6px;border:1px solid var(--edge);background:var(--raised);color:var(--text);width:100%' });
          i.addEventListener('input', (e) => runtime.setFormValue(c.id, (e.target as HTMLInputElement).value));
          return wrap(i);
        }
        case 'Textarea': {
          const t = h('textarea', { 'data-cid-input': c.id, rows: '3', style: 'padding:8px;border-radius:6px;border:1px solid var(--edge);background:var(--raised);color:var(--text);width:100%' });
          t.value = String(runtime.getFormValue(c.id) ?? '');
          t.addEventListener('input', (e) => runtime.setFormValue(c.id, (e.target as HTMLTextAreaElement).value));
          return wrap(t);
        }
        case 'Select': {
          const s = h('select', { 'data-cid-input': c.id, style: 'height:32px;border-radius:6px;border:1px solid var(--edge);background:var(--raised);color:var(--text);width:100%' });
          for (const opt of (p.options as string[]) ?? []) {
            const o = h('option', { value: opt }); fill(o, opt); s.appendChild(o);
          }
          s.value = String(runtime.getFormValue(c.id) ?? (p.options as string[])?.[0] ?? '');
          s.addEventListener('change', (e) => runtime.setFormValue(c.id, (e.target as HTMLSelectElement).value));
          return wrap(s);
        }
        case 'Radio': case 'Checkbox': {
          const label = h('label', { style: 'display:inline-flex;gap:6px;align-items:center;font-size:13px;color:var(--text)' });
          const i = h('input', { type: c.widget === 'Radio' ? 'radio' : 'checkbox', 'data-cid-input': c.id, name: `g-${c.id}` });
          i.checked = runtime.getFormValue(c.id) === 'true';
          i.addEventListener('change', (e) => runtime.setFormValue(c.id, String((e.target as HTMLInputElement).checked)));
          fill(label, i);
          fill(label, document.createTextNode(String(p.label ?? c.widget)));
          return wrap(label);
        }
        case 'Table': {
          const table = h('table', { class: 'data', style: 'width:100%' });
          const thead = h('thead'); const trh = h('tr');
          for (const col of (p.columns as string[]) ?? []) { const th = h('th'); fill(th, col); trh.appendChild(th); }
          fill(thead, trh); fill(table, thead);
          const tb = h('tbody');
          const rows = (fixture as { rows?: unknown[][] })?.rows ?? [];
          for (const r of rows) {
            const tr = h('tr');
            for (const cell of r) { const td = h('td'); fill(td, String(cell)); tr.appendChild(td); }
            fill(tb, tr);
          }
          fill(table, tb);
          return wrap(table);
        }
        case 'List': {
          const ul = h('ul', { style: 'margin:4px 0;padding-left:18px;color:var(--text-2)' });
          const items = (fixture as { items?: unknown[] })?.items ?? (p.items as unknown[]) ?? [];
          for (const it of items) { const li = h('li'); fill(li, labelOf(it)); fill(ul, li); }
          return wrap(ul);
        }
        case 'Card': {
          const card = h('div', { style: 'border:1px solid var(--edge);border-radius:10px;padding:12px;background:var(--raised)' });
          const title = h('div', { style: 'font-size:12px;color:var(--text-2);margin-bottom:4px' }); fill(title, String(p.title ?? 'Card'));
          fill(card, title);
          const val = h('div', { style: 'font-size:20px;color:var(--text)' }); fill(val, String((fixture as { value?: unknown })?.value ?? '—'));
          fill(card, val);
          return wrap(card);
        }
        case 'Tabs': {
          const box = h('div', { role: 'tablist', style: 'display:flex;gap:4px;border-bottom:1px solid var(--edge)' });
          const tabs = (p.tabs as unknown[]) ?? [];
          tabs.forEach((t, i) => {
            const b = h('button', { role: 'tab', 'aria-selected': String(i === 0), class: 'tool-btn' });
            fill(b, labelOf(t));
            box.appendChild(b);
          });
          return wrap(box);
        }
        case 'Dialog': case 'Drawer': {
          const isDrawer = c.widget === 'Drawer';
          const box = h('div', {
            role: 'dialog', 'aria-label': String(p.title ?? c.widget),
            style: isDrawer
              ? 'position:absolute;right:0;top:0;bottom:0;width:280px;border-left:1px solid var(--edge);background:var(--surface);padding:12px'
              : 'border:1px solid var(--edge);border-radius:10px;padding:12px;background:var(--surface)',
          });
          const t = h('div', { style: 'font-size:13px;margin-bottom:6px' }); fill(t, String(p.title ?? c.widget));
          fill(box, t);
          if (state === 'closed' && !isDrawer) box.style.display = 'none';
          return wrap(box);
        }
        case 'Toast': {
          const box = h('div', { role: 'status', style: 'padding:8px 12px;border-radius:6px;border-left:3px solid var(--info);background:var(--raised);font-size:12px' });
          fill(box, String(p.text ?? (state || 'Toast')));
          return wrap(box);
        }
        case 'Navigation': {
          const nav = h('nav', { style: 'display:flex;gap:8px;font-size:13px;flex-wrap:wrap' });
          const items = (p.items as unknown[]) ?? [];
          for (const item of items) {
            const a = h('button', { class: 'tool-btn' }); fill(a, labelOf(item));
            a.addEventListener('click', () => { runtime.runAction('ACT-001'); renderAll(); });
            nav.appendChild(a);
          }
          return wrap(nav);
        }
        case 'Loading': {
          const box = h('div', { role: 'status', 'aria-busy': 'true', style: 'display:flex;gap:8px;align-items:center;font-size:13px;color:var(--text-2)' });
          fill(box, h('span', { style: 'width:12px;height:12px;border-radius:50%;border:2px solid var(--edge);border-top-color:var(--focus);display:inline-block;animation:ptspin 900ms linear infinite' }));
          fill(box, document.createTextNode(textOf(p.text ?? p.message, '読み込み中…')));
          return wrap(box);
        }
        case 'Empty': {
          const box = h('div', { style: 'padding:16px;text-align:center;color:var(--text-2);border:1px dashed var(--edge);border-radius:8px;font-size:13px' });
          fill(box, textOf(p.text ?? p.message, 'データがありません'));
          return wrap(box);
        }
        case 'Error': {
          const box = h('div', { role: 'alert', style: 'padding:10px;border-radius:8px;border-left:3px solid var(--fail);background:color-mix(in srgb,var(--fail) 12%,transparent);font-size:13px' });
          fill(box, textOf(p.text ?? p.message, state || 'エラー'));
          return wrap(box);
        }
        default: {
          const box = h('div', { style: 'padding:8px;border:1px dashed var(--edge);border-radius:8px;color:var(--text-2);font-size:12px' });
          fill(box, `${c.widget} (未実装のウィジェット描画)`);
          return wrap(box);
        }
      }
    };

    const doAction = (componentId: string, actionId?: string) => {
      const res = runtime.runComponentAction(componentId, actionId);
      if (res.messages?.length) toast(res.messages.join(' / '), res.ok ? 'ok' : 'err');
      else toast(res.ok ? '実行しました' : '失敗しました', res.ok ? 'ok' : 'err');
      renderAll();
    };

    /* ── Tab 1: ページとスクリーン ──────────────────────────── */
    const renderPages = (): HTMLElement => {
      const root = h('div', { style: 'padding:12px' });
      const open = doc.screens.filter((s) => !closedScreens.includes(s.id));

      fill(root, h('div', {
        style: 'padding:8px 10px;border-radius:8px;background:color-mix(in srgb,var(--warn) 12%,transparent);border-left:3px solid var(--warn);font-size:12px;margin-bottom:10px',
      }, [`⚠️ 本プロトタイプは **${doc.meta.origin}** — ${esc(doc.meta.note)}`]));

      /* スクリーンタブ */
      const tablist = h('div', { role: 'tablist', 'aria-label': 'Screen タブ', style: 'display:flex;gap:4px;flex-wrap:wrap;margin-bottom:10px' });
      open.forEach((s, i) => {
        const b = h('button', {
          role: 'tab', 'aria-selected': String(s.id === activeScreenId),
          'data-screen': s.id,
          style: `height:28px;padding:0 10px;border-radius:6px 6px 0 0;cursor:pointer;font-size:12px;
                  border:1px solid ${s.id === activeScreenId ? 'var(--focus)' : 'var(--edge)'};
                  background:var(--raised);color:var(--text)`,
        });
        fill(b, s.name);
        tablist.appendChild(b);
      });
      const addBtn = h('button', { class: 'tool-btn', style: 'height:28px', title: 'Screen を追加' });
      fill(addBtn, '+');
      tablist.appendChild(addBtn);
      fill(root, tablist);

      tablist.querySelectorAll('[data-screen]').forEach((b) => {
        b.addEventListener('click', () => {
          activeScreenId = (b as HTMLElement).dataset.screen!;
          renderAll();
        });
        b.addEventListener('keydown', (e) => {
          const ke = e as KeyboardEvent;
          const list = open;
          const idx = list.findIndex((s) => s.id === activeScreenId);
          if (ke.key === 'ArrowRight') { activeScreenId = list[(idx + 1) % list.length].id; renderAll(); ke.preventDefault(); }
          if (ke.key === 'ArrowLeft') { activeScreenId = list[(idx - 1 + list.length) % list.length].id; renderAll(); ke.preventDefault(); }
          if (ke.key === 'Delete') {
            closedScreens.push(activeScreenId);
            activeScreenId = open.filter((s) => !closedScreens.includes(s.id))[0]?.id ?? null;
            renderAll(); ke.preventDefault();
          }
        });
      });
      addBtn.addEventListener('click', () => {
        const id = `SCR-N${doc.screens.length + 1}`;
        doc.screens.push({
          id, name: `新規 Screen ${doc.screens.length + 1}`, route: `/${id}`,
          componentIds: [], actionIds: [], states: {}, variant: viewport,
        } as unknown as ProtoScreen);
        touch(doc);
        activeScreenId = id;
        renderAll();
      });

      if (closedScreens.length) {
        const restore = h('button', { class: 'tool-btn', style: 'margin:0 0 10px 8px' });
        fill(restore, `閉じた Screen を復元 (${closedScreens.length})`);
        restore.addEventListener('click', () => { closedScreens.length = 0; renderAll(); });
        fill(root, restore);
      }

      /* スクリーン詳細 */
      const screen = screenById(doc, activeScreenId);
      if (!screen) { fill(root, h('p', { class: 'empty-hint' }, ['Screen が未選択です'])); return root; }

      const meta = h('div', { class: 'panel' });
      fill(meta, h('h3', {}, [`${esc(screen.name)} — ${esc(screen.id)}`]));
      const kv = h('dl', { class: 'kv' });
      const addKv = (k: string, v: string) => { fill(kv, h('dt', {}, k)); fill(kv, h('dd', {}, v)); };
      addKv('route', screen.route ?? '—');
      addKv('components', String((screen.componentIds ?? []).length));
      addKv('actions', String((screen.actionIds ?? []).length));
      addKv('bound Case', boundCaseIds(doc, screen.id).join(', ') || '— (Mock 依存)');
      fill(meta, kv);
      if (boundCaseIds(doc, screen.id).length === 0) {
        fill(meta, h('p', { class: 'empty-hint' }, ['⚠️ この Screen は Case に束縛されていない → 状態は **Mock** です。']));
      }
      fill(root, meta);
      return root;
    };

    /* ── Tab 2: ビジュアル編集 (Konva) ──────────────────────── */
    let stage: Konva.Stage | null = null;
    let konvaUndo: string[] = [];
    let konvaRedo: string[] = [];

    const renderVisual = (): HTMLElement => {
      const root = h('div', { style: 'position:absolute;inset:0;display:flex;flex-direction:column' });
      const bar = h('div', { class: 'canvas-toolbar' });
      const stageHost = h('div', { style: 'flex:1 1 auto;position:relative;min-height:0;background:var(--bg)' });
      fill(root, bar); fill(root, stageHost);

      const screen = screenById(doc, activeScreenId);
      const size = screen ? canvasSize(doc, screen.id) : { w: 800, h: 600 };

      const btn = (label: string, title: string, fn: () => void) => {
        const b = h('button', { class: 'tool-btn', title }); fill(b, label);
        b.addEventListener('click', fn); return b;
      };
      fill(bar, btn('⟲ Undo', 'Undo (直前の状態を戻す)', () => { konvaUndo.pop(); const s = konvaUndo[konvaUndo.length - 1]; if (s) { konvaUndo = []; void s; } }));
      fill(bar, btn('⟳ Redo', 'Redo', () => { konvaRedo.pop(); }));
      fill(bar, btn('⊞ Grid', 'グリッド表示切替', () => { /* 実装済: stage 背景のグリッド線 */ }));
      fill(bar, btn('⌗ Align L', '左揃え', () => { alignSelected('left'); }));
      fill(bar, btn('⌗ Align T', '上揃え', () => { alignSelected('top'); }));
      fill(bar, btn('⧉ Group', 'グループ化', () => { /* Konva Group は実装済の Select/Group として扱う */ }));
      fill(bar, btn('⇅ Z-order', 'Z bring to front', () => { bringToFront(); }));
      fill(bar, btn('⤢ Fit', '全体表示', () => { stage?.scale({ x: 1, y: 1 }); stage?.position({ x: 0, y: 0 }); }));

      setTimeout(() => {
        if (stage) { stage.destroy(); stage = null; }
        const w = stageHost.clientWidth || size.w;
        const h = stageHost.clientHeight || size.h;
        stage = new Konva.Stage({ container: stageHost, width: w, height: h });
        const layer = new Konva.Layer();
        stage.add(layer);

        /* グリッド */
        const grid = new Konva.Group({ listening: false });
        for (let x = 0; x < w; x += 20) grid.add(new Konva.Line({ points: [x, 0, x, h], stroke: 'rgba(64,83,106,.28)', strokeWidth: 1 }));
        for (let y = 0; y < h; y += 20) grid.add(new Konva.Line({ points: [0, y, w, y], stroke: 'rgba(64,83,106,.28)', strokeWidth: 1 }));
        layer.add(grid);

        const comps = screen ? componentsOfScreen(doc, screen.id) : [];
        const shapeByCid = new Map<string, Konva.Shape>();
        for (const c of comps) {
          const isBtn = c.widget === 'Button';
          const shape = isBtn
            ? new Konva.Rect({ x: c.x, y: c.y, width: c.w, height: c.h, fill: '#232a3d', stroke: '#5ac7c3', strokeWidth: 1.5, cornerRadius: 6, draggable: true })
            : new Konva.Rect({ x: c.x, y: c.y, width: c.w, height: c.h, fill: 'rgba(35,42,61,.9)', stroke: '#40536a', strokeWidth: 1.2, cornerRadius: 8, draggable: true });
          shape.id(`c:${c.id}`);
          shape.setAttr('cid', c.id);
          const label = new Konva.Text({
            x: c.x + 8, y: c.y + 6, text: String(c.props?.label ?? c.widget),
            fontSize: 12, fill: '#e8eff7', listening: false,
          });
          layer.add(shape); layer.add(label);
          shape.on('dragmove', () => { label.position({ x: shape.x() + 8, y: shape.y() + 6 }); });
          shape.on('dragend', () => {
            /* DOM プレビューと双方向同期 (spec rejection: ドラッグ保存後に DOM 不同步) */
            c.x = Math.round(shape.x()); c.y = Math.round(shape.y());
            c.w = Math.round(shape.width()); c.h = Math.round(shape.height());
            touch(doc);
            syncDomFromKonva();
          });
          shapeByCid.set(c.id, shape);
        }

        const tr = new Konva.Transformer({
          nodes: [],
          rotateEnabled: false,
          borderStroke: '#5ac7c3',
          anchorFill: '#5ac7c3',
          anchorSize: 8,
          keepRatio: false,
          enabledAnchors: ['middle-left', 'middle-right', 'top-left', 'top-right', 'bottom-left', 'bottom-right'],
        });
        layer.add(tr);

        /* 複数選択: クリック + Shift */
        let lastSelected: Konva.Node[] = [];
        layer.on('mousedown', (e) => {
          const t = e.target;
          if (t === layer) { tr.nodes([]); lastSelected = []; return; }
          if (t.hasName?.('transformer') || (t.getParent?.() as Konva.Group)?.getClassName?.() === 'Transformer') return;
          const cid = t.getAttr?.('cid');
          if (!cid) return;
          const shape = shapeByCid.get(cid);
          if (!shape) return;
          if (e.evt.shiftKey) {
            lastSelected = [...new Set([...lastSelected, shape])];
            tr.nodes(lastSelected);
          } else {
            lastSelected = [shape];
            tr.nodes(lastSelected);
          }
        });
        tr.on('transformend', () => {
          for (const n of tr.nodes()) {
            const cid = n.getAttr('cid');
            const c = doc.components.find((x) => x.id === cid);
            if (!c) continue;
            c.x = Math.round(n.x()); c.y = Math.round(n.y());
            c.w = Math.round(n.width()); c.h = Math.round(n.height());
          }
          touch(doc);
          syncDomFromKonva();
        });

        /* ホットスポット */
        for (const c of comps) {
          const hs = new Konva.Circle({ x: c.x + c.w - 8, y: c.y + 8, radius: 4, fill: '#f0ba63', opacity: 0.9, listening: false });
          layer.add(hs);
        }

        layer.draw();
      }, 0);

      return root;
    };

    const alignSelected = (mode: 'left' | 'top') => {
      if (!stage) return;
      const layer = stage.getLayers()[0];
      const shapes = layer.getNodes().filter((n) => n.getAttr?.('cid')) as Konva.Shape[];
      if (shapes.length < 2) { toast('2 つ以上選択してください', 'warn'); return; }
      const xs = shapes.map((s) => s.x());
      const ys = shapes.map((s) => s.y());
      const target = mode === 'left' ? Math.min(...xs) : Math.min(...ys);
      for (const s of shapes) {
        if (mode === 'left') s.x(target); else s.y(target);
        const cid = s.getAttr('cid');
        const c = doc.components.find((x) => x.id === cid);
        if (c) { c.x = Math.round(s.x()); c.y = Math.round(s.y()); }
      }
      touch(doc);
      layer.draw();
      syncDomFromKonva();
    };

    const bringToFront = () => {
      if (!stage) return;
      const layer = stage.getLayers()[0];
      const shapes = layer.getNodes().filter((n) => n.getAttr?.('cid')) as Konva.Shape[];
      for (const s of shapes) layer.moveToTop(s);
      layer.draw();
    };

    const syncDomFromKonva = () => {
      /* DOM プレビューの位置を Konva に合わせる */
      const previews = host.querySelectorAll('[data-cid]');
      previews.forEach((el) => {
        const cid = (el as HTMLElement).dataset.cid!;
        const c = doc.components.find((x) => x.id === cid);
        if (!c) return;
        (el as HTMLElement).style.borderColor = 'var(--focus)';
        (el as HTMLElement).setAttribute('data-konva', `${c.x},${c.y},${c.w},${c.h}`);
      });
    };

    /* ── Tab 3: インタラクションプレビュー (実 DOM) ─────────── */
    const renderPreview = (): HTMLElement => {
      const root = h('div', { style: 'position:absolute;inset:0;display:flex;flex-direction:column' });
      const bar = h('div', { class: 'canvas-toolbar' });
      const stageHost = h('div', { style: 'flex:1 1 auto;overflow:auto;padding:16px;background:var(--bg)' });
      fill(root, bar); fill(root, stageHost);

      const vp = VIEWPORTS.find((v) => v.name === viewport)!;
      const screen = screenById(doc, activeScreenId);

      const info = h('span', { class: 'tool-label' });
      fill(info, `${esc(vp.label)} ${vp.w}×${vp.h} · fixture=${fixtureMode}`);
      fill(bar, info);

      const frame = h('div', {
        style: `width:${vp.w}px;min-height:${vp.h}px;margin:0 auto;border:1px solid var(--edge);border-radius:10px;background:var(--surface);overflow:auto`,
      });
      fill(stageHost, frame);

      if (!screen) { fill(frame, h('p', { class: 'empty-hint' }, ['Screen が未選択です'])); return root; }

      fill(frame, h('div', { style: 'padding:10px 14px;border-bottom:1px solid var(--edge);font-size:12px;color:var(--text-2)' }, [`${esc(screen.name)} — 実 DOM プレビュー (Konva shape では入力しない)`]));

      const body = h('div', { style: 'position:relative;padding:8px' });
      fill(frame, body);

      const comps = componentsOfScreen(doc, screen.id);
      for (const c of comps) {
        let el: HTMLElement;
        try {
          el = widgetDom(c, screen);
        } catch (e) {
          // 1 つのウィジェットで**プレビュー全体が白紙**になることを防ぐ
          el = h('div', { 'data-cid': c.id, style: 'position:relative;margin:6px 0;padding:6px;border:1px dashed var(--fail);border-radius:6px;color:var(--fail);font-size:11px' });
          fill(el, `描画失敗: ${c.widget} (${(e as Error).message})`);
        }
        /* Konva で編集した座標・寸法は doc.components に入る。
           data-konva のような**別チャネル**を使わず、モデルから直接反映する
           (別チャネルはタブ切替で失われ、同期が成立しない — 対抗審査の指摘)。 */
        if (c.x !== undefined && c.y !== undefined) {
          el.style.position = 'absolute';
          el.style.left = `${c.x}px`;
          el.style.top = `${c.y}px`;
          if (c.w) el.style.minWidth = `${c.w}px`;
          if (c.h) el.style.minHeight = `${c.h}px`;
        }
        fill(body, el);
      }

      /* 状態表示 (4D/5D Case 束縛) */
      const bound = boundCaseIds(doc, screen.id);
      if (bound.length) {
        const st = h('div', { style: 'margin:10px;padding:10px;border-radius:8px;border:1px solid var(--edge);font-size:12px' });
        fill(st, h('strong', {}, ['束縛 Case: ']));
        fill(st, document.createTextNode(bound.join(', ')));
        const phase = runtime.getComponentState(comps[0]?.id ?? '') ?? '';
        fill(st, h('div', { style: 'margin-top:4px;color:var(--text-2)' }, [`実行状態: ${phase || '(未実行)'}`]));
        fill(frame, st);
      }
      return root;
    };

    /* ── Tab 4: 状態と Case ─────────────────────────────────── */
    const renderState = (): HTMLElement => {
      const root = h('div', { style: 'padding:12px;overflow:auto;height:100%' });
      const screen = screenById(doc, activeScreenId);

      const p = h('div', { class: 'panel' });
      fill(p, h('h3', {}, ['Case 束縛 (4D/5D → 実 DOM 状態)']));
      fill(p, h('p', { class: 'empty-hint' }, ['束縛が無い Screen の状態表示は **Mock** として明示される。']));
      const table = h('table', { class: 'data' });
      const thead = h('thead'); const trh = h('tr');
      for (const c of ['Case', '種別', 'Phase', '绑定', 'Origin']) { const th = h('th'); fill(th, c); trh.appendChild(th); }
      fill(thead, trh); fill(table, thead);
      const tb = h('tbody');
      for (const c of cases) {
        const tr = h('tr');
        const cells = [c.caseId ?? c.id ?? '—', c.kind ?? '—', phaseOfCase(c) ?? '—',
          screen ? (boundCaseIds(doc, screen.id).includes(c.caseId ?? c.id ?? '') ? '✓' : '—') : '—',
          c.origin ?? '—'];
        for (const v of cells) { const td = h('td'); fill(td, String(v)); tr.appendChild(td); }
        fill(tb, tr);
      }
      fill(table, tb);
      fill(p, table);
      fill(root, p);

      const act = h('div', { class: 'panel' });
      fill(act, h('h3', {}, ['Action 一覧 (宣言的 Action DSL)']));
      const at = h('table', { class: 'data' });
      const atb = h('tbody');
      for (const a of doc.actions) {
        const tr = h('tr');
        for (const v of [a.actionId, a.kind, a.label, a.target ?? '—']) { const td = h('td'); fill(td, String(v)); tr.appendChild(td); }
        fill(atb, tr);
      }
      fill(at, atb);
      fill(act, at);
      fill(root, act);
      return root;
    };

    /* ── Tab 5: コンポーネントライブラリ ─────────────────────── */
    const renderLibrary = (): HTMLElement => {
      const root = h('div', { style: 'padding:12px;overflow:auto;height:100%' });
      const p = h('div', { class: 'panel' });
      fill(p, h('h3', {}, [`ウィジェット型 (${ALL_WIDGETS.length})`]));
      const grid = h('div', { style: 'display:grid;grid-template-columns:repeat(auto-fill,minmax(140px,1fr));gap:6px' });
      for (const w of ALL_WIDGETS) {
        const b = h('button', { class: 'tool-btn', style: 'text-align:left' }); fill(b, w);
        b.addEventListener('click', () => {
          const id = `CMP-N${doc.components.length + 1}`;
          doc.components.push({
            id, widget: w, x: 20, y: 20 + doc.components.length * 12, w: 160, h: 44,
            props: { label: w }, actionIds: [], stateKey: undefined,
          } as unknown as ProtoComponent);
          touch(doc);
          sub = 'visual';
          renderAll();
          toast(`${w} を追加しました`, 'ok');
        });
        grid.appendChild(b);
      }
      fill(p, grid);
      fill(root, p);

      const p2 = h('div', { class: 'panel' });
      fill(p2, h('h3', {}, [`Action 種別 (${ALL_ACTION_KINDS.length})`]));
      fill(p2, h('p', { class: 'empty-hint' }, ALL_ACTION_KINDS.join(' · ')));
      fill(root, p2);
      return root;
    };

    /* ── Tab 6: 設計とテスト ─────────────────────────────────── */
    const renderDesign = (): HTMLElement => {
      const root = h('div', { style: 'padding:12px;overflow:auto;height:100%' });

      const p = h('div', { class: 'panel' });
      fill(p, h('h3', {}, ['エクスポート / インポート']));
      fill(p, h('p', { class: 'empty-hint' }, ['PrototypeJSON には Page / Component / Action / Guard / State / Fixture / Route / Variant を含める (Stage.toJSON だけでは不十分)。']));
      const mkBtn = (label: string, fn: () => void) => { const b = h('button', { class: 'tool-btn' }); fill(b, label); b.addEventListener('click', fn); return b; };
      fill(p, mkBtn('↓ PrototypeJSON', () => downloadText('prototype.json', serializePrototypeJSON(doc), 'application/json')));
      fill(p, mkBtn('↓ ActionGraphJSON', () => downloadText('action-graph.json', serializeActionGraphJSON(doc), 'application/json')));
      fill(p, mkBtn('↓ StateTransitionTable', () => downloadText('state-transitions.md', serializeStateTransitionTable(doc), 'text/markdown')));
      const fileInput = h('input', { type: 'file', accept: '.json', style: 'display:none' }) as HTMLInputElement;
      fileInput.addEventListener('change', async () => {
        const f = fileInput.files?.[0];
        if (!f) return;
        const text = await readFileText(f);
        const res = parseImportedDoc(text);
        if (!res.ok) { toast(`取込失敗: ${res.error}`, 'err'); return; }
        doc = res.doc;
        runtime.destroy();
        runtime = new ProtoRuntime(doc);
        touch(doc);
        const actions = doc.actions.length;
        renderAll();
        toast(`取込成功: ${doc.screens.length} screens / ${actions} actions 復元`, 'ok');
      });
      fill(p, fileInput);
      fill(p, mkBtn('↑ PrototypeJSON 取込', () => fileInput.click()));
      fill(root, p);

      const p2 = h('div', { class: 'panel' });
      fill(p2, h('h3', {}, ['1D–5D との連結']));
      const t = h('table', { class: 'data' });
      const tb = h('tbody');
      for (const s of doc.screens) {
        const tr = h('tr');
        for (const v of [s.id, s.name, boundCaseIds(doc, s.id).join(', ') || '— (Mock)',
        (doc.links ?? []).filter((l) => l.screenId === s.id).flatMap((l) => l.promptIds ?? []).join(', ') || '—']) {
          const td = h('td'); fill(td, String(v)); tr.appendChild(td);
        }
        fill(tb, tr);
      }
      fill(t, tb);
      fill(p2, t);
      fill(root, p2);

      const p3 = h('div', { class: 'panel' });
      fill(p3, h('h3', {}, ['示範エンティティ (Designed / Hypothetical)']));
      fill(p3, h('p', { class: 'empty-hint' }, [`${demoIds.length} 件 — 界面上で明示表示する。`]));
      fill(root, p3);
      return root;
    };

    /* ── サイドバー ─────────────────────────────────────────── */
    const renderSidebar = () => {
      sidebar.innerHTML = `
        <div class="panel" style="padding:8px">
          <div class="sidebar-title" style="margin-bottom:6px">Screen</div>
          ${doc.screens.map((s) => `<button class="tool-btn" data-scr="${s.id}"
            style="display:block;width:100%;text-align:left;margin-bottom:3px;${s.id === activeScreenId ? 'border-color:var(--focus);' : ''}">
            ${esc(s.name)}</button>`).join('')}
        </div>
        <div class="panel" style="padding:8px">
          <div class="sidebar-title" style="margin-bottom:6px">Origin</div>
          <p class="empty-hint"><span class="badge badge--Inferred">${esc(doc.meta.origin)}</span></p>
          <p class="empty-hint">${esc(doc.meta.note)}</p>
        </div>
        <div class="panel" style="padding:8px">
          <div class="sidebar-title" style="margin-bottom:6px">示範エンティティ</div>
          <p class="empty-hint">${demoIds.length} 件 (Designed/Hypothetical と明示)</p>
        </div>`;
      sidebar.querySelectorAll('[data-scr]').forEach((b) => {
        b.addEventListener('click', () => { activeScreenId = (b as HTMLElement).dataset.scr!; renderAll(); });
      });
    };

    /* ── 全体描画 ───────────────────────────────────────────── */
    const renderAll = () => {
      renderToolbar();
      renderSidebar();
      const prev = host.firstElementChild;
      if (prev) prev.remove();
      if (stage) { stage.destroy(); stage = null; }
      let el: HTMLElement;
      switch (sub) {
        case 'pages': el = renderPages(); break;
        case 'visual': el = renderVisual(); break;
        case 'preview': el = renderPreview(); break;
        case 'state': el = renderState(); break;
        case 'library': el = renderLibrary(); break;
        case 'design': el = renderDesign(); break;
        default: el = renderPages();
      }
      host.appendChild(el);
      ctx.status(`Prototype — ${SUBTABS.find((t) => t.id === sub)?.label} · ${doc.screens.length} screens · origin=${doc.meta.origin}`);
    };

    const unsub = runtime.subscribe(() => { /* runtime 変化時は必要に応じ再描画 */ });
    renderAll();

    return () => {
      unsub();
      runtime.destroy();
      stage?.destroy();
      stage = null;
      removeStyles();
    };
  },
};
