/**
 * scene-host.ts — 永続化 3D ステージ (3D / 4D / 5D 共通)
 *
 * spec の要求:
 *   - 4D: "在**同一个 3D** Graph/Scene 上，以逻辑时间驱动 Case"
 *   - 5D: "可干预且可新增互动元素的因果 4D，**必须复用 4D 引擎与 3D 场景**"
 *
 * したがって 3D シーンは「3D 次元の持ち物」ではなく、**シーン本身が 1 個だけ**存在し、
 * 3D / 4D / 5D はその**ビュー**に過ぎない。
 * 切替時に WebGL コンテキストを作り直さないよう、canvas 要素は保持したまま
 * DOM 上の親だけ差し替える。
 *
 * 役割分担:
 *   SceneHost  … 3D 形状・カメラ・materials・state 適用・focus 退避・ピッキング
 *   d3.ts      … 12 View / 12 ナビゲーション / ラベルモードの UI
 *   d4.ts      … 4D エンジン実行 + タイムライン + **ステージの state 適用**
 *   d5.ts      … A/B 因果 + **ステージの branch state 適用** + overlay/split
 */

import * as THREE from 'three';
import { LeftHandAdapter } from '../three/adapter';
import {
  gemGeometry, ringGeometry, slabGeometry, wallGeometry, portGeometry, arrowGeometry,
  disposeGemCache, type GemRole,
} from '../three/gems';
import { buildEdgeCurve, solidTube, dashedTube, orientArrow } from '../three/edges';
import { LabelLayer, type LabelMode, type LabelTarget, type LabelPriority } from '../three/labels';
import { buildAtlasModel, type AtlasModel, type NodeInfo } from '../three/model';
import { viewById, type ViewId } from '../three/views';
import { DOMAIN_COLOR, RING_STATE_COLOR } from './tokens';
import type { ArchitectureModel, SpatialModel } from './types';
import type { CausalRuleInfo, CaseInfo } from '../three/model';

export interface StagePickResult { kind: 'node' | 'edge' | 'port' | 'room'; id: string }

const STATE_COLORS: Record<string, string> = {
  Idle: '#5A6C85', Running: '#5AC7C3', Waiting: '#F0BA63',
  Completed: '#74CA9D', Failed: '#F08886', Recovered: '#82ADFA', Unknown: '#81909F',
};

export class SceneHost {
  readonly adapter = new LeftHandAdapter();
  readonly atlas: AtlasModel;

  private renderer: THREE.WebGLRenderer | null = null;
  private scene = new THREE.Scene();
  private camera = new THREE.PerspectiveCamera(50, 1, 0.1, 4000);
  private canvas: HTMLCanvasElement | null = null;
  private canvasHost: HTMLElement | null = null;

  private nodeMeshes = new Map<string, THREE.Mesh>();
  private ringMeshes = new Map<string, THREE.Mesh>();
  private roomMeshes = new Map<string, THREE.Mesh>();
  private edgeMeshes = new Map<string, THREE.Mesh>();
  private labels: LabelLayer | null = null;
  private labelHost: HTMLElement | null = null;

  /** 現在表示中のビュー (3D と同じ 12 種) */
  view: ViewId = 'Overview';
  /** ラベル表示モード */
  labelMode: LabelMode = 'Smart';
  /** 選択ノード */
  selected: { kind: string; id: string } | null = null;
  /** focus retreat の対象集合 (null = なし) */
  focus: Set<string> | null = null;
  /** ラベル優先度の上書き (因果モード用) */
  private priorityOverride = new Map<string, LabelPriority>();

  /** カメラ状態 */
  private az = 0.62; private el = 0.44; private dist = 270;
  private target = new THREE.Vector3();
  private rafId: number | null = null;
  private resizeObs: ResizeObserver | null = null;

