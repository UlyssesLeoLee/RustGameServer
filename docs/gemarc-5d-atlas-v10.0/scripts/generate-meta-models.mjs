#!/usr/bin/env node
/**
 * generate-meta-models.mjs — 生成规格派生的 4 个元模型
 *
 * 这 4 个文件编码的是**规格结构**（不是仓库数据），所以是静态派生而非扫描产出:
 *   - models/schema.json               : 模型的 JSON Schema (GraphModel / Prompt / Case / Intervention ...)
 *   - models/diagram-definitions.json   : 14 种专业图的定义与渲染器绑定
 *   - models/information-placement.yaml : 信息放置矩阵 (spec source_of_truth)
 *   - models/scenario-deltas.json       : 干预/场景增量的目录与模板
 */

import { writeFileSync, readFileSync, existsSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, '..');
const M = join(ROOT, 'models');

/* ── 1. schema.json ──────────────────────────────────────────────── */
const schema = {
  $schema: 'https://json-schema.org/draft/2020-12/schema',
  $id: 'https://local/gemarc-5d-atlas/v10/schema.json',
  title: 'Universal GemArc 5D Design Atlas — Canonical Engineering Model',
  description: 'spec `canonical_engineering_model` の機械可読な定義。分割_models は個別に版管理する。',
  $defs: {
    ExistenceStatus: { enum: ['Present', 'Absent', 'Unknown'] },
    ImplementationStatus: { enum: ['Implemented', 'Designed', 'Inferred', 'Absent'] },
    EvidenceKind: { enum: ['SourceCode', 'TestTrace', 'RuntimeLog', 'Document', 'PublicObservation', 'UserRequirement'] },
    ScenarioOrigin: { enum: ['ObservedReplay', 'DesignedSimulation', 'HypotheticalExperiment'] },
    QAStatus: { enum: ['PASS', 'FAIL', 'NOT_TESTED', 'DEGRADED', 'BLOCKED', 'FLAKY_UNRESOLVED'] },
    Evidence: {
      type: 'object', required: ['id', 'kind', 'fileLine', 'path', 'startLine', 'endLine', 'revision', 'note'],
      properties: {
        id: { type: 'string', pattern: '^EV-\\d{4}$' },
        kind: { $ref: '#/$defs/EvidenceKind' },
        fileLine: { type: 'string', description: 'path:startLine-endLine' },
        path: { type: 'string' },
        startLine: { type: 'integer', minimum: 1 },
        endLine: { type: 'integer', minimum: 1 },
        revision: { type: 'string', description: '采集时的仓库 revision (短 SHA)' },
        note: { type: 'string' },
        snippet: { type: 'string' },
      },
    },
    GraphNode: {
      type: 'object',
      required: ['id', 'kind', 'name', 'qualifiedName', 'role', 'domainId', 'parentId', 'portIds', 'properties', 'verification', 'evidenceIds'],
      properties: {
        id: { type: 'string' }, kind: { type: 'string' }, name: { type: 'string' },
        qualifiedName: { type: 'string' }, role: { type: 'string' },
        domainId: { type: 'string', enum: ['Core', 'Interaction', 'Data', 'Security', 'Intelligence', 'Infrastructure', 'External', 'Operations', 'Unknown'] },
        parentId: { type: ['string', 'null'] }, portIds: { type: 'array', items: { type: 'string' } },
        properties: { type: 'object' },
        verification: { $ref: '#/$defs/ImplementationStatus' },
        evidenceIds: { type: 'array', items: { type: 'string' } },
      },
    },
    GraphEdge: {
      type: 'object',
      required: ['id', 'sourceId', 'targetId', 'type', 'directed', 'verification', 'evidenceIds'],
      properties: {
        id: { type: 'string' }, sourceId: { type: ['string', 'null'] }, targetId: { type: ['string', 'null'] },
        sourcePortId: { type: ['string', 'null'] }, targetPortId: { type: ['string', 'null'] },
        type: { type: 'string' }, directed: { type: 'boolean' }, guardId: { type: ['string', 'null'] },
        properties: { type: 'object' },
        verification: { $ref: '#/$defs/ImplementationStatus' },
        evidenceIds: { type: 'array', items: { type: 'string' } },
      },
    },
    Prompt: {
      type: 'object',
      required: ['promptId', 'version', 'objective', 'requirementIds', 'nodeEdgeIds', 'caseIds', 'inputs', 'constraints', 'outputs', 'acceptance', 'evidenceIds'],
      properties: {
        promptId: { type: 'string' }, version: { type: 'string' }, objective: { type: 'string' },
        requirementIds: { type: 'array', items: { type: 'string' } },
        nodeEdgeIds: { type: 'array', items: { type: 'string' } },
        caseIds: { type: 'array', items: { type: 'string' } },
        inputs: { type: 'array' }, constraints: { type: 'array' }, outputs: { type: 'array' }, acceptance: { type: 'array' },
        evidenceIds: { type: 'array', items: { type: 'string' } },
      },
    },
    Case: {
      type: 'object',
      required: ['caseId', 'kind', 'title', 'origin', 'description', 'startNodeId', 'steps', 'expectedOutcome', 'nodeIds', 'evidenceIds'],
      properties: {
        caseId: { type: 'string' },
        kind: { enum: ['Normal', 'Abnormal', 'Boundary', 'Concurrent', 'Delay', 'Timeout', 'DuplicateDelivery', 'OutOfOrder', 'FailurePropagation', 'Retry', 'Recovery', 'Compensation', 'ResourceContention', 'Combined'] },
        title: { type: 'string' }, origin: { $ref: '#/$defs/ScenarioOrigin' },
        description: { type: 'string' }, startNodeId: { type: 'string' },
        precondition: { type: 'string' },
        steps: { type: 'array', items: { type: 'object' } },
        expectedOutcome: { type: 'string' },
        nodeIds: { type: 'array', items: { type: 'string' } },
        edgeIds: { type: 'array', items: { type: 'string' } },
        evidenceIds: { type: 'array', items: { type: 'string' } },
      },
    },
    Intervention: {
      type: 'object', required: ['interventionId', 'kind', 'targetId', 'label', 'origin'],
      properties: {
        interventionId: { type: 'string' },
        kind: { enum: ['DisableNode', 'DisableEdge', 'BypassNode', 'ChangeGuard', 'ChangeTimeout', 'ChangeRetry', 'DelayMessage', 'InjectFault', 'AddInteractiveElement', 'RewireGraph'] },
        targetId: { type: 'string' }, label: { type: 'string' },
        value: {}, payload: { type: 'object' }, origin: { $ref: '#/$defs/ScenarioOrigin' },
      },
    },
    SimulationEvent: {
      type: 'object',
      required: ['eventId', 'caseId', 'logicalTime', 'sequence', 'previousState', 'nextState'],
      properties: {
        eventId: { type: 'string' }, caseId: { type: 'string' },
        logicalTime: { type: 'integer' }, sequence: { type: 'integer' },
        sourceNodeId: { type: ['string', 'null'] }, targetNodeId: { type: ['string', 'null'] },
        edgeId: { type: ['string', 'null'] },
        previousState: { type: ['string', 'null'] }, nextState: { type: ['string', 'null'] },
        ruleId: { type: ['string', 'null'] }, evidenceIds: { type: 'array', items: { type: 'string' } },
      },
    },
  },
  $ref: '#/$defs/GraphNode',
};
writeFileSync(join(M, 'schema.json'), JSON.stringify(schema, null, 2), 'utf8');

