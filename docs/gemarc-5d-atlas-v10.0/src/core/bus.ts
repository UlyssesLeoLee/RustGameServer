/**
 * bus.ts — 维度脊柱 (Dimension Spine)
 *
 * 5D は 4D の上に、4D は 3D の上に、2D は 1D の上に**立つ**。
 * それを実装する唯一の方法は、各次元が独立した世界を持ち続けることではない。
 *
 *   1D Prompt  ──(promptScope)──►  2D 無限キャンバス
 *   2D 選択    ──(selection2d)──►  3D 空間ステージ
 *   3D 焦点    ──(focus3d)─────►  4D Case 実行
 *   4D Trace   ──(trace)───────►  5D A/B 因果実験
 *   5D 分岐    ──(branch)──────►  3D ステージ (因果オーバーレイ / 分屏)
 *
 * このバスは**唯一**の接続点。各次元は
 *   - 上流から状態を読む (自分のスコープを決める)
 *   - 自分の状態を公開する (下流が 그것 を読む)
 * という 2 つの役割だけを持つ。
 *
 * 重要な不変条件:
 *   - アニメーションは Event Trace の**再生**であり、FPS は結果を決めない
 *   - 3D/4D/5D は**同じ 3D ステージ**を共有する (SceneHost)
 *   - 干渉は実験コピーに対する ScenarioGraphDelta であり、正式事実グラフは不変
 */

import type { SimulationEvent } from './types';

export type SelectionKind = 'node' | 'edge' | 'port' | 'room' | null;

export interface Selection {
  kind: SelectionKind;
  id: string | null;
  /** 選択元次元 (UI ヒント表示用) */
  origin?: Dimension;
}
export type Dimension = '0D' | '1D' | '2D' | '3D' | '4D' | '5D' | 'Prototype';

export interface AtlasState {
  /* ── 1D → 2D: 選択中 Prompt が定義する設計スコープ ────────── */
  promptId: string | null;
  promptScopeNodeIds: Set<string>;
  promptScopeEdgeIds: Set<string>;
  promptTitle: string | null;

  /* ── 2D → 3D: 2D で選んだもの ─────────────────────────── */
  selection2d: Selection;
  /** 2D のグループ (ドメイン) — 3D のフロア選択に連動 */
  group2d: string | null;

  /* ── 3D → 4D: 3D で焦点を当てたノード集合 ───────────────── */
  focus3d: Set<string> | null;
  activeFloorId: string | null;
  activeRoomId: string | null;

  /* ── 4D → 5D / 3D: 実行中の Trace ─────────────────────── */
  traceId: string | null;
  caseId: string | null;
  caseTitle: string | null;
  events: SimulationEvent[];
  playhead: number;
  /** playhead 時点のノード状態 (ノードID → ランタイム状態) */
  states: Record<string, string>;
  playing: boolean;
  speed: number;

  /* ── 5D → 3D: 実験の分岐 ──────────────────────────────── */
  branchTraceId: string | null;
  branchEvents: SimulationEvent[];
  branchPlayhead: number;
  branchStates: Record<string, string>;
  /** 3D 上に因果オーバーレイを出すか / 分屏するか */
  causalMode: 'none' | 'overlay' | 'split';

  /** 2D のデモツアー (認知負荷を下げる導線) */
  demoTourStep: number;
  demoTourActive: boolean;
}

function initial(): AtlasState {
  return {
    promptId: null, promptScopeNodeIds: new Set(), promptScopeEdgeIds: new Set(), promptTitle: null,
    selection2d: { kind: null, id: null, origin: '2D' },
    group2d: null,
    focus3d: null, activeFloorId: null, activeRoomId: null,
    traceId: null, caseId: null, caseTitle: null, events: [], playhead: 0, states: {},
    playing: false, speed: 1,
    branchTraceId: null, branchEvents: [], branchPlayhead: 0, branchStates: {},
    causalMode: 'none',
    demoTourStep: 0, demoTourActive: false,
  };
}

type Listener = (s: AtlasState) => void;

class Spine {
  private listeners = new Set<Listener>();
  private state: AtlasState = initial();
  private timer: number | null = null;

  get snapshot(): AtlasState { return this.state; }

