/**
 * demo2d.ts — 2D 認知負荷低減デモツアー
 *
 * 2D は 160 ノード / 14 図種 / 6 レイヤーが一度に出ると、最初の一歩が分からない。
 * このツアーは「次に何をするか」を**Coach Mark** (呼び出し注記) で順に示し、
 * 対象を実際にハイライト・アニメーションさせる。
 *
 * 設計:
 *   - **isms 卵図には頼らない**。対象要素は実 DOM を矩形で囲って示す。
 *   - 各ステップは任意で**実際の操作**を呼ぶ (focusNode / openDiagram / setLayer)。
 *     説明だけ的大门で終わらせない。
 *   - Esc / 「スキップ」で即座に解除。Esc キーは他の操作を奪わない。
 *   - prefers-reduced-motion なら paketing アニメーションを静止させる。
 *   - zing モード (2D の canvas の上に重なる) で、 tours 中は下の操作を軽く無効化しない
 *     (Coach Mark 側のボタンだけが操作を奪う)。
 */

export interface DemoActions {
  /** ノードにフォーカスして選択する (ズーム + 選択) */
  focusNode(id: string): void;
  /** 独立図を開く */
  openDiagram(type: string): void;
  /** レイヤーの表示を切り替える */
  setLayer(key: string, visible: boolean): void;
  /** 検索文字列を設定 */
  setSearch(q: string): void;
  /** 現在選択中のノード ID */
  selectedNodeId(): string | null;
  /** 2D の内部状態 (STEP の例示に使う) */
  stats(): { nodes: number; edges: number; ports: number; diagrams: number };
}

interface Step {
  id: string;
  title: string;
  body: string;
  /** ハイライト対象の CSS セレクタ (最初の 1 件) */
  anchor?: string;
  /** ハイライトを当てる対象の種類 */
  spot?: 'canvas' | 'sidebar' | 'toolbar' | 'inspector' | 'node' | 'edge' | 'none';
  /** 実行する操作 */
  act?: (a: DemoActions) => void;
  /** 3D への誘導ステップか */
  handoff?: boolean;
}

const STYLE = `
.d2-demo-mask{position:absolute;inset:0;z-index:40;pointer-events:none;}
.d2-demo-ring{position:absolute;border:2px solid var(--focus);border-radius:12px;
  box-shadow:0 0 0 9999px rgba(4,8,14,.55), 0 0 0 4px color-mix(in srgb,var(--focus) 40%,transparent);
  transition:all var(--t-std) var(--ease);pointer-events:none;}
.d2-demo-ring.is-pulse{animation:d2-demo-pulse 1.6s ease-in-out infinite;}
@keyframes d2-demo-pulse{0%,100%{box-shadow:0 0 0 9999px rgba(4,8,14,.55),0 0 0 0 color-mix(in srgb,var(--focus) 55%,transparent);}
  50%{box-shadow:0 0 0 9999px rgba(4,8,14,.55),0 0 0 14px color-mix(in srgb,var(--focus) 28%,transparent);}}
.d2-demo-card{position:absolute;z-index:41;max-width:340px;background:var(--surface);
  border:1px solid var(--focus);border-radius:12px;box-shadow:var(--shadow-3);padding:12px 14px;pointer-events:auto;}
.d2-demo-card h4{margin:0 0 6px;font-size:13px;color:var(--text);}
.d2-demo-card p{margin:0 0 10px;font-size:12px;color:var(--text-2);line-height:1.55;}
.d2-demo-foot{display:flex;align-items:center;gap:6px;}
.d2-demo-foot button{height:26px;padding:0 10px;font-size:12px;border-radius:6px;cursor:pointer;
  border:1px solid var(--edge);background:var(--raised);color:var(--text);}
.d2-demo-foot button.primary{border-color:var(--focus);color:var(--focus);}
.d2-demo-foot .sp{flex:1 1 auto;}
.d2-demo-count{font-size:11px;color:var(--text-2);font-family:var(--mono);}
@media (prefers-reduced-motion: reduce){.d2-demo-ring.is-pulse{animation:none;}}
`;

export function buildDemoSteps(a: DemoActions): Step[] {
  const s = a.stats();
  const firstNode = a.selectedNodeId();
  return [
    {
      id: 'intro', title: '2D 无尽画布',
      body: `这里有 ${s.nodes} 个节点、${s.edges} 条边、${s.ports} 个端口, 全部来自 RustGameServer 仓库的真实采集。` +
        '接下来 6 步, 走一遍最重要的路径。',
      spot: 'canvas',
    },
    {
      id: 'prompt', title: '1D 的 Prompt 决定画什么',
      body: '如果 1D 选中了 Prompt, 它的 nodeEdgeIds 会成为这里的绘制范围。' +
        '没有选中 Prompt 时, 画布显示全部事实节点。',
      anchor: '#d2-prompt-scope',
    },
    {
      id: 'layers', title: '图层: 一次只看一层',
      body: '节点 / 边 / 端口三层可以独立开关。密集时先关掉端口层。',
      spot: 'sidebar',
      act: (x) => { x.setLayer('port', false); },
    },
    {
      id: 'select', title: '点一个节点',
      body: '节点可以被独立选中 (边和端口也可以)。右侧检查器会显示它的**真实证据**: file:line 与 revision。',
      spot: 'canvas',
      act: (x) => { if (firstNode) x.focusNode(firstNode); },
    },
    {
      id: 'diagram', title: '14 种专业工程图',
      body: '类图 / ER / 时序 / BPMN 等各有独立渲染器, 可以浮动、缩放、对比。' +
        '它们是审查单元, 不是一个通用图。',
      act: (x) => x.openDiagram('UML_Class'),
    },
    {
      id: 'handoff', title: '2D → 3D',
      body: '在 3D 里, 刚才选中的节点会被聚焦, 无关对象会退避。' +
        '接着 3D → 4D (跑 Case) → 5D (做因果实验) 是一条连续的链。',
      handoff: true,
    },
  ];
}