/* ── 2. diagram-definitions.json ─────────────────────────────────── */
const D = (type, file, symbols, notes) => ({
  diagramType: type, renderer: `src/diagrams/${file}.ts`, symbols, notes,
  semanticRequirements: notes,
});
const diagramDefs = {
  meta: { version: '10.0.0', generatedBy: 'scripts/generate-meta-models.mjs', note: '各図種は独立レンダラを持ち、全図種を Sequence として描画してはならない。' },
  diagramTypes: [
    D('UML_Class', 'uml-class', ['三区画 (名称/属性/操作)', '可視性記号 (+ - # ~)', '抽象クラスは斜体', '関連: 多重度'],
      'UML 2.x クラス図の区画構成と可視性記号を守る。宝石化しない。'),
    D('UML_Component', 'uml-component', ['コンポーネント矩形', '棒棒糖 (ロリポップ) インタフェース', '装配関係'],
      'UML 2.x コンポーネント図。'),
    D('UML_Package', 'uml-package', ['折りたたみタブ付きパッケージ', 'パッケージ間依存'],
      'UML 2.x パッケージ図。'),
    D('UML_Sequence', 'uml-sequence', ['生命線 (破線)', '適���条 (activation bar)', 'alt / opt / loop / par 組み合わせ枠', '同期/非同期メッセージ'],
      'UML 2.x シーケンス図。alt/opt/loop/par の枠と guard を省略しない。'),
    D('UML_State', 'uml-state', ['角丸状態', '初期状態 (黒点)', '終端状態 (二重円)', '遷移条件 guard'],
      'UML 2.x 状態図。'),
    D('UML_Activity', 'uml-activity', ['角丸アクティビティ', '菱形判断 (guard 付き)', '開始/終了ノード', '泳道'],
      'UML 2.x アクティビティ図。'),
    D('UML_Deployment', 'uml-deployment', ['ノード (3D 箱)', '実行可能制品', '通信路', '配置矩形'],
      'UML 2.x 配置図。'),
    D('UML_UseCase', 'uml-usecase', ['楕円ユースケース', 'アクタ (人型)', '関連 (association)', 'include / extend'],
      'UML 2.x ユースケース図。'),
    D('ER', 'er', ['エンティティ矩形', '関係 (菱形)', '多重度 (1 / 0..* / 1..*)', 'PK は下線', 'FK は破線'],
      ' Barker / Chen 表記の ER。PK/FK 表記は実 DDL から機械判定する。'),
    D('DFD', 'dfd', ['データフロー (開口矩形)', '処理 (丸)', 'データストア (平行線)', '外部エンティティ (矩形)'],
      'DFD の記法。'),
    D('BPMN', 'bpmn', ['プール/レーン', 'イベント (円)', 'ゲートウェイ (菱形)', 'タスク (角丸矩形)', 'メッセージフロー'],
      'BPMN 2.0 の記法。'),
    D('Security', 'security', ['信頼境界 (破線枠)', '主体/客体', '権限マトリクス'],
      'セキュリティ境界の可視化。'),
    D('Network', 'network', ['デバイス', 'リンク', '向き', 'ラベル'],
      'ネットワーク構成。'),
    D('DataFlow', 'dataflow', ['データフロー (矢印)', '変換ノード', 'ストア', '境界'],
      'データフロー図。'),
  ],
  interoperability: {
    exportFormats: ['GraphModel JSON (versioned)', 'Workflow JSON (BPMN-compatible subset)'],
    adapters: [
      { name: 'React Flow', supported: ['nodes', 'edges', 'ports', 'directed/undirected', 'subgraph'], semanticLoss: ['UML 記号の厳密な表現', 'PK/FK 表記', '多重度の制約検証'] },
      { name: 'tldraw', supported: ['nodes', 'edges', 'group', 'styles'], semanticLoss: ['ガード/多重度', 'alt/opt/loop/par 枠', 'PK/FK'] },
      { name: 'BPMN (bpmn-js)', supported: ['pool/lane', 'task', 'gateway', 'sequence flow'], semanticLoss: ['非 BPMN 図種 (UML/ER 等) は変換不可'] },
    ],
    aiPatchPolicy: 'AI は受控 Patch (JSON Patch 限定) でのみ適用。无语义的 diff を直接適用しない。',
  },
};
writeFileSync(join(M, 'diagram-definitions.json'), JSON.stringify(diagramDefs, null, 2), 'utf8');