  constructor(
    arch: ArchitectureModel, idx: { nodes: Map<string, unknown> },
    spatial: SpatialModel, rules: CausalRuleInfo[], cases: CaseInfo[],
  ) {
    this.atlas = buildAtlasModel(arch, idx as never, spatial, rules, cases);
    this.buildScene();
  }

  /* ── シーン構築 ─────────────────────────────────────────── */
  private buildScene(): void {
    this.scene.add(new THREE.AmbientLight(0xffffff, 0.55));
    const key = new THREE.DirectionalLight(0xffffff, 0.9);
    key.position.set(60, 90, 70); this.scene.add(key);
    const rim = new THREE.DirectionalLight(0x5ac7c3, 0.35);
    rim.position.set(-70, 40, -50); this.scene.add(rim);

    // 部屋 (スラブ)
    for (const room of this.atlas.rooms) {
      const geo = slabGeometry(this.adapter);
      const mat = new THREE.MeshStandardMaterial({
        color: new THREE.Color(DOMAIN_COLOR[room.domainId] ?? DOMAIN_COLOR.Unknown).multiplyScalar(0.28),
        transparent: true, opacity: 0.42, roughness: 0.85, metalness: 0.05,
      });
      const mesh = new THREE.Mesh(geo, mat);
      mesh.position.copy(this.adapter.toThree(room.center.x, room.center.y, room.center.z));
      mesh.scale.set(room.bounds.w, room.bounds.h, room.bounds.d);
      mesh.userData = { kind: 'room', id: room.id, name: room.name };
      this.scene.add(mesh);
      this.roomMeshes.set(room.id, mesh);
    }

    // フロア壁
    for (const floor of this.atlas.floors) {
      const geo = wallGeometry(this.adapter);
      const mat = new THREE.MeshStandardMaterial({
        color: 0x40536a, transparent: true, opacity: 0.16, roughness: 1, metalness: 0,
      });
      const mesh = new THREE.Mesh(geo, mat);
      mesh.position.copy(this.adapter.toThree(0, floor.y - 6, 0));
      mesh.scale.set(150, 1, 110);
      mesh.userData = { kind: 'floor', id: floor.id, name: floor.name };
      this.scene.add(mesh);
    }

    // ノード (宝石) + リング
    for (const n of this.atlas.nodes.values()) {
      const geo = gemGeometry(n.role, this.adapter);
      const mat = new THREE.MeshStandardMaterial({
        color: new THREE.Color(n.color), roughness: 0.35, metalness: 0.55,
        emissive: new THREE.Color(n.color).multiplyScalar(0.18),
      });
      const mesh = new THREE.Mesh(geo, mat);
      mesh.position.copy(this.adapter.toThree(n.position.x, n.position.y, n.position.z));
      mesh.scale.setScalar(Math.max(1.2, n.radius));
      mesh.userData = { kind: 'node', id: n.id, name: n.display };
      this.scene.add(mesh);
      this.nodeMeshes.set(n.id, mesh);

      const ring = new THREE.Mesh(ringGeometry(this.adapter),
        new THREE.MeshBasicMaterial({ color: new THREE.Color(STATE_COLORS.Idle) }));
      ring.position.copy(mesh.position);
      ring.scale.setScalar(Math.max(1.2, n.radius) * 1.28);
      ring.userData = { kind: 'ring', id: n.id };
      this.scene.add(ring);
      this.ringMeshes.set(n.id, ring);
    }

    // ポート
    for (const p of this.atlas.portIndex.values()) {
      const mesh = new THREE.Mesh(portGeometry(this.adapter),
        new THREE.MeshStandardMaterial({ color: 0xaab9cb, roughness: 0.5, metalness: 0.3 }));
      mesh.position.copy(this.adapter.toThree(p.position.x, p.position.y, p.position.z));
      mesh.scale.setScalar(0.55);
      mesh.userData = { kind: 'port', id: p.id, name: p.port.name };
      this.scene.add(mesh);
    }

    // エッジ (チューブ + 矢印)
    for (const e of this.atlas.edges) {
      const src = this.atlas.nodes.get(e.sourceId);
      const tgt = this.atlas.nodes.get(e.targetId);
      if (!src || !tgt) continue;
      const ec = buildEdgeCurve(this.adapter, e, src, tgt);
      if (e.designedOnly) {
        for (const geo of dashedTube(ec.curve, 0.22, 12, 0.55)) {
          const mesh = new THREE.Mesh(geo, new THREE.MeshStandardMaterial({
            color: 0x81909f, roughness: 0.6, metalness: 0.2, transparent: true, opacity: 0.7,
          }));
          mesh.userData = { kind: 'edge', id: e.id, name: e.label };
          this.scene.add(mesh);
          this.edgeMeshes.set(e.id, mesh);
        }
      } else {
        const mesh = new THREE.Mesh(solidTube(ec.curve, 0.18, 48),
          new THREE.MeshStandardMaterial({ color: 0x40536a, roughness: 0.6, metalness: 0.2 }));
        mesh.userData = { kind: 'edge', id: e.id, name: e.label };
        this.scene.add(mesh);
        this.edgeMeshes.set(e.id, mesh);
      }
      const arrow = new THREE.Mesh(arrowGeometry(this.adapter),
        new THREE.MeshStandardMaterial({ color: e.designedOnly ? 0x81909f : 0x8fa8c8, roughness: 0.5 }));
      orientArrow(arrow, ec.tangent, ec.curve.getPointAt(0.82));
      arrow.scale.setScalar(0.9);
      arrow.userData = { kind: 'edge', id: e.id, name: e.label };
      this.scene.add(arrow);
      this.edgeMeshes.set(`${e.id}::arrow`, arrow);
    }

    // 初期カメラ注視点
    const box = new THREE.Box3();
    for (const m of this.nodeMeshes.values()) box.expandByObject(m);
    if (!box.isEmpty()) this.target.copy(box.getCenter(new THREE.Vector3()));
  }