  subscribe(fn: Listener): () => void {
    this.listeners.add(fn);
    fn(this.state);
    return () => { this.listeners.delete(fn); };
  }

  private emit() { for (const l of this.listeners) l(this.state); }

  private patch(p: Partial<AtlasState>) {
    this.state = { ...this.state, ...p };
    this.emit();
  }

  /* ── 1D → 2D ─────────────────────────────────────────── */
  setPrompt(p: { id: string | null; title?: string | null; nodeIds?: string[]; edgeIds?: string[] }): void {
    this.patch({
      promptId: p.id,
      promptTitle: p.title ?? null,
      promptScopeNodeIds: new Set(p.nodeIds ?? []),
      promptScopeEdgeIds: new Set(p.edgeIds ?? []),
    });
  }

  /* ── 2D → 3D ─────────────────────────────────────────── */
  setSelection2d(sel: Selection): void { this.patch({ selection2d: sel }); }
  setGroup2d(g: string | null): void { this.patch({ group2d: g }); }

  /* ── 3D → 4D ─────────────────────────────────────────── */
  setFocus3d(set: Set<string> | null): void { this.patch({ focus3d: set }); }
  setActiveFloor(id: string | null): void { this.patch({ activeFloorId: id }); }
  setActiveRoom(id: string | null): void { this.patch({ activeRoomId: id }); }

  /* ── 4D Trace ─────────────────────────────────────────── */
  publishTrace(t: { traceId: string; caseId: string; caseTitle: string; events: SimulationEvent[]; finalStates: Record<string, string> }): void {
    this.patch({
      traceId: t.traceId, caseId: t.caseId, caseTitle: t.caseTitle,
      events: t.events, playhead: 0, states: this.statesAt(t.events, 0),
    });
  }
  publishBranch(t: { traceId: string; events: SimulationEvent[]; finalStates: Record<string, string> } | null): void {
    if (!t) {
      this.patch({ branchTraceId: null, branchEvents: [], branchPlayhead: 0, branchStates: {} });
      return;
    }
    this.patch({
      branchTraceId: t.traceId, branchEvents: t.events, branchPlayhead: 0,
      branchStates: this.statesAt(t.events, 0),
    });
  }
  setCausalMode(m: AtlasState['causalMode']): void { this.patch({ causalMode: m }); }

  private statesAt(events: SimulationEvent[], idx: number): Record<string, string> {
    const out: Record<string, string> = {};
    for (const e of events) {
      if (e.sequence > idx) break;
      if (e.sourceNodeId && e.nextState) out[e.sourceNodeId] = e.nextState;
      if (e.targetNodeId && e.nextState) out[e.targetNodeId] = e.nextState;
    }
    return out;
  }

  seek(idx: number): void {
    const max = this.state.events.length - 1;
    const p = Math.max(0, Math.min(idx, max));
    this.patch({ playhead: p, states: this.statesAt(this.state.events, p) });
  }
  seekBranch(idx: number): void {
    const max = this.state.branchEvents.length - 1;
    const p = Math.max(0, Math.min(idx, max));
    this.patch({ branchPlayhead: p, branchStates: this.statesAt(this.state.branchEvents, p) });
  }
  step(d: number): void { this.seek(this.state.playhead + d); }
  stepBranch(d: number): void { this.seekBranch(this.state.branchPlayhead + d); }
  setSpeed(s: number): void { this.patch({ speed: s }); }

  play(): void {
    if (!this.state.events.length) return;
    if (this.state.playhead >= this.state.events.length - 1) this.seek(0);
    this.patch({ playing: true });
    this.tick();
  }
  pause(): void { this.stop(); this.patch({ playing: false }); }
  private tick = () => {
    if (!this.state.playing) return;
    this.step(1);
    if (this.state.playhead >= this.state.events.length - 1) { this.pause(); return; }
    this.timer = setTimeout(this.tick, Math.max(16, 220 / Math.max(0.1, this.state.speed))) as unknown as number;
  };
  stop(): void { if (this.timer !== null) { clearTimeout(this.timer); this.timer = null; } }
  reset(): void { this.pause(); this.seek(0); }

  /* ── 2D デモツアー ─────────────────────────────────────── */
  setDemoTour(step: number, active: boolean): void { this.patch({ demoTourStep: step, demoTourActive: active }); }
}

export const bus = new Spine();