/* ── 3. information-placement.yaml ──────────────────────────────── */
const placement = `# 情報配置マトリクス (Information Placement Matrix)
#
# spec: requirements_and_information_architecture.source_of_truth
#       "models/information-placement.yaml"
#
# 変更手順: 情報を移動・追加するたびに、本ファイル / ナビゲーションテスト / 設計決定を
# 同時に更新する。複数ラウンドの競合追記は禁止 (spec change_control)。

meta:
  version: "10.0.0"
  subject: "RustGameServer (RGS)"
  source_of_truth: true
  generated_by: "scripts/generate-meta-models.mjs"

placement:
  0D:
    kind: gate
    navigation_page: false
    location: "ヘッダーゲート指示器 → モーダル"
    content: "存在性マトリクス (Present / Absent / Unknown)"
    prohibition: "独立ページを作らない / Unknown を Absent に畳まない / 装飾的アーキテクチャ図を作らない"

  1D:
    kind: page
    navigation_page: true
    location: "ヘッダー タブ「1D」"
    content: "プロンプトの生成・編集・検証・コピー・エクスポート・版差分・証拠リンク"
    actions: [generate, edit, validate, copy, export, diff, link_evidence]
    prohibition: "図を描かない (1D はプロンプトである)"

  2D:
    kind: page
    navigation_page: true
    location: "ヘッダー タブ「2D」"
    content: "全設計要素の無限キャンバス + 独立した専門図のレビュー単位"
    canvas: [pan_zoom, minimap, search, layer, filter, focus, group, collapse, multiselect, undo_redo,
             node_edge_port_independent_select, relation_and_requirement_trace]
    diagrams: [UML_Class, UML_Component, UML_Package, UML_Sequence, UML_State, UML_Activity,
               UML_Deployment, UML_UseCase, ER, DFD, BPMN, Security, Network, DataFlow]
    prohibition: "全図種を Sequence として描画しない / 宝石で専門記号を描かない"

  prototype:
    kind: workspace
    navigation_page: true
    location: "ヘッダー タブ「Prototype」"
    content: "複数タブのプロダクトプロトタイプ (独立ワークスペース)"
    sub_tabs: [ページとスクリーン, ビジュアル編集, インタラクションプレビュー, 状態とCase, コンポーネントライブラリ, 設計とテスト]
    prohibition: "第六次元ではない / 2D 無限キャンバスと同一視しない / 空メニューにしない"

  3D:
    kind: page
    navigation_page: true
    location: "ヘッダー タブ「3D」"
    content: "意味的ビルディング / フロア / ルーム / ノード / エッジの 3D 空間グラフ"
    views: [Overview, BirdEye, ExplodedFloor, RoomCutaway, Logical, Development, Deployment, Data, Process, Security, Operations, Dependency]
    navigation: [OrbitYaw, OrbitPitch, Pan, Zoom, Focus, AxisGizmo, RoomSwitch, FloorSwitch, ResetCamera]
    prohibition: "仮想空間を物理配備の事実として偽装しない / 宝石を 3D ノード以外に使わない"

  4D:
    kind: page
    navigation_page: true
    location: "ヘッダー タブ「4D」 + 下部タイムライン"
    content: "同一 3D 上の論理時間 / イベント / 状態 / Case アニメーション"
    controls: [Play, Pause, StepForward, StepBack, Seek, Speed, Reset, Snapshot, Replay, CaseSwitch]
    prohibition: "描画 FPS が結果を決めない (Event Trace のみを再生)"

  5D:
    kind: page
    navigation_page: true
    location: "ヘッダー タブ「5D」 + 下部タイムライン"
    content: "同じ動的 3D 上でモジュール無効化 / 規則変更 / 相互作用要素追加 / A-B 因果説明"
    operations: [DisableNode, DisableEdge, BypassNode, ChangeGuard, ChangeTimeout, ChangeRetry,
                 DelayMessage, InjectFault, AddInteractiveElement, RewireGraph]
    modes: [Overlay3D, SplitSynchronized3D, BranchTimeline]
    essential: [SharedPath, FirstDivergence, BranchA, BranchB, Propagation, Blocking, Feedback, Retry,
                Recovery, Compensation, Convergence, OutcomeDiff, NoDifference]
    prohibition:
      - "事前計算の光跡を因果として偽装しない"
      - "色を変えるだけで再計算しない"
      - "装飾ノードを置いただけamente干渉成功と主張しない"
      - "実験で正式な事実グラフを変更しない"

view_independence:
  note: "Logical / Development / Process / Deployment / Data / Security / Operations / Dependency の各 View は 1D-5D の表現次元とは独立して定義する。"
  views: [Logical, Development, Process, Deployment, Data, Security, Operations, Dependency]

trace_chain: "Requirement ↔ Prompt ↔ BD ↔ DD ↔ Screen/Node/Edge/Port ↔ API/DB/Event ↔ Case ↔ UT/IT/ST/UAT"

immutable_ids:
  note: "Requirement, Prompt, Node, Edge, Port, Diagram, Screen, Component, Action, Event, Rule, Case, Test はそれぞれ安定 ID を持ち、次元間で再採番しない。"
  entities: [Requirement, Prompt, Node, Edge, Port, Diagram, Screen, Component, Action, Event, Rule, Case, Test]
`;
writeFileSync(join(M, 'information-placement.yaml'), placement, 'utf8');