export interface DemoTour {
  stop(): void;
  readonly active: boolean;
}

/* バスへの通知 (bus.ts は d2.ts を import しないので循環しない) */
import { bus } from '../core/bus';

/**
 * startDemoTour — ツアーを開始し、停止関数を返す。
 * 同じホストで再呼び出しされた場合は前のツアーを停止する。
 */
export function startDemoTour(host: HTMLElement, actions: DemoActions): () => void {
  // 既存のツアーを掃除
  host.querySelectorAll('.d2-demo-mask, .d2-demo-ring, .d2-demo-card').forEach((e) => e.remove());

  const style = document.createElement('style');
  style.textContent = STYLE;
  document.head.appendChild(style);

  const steps = buildDemoSteps(actions);
  let idx = 0;

  const mask = document.createElement('div');
  mask.className = 'd2-demo-mask';
  const ring = document.createElement('div');
  ring.className = 'd2-demo-ring is-pulse';
  const card = document.createElement('div');
  card.className = 'd2-demo-card';
  card.setAttribute('role', 'dialog');
  card.setAttribute('aria-label', '2D デモツアー');
  host.appendChild(mask);
  host.appendChild(ring);
  host.appendChild(card);

  const place = () => {
    const st = steps[idx];
    let rect: DOMRect | null = null;
    if (st.anchor) {
      const el = document.querySelector(st.anchor) as HTMLElement | null;
      if (el) rect = el.getBoundingClientRect();
    }
    if (!rect && st.spot && st.spot !== 'none') {
      const sel: Record<string, string> = {
        canvas: '.d2-stage',
        sidebar: '#sidebar-body',
        toolbar: '#canvas-toolbar',
        inspector: '#inspector-body',
        node: '[data-node-id]',
        edge: '[data-edge-id]',
      };
      const el = document.querySelector(sel[st.spot]) as HTMLElement | null;
      if (el) rect = el.getBoundingClientRect();
    }
    const hostRect = host.getBoundingClientRect();
    if (rect) {
      const pad = 6;
      ring.style.left = `${rect.left - hostRect.left - pad}px`;
      ring.style.top = `${rect.top - hostRect.top - pad}px`;
      ring.style.width = `${rect.width + pad * 2}px`;
      ring.style.height = `${rect.height + pad * 2}px`;
      ring.style.display = 'block';
    } else {
      ring.style.display = 'none';
    }

    card.innerHTML = `
      <h4>${st.title}</h4>
      <p>${st.body}</p>
      <div class="d2-demo-foot">
        <span class="d2-demo-count">${idx + 1} / ${steps.length}</span>
        <span class="sp"></span>
        <button data-act="skip">スキップ</button>
        <button data-act="prev" ${idx === 0 ? 'disabled' : ''}>前</button>
        <button class="primary" data-act="next">${idx === steps.length - 1 ? '完了' : '次へ'}</button>
      </div>`;
    // カード位置: 対象の下 or 右
    const cr = card.getBoundingClientRect();
    let cx = 16, cy = 16;
    if (rect) {
      cx = Math.min(Math.max(8, rect.left - hostRect.left), hostRect.width - cr.width - 8);
      const below = rect.bottom - hostRect.top + 12;
      cy = below + cr.height < hostRect.height ? below : Math.max(8, rect.top - hostRect.top - cr.height - 12);
    }
    card.style.left = `${cx}px`;
    card.style.top = `${cy}px`;

    card.querySelector('[data-act="next"]')?.addEventListener('click', () => {
      if (idx === steps.length - 1) { stop(); return; }
      idx += 1; runStep();
    });
    card.querySelector('[data-act="prev"]')?.addEventListener('click', () => {
      if (idx > 0) { idx -= 1; runStep(); }
    });
    card.querySelector('[data-act="skip"]')?.addEventListener('click', () => stop());
  };

  const runStep = () => {
    const st = steps[idx];
    try { st.act?.(actions); } catch { /* 操作に失敗してもツアーは続行 */ }
    place();
    bus.setDemoTour(idx, true);
  };

  const onKey = (e: KeyboardEvent) => {
    const t = e.target as HTMLElement | null;
    if (t && ['INPUT', 'TEXTAREA', 'SELECT'].includes(t.tagName)) return;
    if (e.key === 'Escape') { e.stopPropagation(); stop(); }
    if (e.key === 'ArrowRight') { if (idx < steps.length - 1) { idx += 1; runStep(); } }
    if (e.key === 'ArrowLeft') { if (idx > 0) { idx -= 1; runStep(); } }
  };
  document.addEventListener('keydown', onKey, true);

  function stop() {
    document.removeEventListener('keydown', onKey, true);
    mask.remove(); ring.remove(); card.remove(); style.remove();
    bus.setDemoTour(0, false);
  }

  runStep();
  return stop;
}