  /* ── マウント (canvas は保持し、親だけ差し替える) ─────────── */
  attachTo(container: HTMLElement): boolean {
    if (!this.renderer) {
      const probe = document.createElement('canvas');
      const gl = probe.getContext('webgl2') ?? probe.getContext('webgl');
      if (!gl) return false;
      try {
        this.renderer = new THREE.WebGLRenderer({ antialias: true, alpha: false, canvas: probe });
      } catch { return false; }
      this.renderer.setPixelRatio(Math.min(2, window.devicePixelRatio || 1));
      this.renderer.setClearColor(0x0b1220, 1);
      this.canvas = probe;
      probe.style.cssText = 'width:100%;height:100%;display:block;';
      this.canvasHost = document.createElement('div');
      this.canvasHost.style.cssText = 'position:absolute;inset:0;';
      this.canvasHost.appendChild(probe);
      this.labelHost = document.createElement('div');
      this.labelHost.style.cssText = 'position:absolute;inset:0;pointer-events:none;';
      this.canvasHost.appendChild(this.labelHost);
      this.labels = new LabelLayer(this.labelHost, (kind, id) => {
        this.onLabelSelect?.(kind as 'node' | 'edge' | 'port' | 'room', id);
      });
      this.resizeObs = new ResizeObserver(() => this.resize());
      this.resizeObs.observe(this.canvasHost);
    }
    // 既に別の親 firefighter にある場合は移動する (DOM 移動のみ。WebGL コンテキストは維持)
    if (this.canvasHost!.parentElement !== container) container.appendChild(this.canvasHost!);
    this.resize();
    this.start();
    return true;
  }

  detach(): void {
    // canvas は破棄しない (4D/5D が同じシーンを使うため)
    this.stop();
  }

  onLabelSelect: ((kind: 'node' | 'edge' | 'port' | 'room', id: string) => void) | null = null;

  /** キャンバス要素 (3D/4D/5D が共有) */
  canvasElement(): HTMLCanvasElement | null { return this.canvas; }