/* ── 4. scenario-deltas.json ────────────────────────────────────── */
const deltas = {
  meta: {
    version: '10.0.0',
    generatedBy: 'scripts/generate-meta-models.mjs',
    isolation: 'すべての追加/遮蔽/変更は ScenarioGraphDelta に属する。正式な事実グラフは不変。実験の採用は設計変更提案を経由する。',
  },
  customElementTemplates: [
    { template: 'Cache', kind: 'Processor', typedInputPorts: ['request'], typedOutputPorts: ['hit', 'miss'], stateMachine: ['Idle', 'Running', 'Completed'], guardAst: 'cache.contains(key)', eventHandlers: ['onRequest'], errorRules: ['onCapacityExceeded'], resourceLimits: { maxEntries: 10000 }, terminationCondition: 'single response' },
    { template: 'Validator', kind: 'Processor', typedInputPorts: ['payload'], typedOutputPorts: ['valid', 'invalid'], stateMachine: ['Idle', 'Running', 'Completed'], guardAst: 'schema.valid(payload)', eventHandlers: ['onPayload'], errorRules: ['onSchemaViolation'], resourceLimits: {}, terminationCondition: 'single verdict' },
    { template: 'Proxy', kind: 'Processor', typedInputPorts: ['request'], typedOutputPorts: ['response', 'error'], stateMachine: ['Idle', 'Running', 'Waiting', 'Completed', 'Failed'], guardAst: 'upstream.healthy', eventHandlers: ['onRequest', 'onUpstreamError'], errorRules: ['onTimeout'], resourceLimits: { timeoutMs: 5000 }, terminationCondition: 'single response' },
    { template: 'MessageQueue', kind: 'Processor', typedInputPorts: ['message'], typedOutputPorts: ['ack', 'nack'], stateMachine: ['Idle', 'Running', 'Waiting', 'Completed'], guardAst: 'queue.hasCapacity', eventHandlers: ['onMessage', 'onAck', 'onNack'], errorRules: ['onPoisonMessage'], resourceLimits: { maxDepth: 1000 }, terminationCondition: 'ack or nack' },
    { template: 'CircuitBreaker', kind: 'Processor', typedInputPorts: ['call'], typedOutputPorts: ['pass', 'shortCircuit'], stateMachine: ['Idle', 'Running', 'Waiting', 'Completed', 'Failed', 'Recovered'], guardAst: 'state != Open', eventHandlers: ['onCall', 'onSuccess', 'onFailure'], errorRules: ['onThreshold'], resourceLimits: { failureThreshold: 5, openDurationMs: 30000 }, terminationCondition: 'pass or shortCircuit' },
    { template: 'FaultSource', kind: 'Actor', typedInputPorts: ['trigger'], typedOutputPorts: ['fault'], stateMachine: ['Idle', 'Running', 'Completed'], guardAst: 'enabled', eventHandlers: ['onTrigger'], errorRules: [], resourceLimits: {}, terminationCondition: 'single fault' },
    { template: 'RetryHandler', kind: 'Processor', typedInputPorts: ['call'], typedOutputPorts: ['result', 'giveUp'], stateMachine: ['Idle', 'Running', 'Waiting', 'Completed', 'Failed'], guardAst: 'isRetryable(error.code)', eventHandlers: ['onCall', 'onError'], errorRules: ['onMaxRetries'], resourceLimits: { maxRetries: 3, initialIntervalMs: 100, multiplier: 2, maxIntervalMs: 30000, jitter: 0.2 }, terminationCondition: 'result or giveUp' },
    { template: 'RecoveryHandler', kind: 'Processor', typedInputPorts: ['state'], typedOutputPorts: ['recovered', 'stillFailed'], stateMachine: ['Idle', 'Running', 'Waiting', 'Recovered', 'Failed'], guardAst: 'probe.allowed', eventHandlers: ['onProbe', 'onProbeSuccess', 'onProbeFailure'], errorRules: ['onProbeFailure'], resourceLimits: { probeLimit: 1 }, terminationCondition: 'recovered or stillFailed' },
    { template: 'ExternalStub', kind: 'Interface', typedInputPorts: ['request'], typedOutputPorts: ['response'], stateMachine: ['Idle', 'Running', 'Completed', 'Failed'], guardAst: 'stub.available', eventHandlers: ['onRequest'], errorRules: ['onUnavailable'], resourceLimits: {}, terminationCondition: 'single response' },
  ],
  interventionCatalog: [
    { kind: 'DisableNode', targetKind: 'Node', effect: 'ノードを到達不能にする', graphEffect: 'node disabled' },
    { kind: 'DisableEdge', targetKind: 'Edge', effect: 'エッジを削除する', graphEffect: 'edge removed' },
    { kind: 'BypassNode', targetKind: 'Node', effect: 'ノードを迂回する', graphEffect: 'edges rerouted' },
    { kind: 'ChangeGuard', targetKind: 'Rule', effect: 'ガード式を差し替える', graphEffect: 'rule override' },
    { kind: 'ChangeTimeout', targetKind: 'Rule', effect: 'タイムアウトを変更する', graphEffect: 'timing override' },
    { kind: 'ChangeRetry', targetKind: 'Rule', effect: '再試行回数を変更する', graphEffect: 'timing override' },
    { kind: 'DelayMessage', targetKind: 'Node', effect: 'メッセージ遅延を注入する', graphEffect: 'timing override' },
    { kind: 'InjectFault', targetKind: 'Node', effect: '故障を強制する', graphEffect: 'runtime fault (model only)' },
    { kind: 'AddInteractiveElement', targetKind: 'Node', effect: 'A→B を A→X→B に真に改写する', graphEffect: 'edge rewrite' },
    { kind: 'RewireGraph', targetKind: 'Edge', effect: '経路を差し替える', graphEffect: 'edge rewrite' },
  ],
  validation: ['PortTypes', 'Direction', 'Schema', 'Guard', 'EventReachability', 'CycleTermination', 'PermissionBoundary'],
  rewriteContract: '挿入ノードは A→B を本当に A→X→B に改写し、古い Edge の扱いを明示する。三次元方才 设置だけでは完了としない。',
  safetyLimits: { maxEvents: 5000, maxSteps: 2000, maxBranches: 8, maxIterations: 1000, maxMillis: 2000, maxMemoryMb: 64 },
};
writeFileSync(join(M, 'scenario-deltas.json'), JSON.stringify(deltas, null, 2), 'utf8');

console.log(JSON.stringify({
  out: ['models/schema.json', 'models/diagram-definitions.json', 'models/information-placement.yaml', 'models/scenario-deltas.json'],
  diagramTypes: diagramDefs.diagramTypes.length,
  customElementTemplates: deltas.customElementTemplates.length,
  interventionKinds: deltas.interventionCatalog.length,
}, null, 2));