  /* ── 状態適用 (4D/5D から呼ばれる) ─────────────────────── */
  /** ノード ID → ランタイム状態 をリング色と emissive に反映 */
  applyStates(states: Record<string, string>): void {
    for (const [id, st] of Object.entries(states)) {
      const ring = this.ringMeshes.get(id);
      if (!ring) continue;
      const mat = ring.material as THREE.MeshBasicMaterial;
      mat.color.set(STATE_COLORS[st] ?? STATE_COLORS.Unknown);
      const node = this.nodeMeshes.get(id);
      if (node) {
        const nm = node.material as THREE.MeshStandardMaterial;
        nm.emissive.set((STATE_COLORS[st] ?? STATE_COLORS.Unknown));
        nm.emissiveIntensity = st === 'Idle' ? 0.1 : 0.5;
      }
    }
    this.requestRender();
  }

  /** 現在のノード状態をクリア (4D/5D 切替時) */
  clearStates(): void {
    for (const [id, ring] of this.ringMeshes) {
      const mat = ring.material as THREE.MeshBasicMaterial;
      mat.color.set(STATE_COLORS.Idle);
      const node = this.nodeMeshes.get(id);
      if (node) (node.material as THREE.MeshStandardMaterial).emissiveIntensity = 0.18;
    }
    this.requestRender();
  }

  /**
   * focus retreat — 関連しない対象を退避・減光。
   * **視覚状態だけ**を変え、モデルや実行状態は変えない (spec focus_retreat)。
   */
  applyFocus(set: Set<string> | null): void {
    this.focus = set;
    for (const [id, mesh] of this.nodeMeshes) {
      const related = !set || set.has(id);
      const mat = mesh.material as THREE.MeshStandardMaterial;
      mat.transparent = !related;
      mat.opacity = related ? 1 : 0.16;
      mat.depthWrite = related;
    }
    for (const [id, ring] of this.ringMeshes) {
      const related = !set || set.has(id);
      (ring.material as THREE.MeshBasicMaterial).opacity = related ? 1 : 0.12;
      (ring.material as THREE.MeshBasicMaterial).transparent = !related;
    }
    this.requestRender();
  }

  /** 因果モードでラベル優先度を上書き (5D の分岐ノードを強調) */
  setPriorityOverride(m: Map<string, LabelPriority>): void {
    this.priorityOverride = m;
    this.requestRender();
  }

  /* ── 軸ギズモ (共有ステージの一部) ───────────────────────── */
  /**
   * 軸ベクトルは**必ず Adapter を通す**。
   * 素の THREE.Vector3 だと semantic +Z が Three +Z(= semantic -Z) を指してしまい、
   * 「+Z」の表示が実際と逆向きになる (对抗審査で発見)。
   */
  private gizmo: THREE.Group | null = null;
  private gizmoVisible = false;

  private buildGizmo(): void {
    const g = new THREE.Group();
    const mk = (color: number, dir: THREE.Vector3, label: string) => {
      const m = new THREE.Mesh(new THREE.ConeGeometry(2.2, 7, 12),
        new THREE.MeshBasicMaterial({ color }));
      m.position.copy(dir.clone().multiplyScalar(12));
      m.lookAt(0, 0, 0);
      m.rotateX(Math.PI / 2);
      m.userData = { axis: label };
      g.add(m);
    };
    mk(0xf08886, this.adapter.toThree(1, 0, 0), '+X');
    mk(0x74ca9d, this.adapter.toThree(0, 1, 0), '+Y');
    mk(0x82adfa, this.adapter.toThree(0, 0, 1), '+Z');
    g.visible = false;
    this.gizmo = g;
    this.scene.add(g);
  }

  toggleGizmo(): boolean {
    if (!this.gizmo) this.buildGizmo();
    this.gizmoVisible = !this.gizmoVisible;
    if (this.gizmo) this.gizmo.visible = this.gizmoVisible;
    this.requestRender();
    return this.gizmoVisible;
  }

  isGizmoVisible(): boolean { return this.gizmoVisible; }

  /* ── 因果提示 (5D) ───────────────────────────────────────── */
  /**
   * 5D の Overlay / Split を**実描画**に落とす。
   *  - overlay: 分岐ノードを強調 (リング色 + 発光)
   * - split  : 1 つのキャンバスを左右に区切り、左=ベースライン、右=分岐を
   *             **別々の状態で 2 パス描画**する (scissor テスト)。
   *             同一のメッシュ集合を 2 度描画するので、2 つの「世界」が
   *             同じ空間に重なるのではなく左右に並ぶ。
   */
  setCausalPresentation(p: {
    mode: 'none' | 'overlay' | 'split';
    baseStates: Record<string, string>;
    branchStates: Record<string, string>;
    divergedNodeIds: string[];
  }): void {
    this.causal = p;
    if (p.mode === 'split') {
      // 左右に分ける場合、注視点は中央 slightly 引く
      this.requestRender();
      return;
    }
    // overlay / none は 1 パス
    this.applyStates(p.mode === 'overlay' ? p.branchStates : p.baseStates);
    if (p.mode === 'overlay') {
      for (const id of p.divergedNodeIds) {
        const ring = this.ringMeshes.get(id);
        if (ring) (ring.material as THREE.MeshBasicMaterial).color.set('#F0BA63');
        const node = this.nodeMeshes.get(id);
        if (node) {
          const nm = node.material as THREE.MeshStandardMaterial;
          nm.emissive.set('#F0BA63'); nm.emissiveIntensity = 0.9;
        }
      }
    }
    this.requestRender();
  }

  private causal: { mode: 'none' | 'overlay' | 'split'; baseStates: Record<string, string>; branchStates: Record<string, string>; divergedNodeIds: string[] } =
    { mode: 'none', baseStates: {}, branchStates: {}, divergedNodeIds: [] };

  /* ── ビュー / カメラ ────────────────────────────────────── */
  setView(id: ViewId): void {
    this.view = id;
    this.applyViewVisibility();
    this.requestRender();
  }

  applyViewVisibility(): void {
    const v = viewById(this.view);
    const c = {
      model: this.atlas,
      activeFloorId: null, activeRoomId: null,
      selectedNodeId: this.selected?.id ?? null,
      focusSet: this.focus,
    } as never;
    for (const n of this.atlas.nodes.values()) {
      const visible = v.nodeVisible(n, c);
      const m = this.nodeMeshes.get(n.id);
      const r = this.ringMeshes.get(n.id);
      if (m) m.visible = visible;
      if (r) r.visible = visible;
    }
    for (const room of this.atlas.rooms) {
      const m = this.roomMeshes.get(room.id);
      if (m) m.visible = v.roomVisible(room, c);
    }
    for (const [id, m] of this.edgeMeshes) {
      const e = this.atlas.edges.find((x) => x.id === id || `${x.id}::arrow` === id);
      if (!e) continue;
      m.visible = v.edgeVisible(e, c);
    }
  }

  setCamera(az: number, el: number, dist: number, target?: THREE.Vector3): void {
    this.az = az; this.el = el; this.dist = dist;
    if (target) this.target.copy(target);
    this.applyCamera();
    this.requestRender();
  }
  getCamera(): { az: number; el: number; dist: number } {
    return { az: this.az, el: this.el, dist: this.dist };
  }
  orbit(dAz: number, dEl: number): void {
    this.az += dAz;
    this.el = Math.max(-1.4, Math.min(1.4, this.el + dEl));
    this.applyCamera(); this.requestRender();
  }
  pan(dx: number, dy: number): void {
    this.target.x -= dx * this.dist * 0.0015;
    this.target.y += dy * this.dist * 0.0015;
    this.applyCamera(); this.requestRender();
  }
  zoom(f: number): void {
    this.dist = Math.max(18, Math.min(600, this.dist * f));
    this.applyCamera(); this.requestRender();
  }
  private applyCamera(): void {
    this.camera.position.set(
      this.target.x + this.dist * Math.cos(this.el) * Math.sin(this.az),
      this.target.y + this.dist * Math.sin(this.el),
      this.target.z + this.dist * Math.cos(this.el) * Math.cos(this.az),
    );
    this.camera.lookAt(this.target);
    this.camera.updateMatrixWorld();
  }
  resetCamera(): void {
    const box = new THREE.Box3();
    for (const m of this.nodeMeshes.values()) box.expandByObject(m);
    if (!box.isEmpty()) this.target.copy(box.getCenter(new THREE.Vector3()));
    this.az = 0.62; this.el = 0.44; this.dist = 270;
    this.applyCamera(); this.requestRender();
  }
  focusNode(id: string): void {
    const n = this.atlas.nodes.get(id);
    if (!n) return;
    this.target.copy(this.adapter.toThree(n.position.x, n.position.y, n.position.z));
    this.dist = Math.max(40, this.dist * 0.6);
    this.applyCamera(); this.requestRender();
  }

  /* ── ピッキング ─────────────────────────────────────────── */
  private ray = new THREE.Raycaster();
  private ndc = new THREE.Vector2();
  pick(clientX: number, clientY: number): StagePickResult | null {
    if (!this.canvas) return null;
    const rect = this.canvas.getBoundingClientRect();
    this.ndc.x = ((clientX - rect.left) / rect.width) * 2 - 1;
    this.ndc.y = -((clientY - rect.top) / rect.height) * 2 + 1;
    this.ray.setFromCamera(this.ndc, this.camera);
    const hits = this.ray.intersectObjects(this.scene.children, true);
    for (const h of hits) {
      const ud = h.object.userData as { kind?: string; id?: string };
      if (!ud?.kind || !ud.id) continue;
      if (ud.kind === 'ring') continue;
      if (ud.kind === 'node' || ud.kind === 'port' || ud.kind === 'edge' || ud.kind === 'room') {
        return { kind: ud.kind, id: ud.id };
      }
    }
    return null;
  }

  /* ── ラベル ─────────────────────────────────────────────── */
  private buildLabelTargets(): LabelTarget[] {
    const out: LabelTarget[] = [];
    const prio = (id: string): LabelPriority => {
      const o = this.priorityOverride.get(id);
      if (o) return o;
      if (this.selected?.id === id) return 'Selected';
      if (this.focus?.has(id)) return 'DirectlyRelated';
      if (this.selected?.id && this.focus) return 'Context';
      return 'Unrelated';
    };
    for (const n of this.atlas.nodes.values()) {
      const m = this.nodeMeshes.get(n.id);
      if (m && !m.visible) continue;
      out.push({
        id: n.id, kind: 'node', text: n.display,
        sub: `${n.node.domainId} · ${n.ring}${n.metricLabel ? ` · ${n.metricLabel}=${n.metricValue}` : ''}`,
        world: this.adapter.toThree(n.position.x, n.position.y, n.position.z),
        radius: Math.max(1.2, n.radius), priority: prio(n.id),
      });
    }
    for (const e of this.atlas.edges) {
      const src = this.atlas.nodes.get(e.sourceId);
      const tgt = this.atlas.nodes.get(e.targetId);
      if (!src || !tgt) continue;
      const em = this.edgeMeshes.get(e.id);
      if (em && !em.visible) continue;
      out.push({
        id: `edge:${e.id}`, kind: 'edge', text: e.label,
        sub: e.designedOnly ? 'Inferred (注释/字符串内のみ)' : 'Implemented (実使用点)',
        world: this.adapter.toThree(
          (src.position.x + tgt.position.x) / 2,
          (src.position.y + tgt.position.y) / 2,
          (src.position.z + tgt.position.z) / 2,
        ),
        radius: 1.4,
        priority: this.selected?.id === e.sourceId || this.selected?.id === e.targetId
          ? 'DirectlyRelated' : 'Unrelated',
      });
    }
    return out;
  }

  setLabelMode(m: LabelMode): void { this.labelMode = m; this.requestRender(); }

  /* ── レンダリング ───────────────────────────────────────── */
  private resize(): void {
    if (!this.renderer || !this.canvasHost) return;
    const w = this.canvasHost.clientWidth || 1;
    const h = this.canvasHost.clientHeight || 1;
    this.renderer.setSize(w, h, false);
    this.camera.aspect = w / h;
    this.camera.updateProjectionMatrix();
    this.requestRender();
  }
  private renderFrame(): void {
    if (!this.renderer) return;
    const w = this.canvasHost?.clientWidth || 1;
    const h = this.canvasHost?.clientHeight || 1;
    if (this.labels) {
      const targets = this.buildLabelTargets();
      this.labels.setTargets(targets);
      this.labels.update(this.camera, w, h, this.labelMode, new Set(targets.map((t) => t.id)));
    }
    if (this.causal.mode === 'split') {
      /* 左右 2 パス: 左=ベースライン、右=分岐 */
      const half = Math.floor(w / 2);
      this.renderer.setScissorTest(true);
      // 左 (ベースライン)
      this.applyStates(this.causal.baseStates);
      this.renderer.setViewport(0, 0, half, h);
      this.renderer.setScissor(0, 0, half, h);
      this.renderer.render(this.scene, this.camera);
      // 右 (分岐)
      this.applyStates(this.causal.branchStates);
      for (const id of this.causal.divergedNodeIds) {
        const ring = this.ringMeshes.get(id);
        if (ring) (ring.material as THREE.MeshBasicMaterial).color.set('#F0BA63');
      }
      this.renderer.setViewport(half, 0, w - half, h);
      this.renderer.setScissor(half, 0, w - half, h);
      this.renderer.render(this.scene, this.camera);
      this.renderer.setScissorTest(false);
      this.renderer.setViewport(0, 0, w, h);
      return;
    }
    this.renderer.render(this.scene, this.camera);
  }
  private pending = false;
  requestRender(): void {
    if (this.pending) return;
    this.pending = true;
    requestAnimationFrame(() => { this.pending = false; this.renderFrame(); });
  }
  private start(): void {
    if (this.rafId !== null) return;
    const loop = () => { this.renderFrame(); this.rafId = requestAnimationFrame(loop); };
    this.rafId = requestAnimationFrame(loop);
  }
  private stop(): void {
    if (this.rafId !== null) { cancelAnimationFrame(this.rafId); this.rafId = null; }
  }

  /** 破棄 (アプリ終了時のみ) */
  dispose(): void {
    this.stop();
    this.resizeObs?.disconnect();
    this.labels?.dispose();
    disposeGemCache();
    this.scene.traverse((o) => {
      const m = o as THREE.Mesh;
      if (m.geometry) m.geometry.dispose();
      const mat = m.material as THREE.Material | THREE.Material[] | undefined;
      if (Array.isArray(mat)) mat.forEach((x) => x.dispose());
      else if (mat) mat.dispose();
    });
    this.renderer?.dispose();
    this.renderer = null;
  }
}

/* ── プロセス内で 1 個だけ存在するステージ ─────────────────────────── */
let instance: SceneHost | null = null;
export function getSceneHost(
  arch: ArchitectureModel, idx: { nodes: Map<string, unknown> },
  spatial: SpatialModel, rules: CausalRuleInfo[], cases: CaseInfo[],
): SceneHost {
  if (!instance) instance = new SceneHost(arch, idx, spatial, rules, cases);
  return instance;
}
export function hasSceneHost(): boolean { return instance !== null; }
export function disposeSceneHost(): void { instance?.dispose(); instance = null; }
export type { NodeInfo, GemRole };
