/**
 * types.ts — Universal GemArc 5D Design Atlas v10.0 规范模型类型
 *
 * 对应 spec `canonical_engineering_model` 与 `source_and_truth.statuses`。
 * 所有 ID 稳定, 跨维度不重编 (per spec `immutable_ids`)。
 */

/* ── 存在性 / 实现状态 (spec: statuses) ─────────────────────────────── */
export type ExistenceStatus = 'Present' | 'Absent' | 'Unknown';
export type ImplementationStatus = 'Implemented' | 'Designed' | 'Inferred' | 'Absent';
export type EvidenceKind =
  | 'SourceCode' | 'TestTrace' | 'RuntimeLog' | 'Document'
  | 'PublicObservation' | 'UserRequirement';
export type ScenarioOrigin = 'ObservedReplay' | 'DesignedSimulation' | 'HypotheticalExperiment';
export type QAStatus = 'PASS' | 'FAIL' | 'NOT_TESTED' | 'DEGRADED' | 'BLOCKED' | 'FLAKY_UNRESOLVED';

/* ── 证据 ───────────────────────────────────────────────────────────── */
export interface Evidence {
  id: string;
  kind: EvidenceKind;
  fileLine: string;      // path:startLine-endLine
  path: string;
  startLine: number;
  endLine: number;
  revision: string;
  note: string;
  snippet?: string;
}

/* ── 图模型 ─────────────────────────────────────────────────────────── */
export type NodeKind = 'Service' | 'Api' | 'Table' | 'Message' | 'Component' | 'Screen' | 'Actor' | 'Requirement';
export type EdgeType = 'grpc-call' | 'grpc-call-designed' | 'uses' | 'owns' | 'deploys-to' | 'reads' | 'writes' | 'depends-on';

export interface GraphNode {
  id: string;
  kind: NodeKind;
  name: string;
  qualifiedName: string;
  role: string;
  domainId: string;          // 业务域 (spec domain_palette)
  parentId: string | null;
  portIds: string[];
  properties: Record<string, unknown>;
  verification: ImplementationStatus;
  evidenceIds: string[];
}

export interface GraphEdge {
  id: string;
  sourceId: string | null;
  targetId: string | null;
  sourcePortId: string | null;
  targetPortId: string | null;
  type: EdgeType;
  directed: boolean;
  guardId: string | null;
  properties: Record<string, unknown>;
  verification: ImplementationStatus;
  evidenceIds: string[];
}

export interface GraphPort {
  id: string;
  name: string;
  apiId: string | null;
  crate: string;
  requestType: string;
  responseType: string;
  direction: 'caller->callee' | 'callee->caller' | 'bidirectional';
  type: string;
  properties: Record<string, unknown>;
  evidenceIds: string[];
}

export type ContainerKind = 'System' | 'Building' | 'Floor' | 'Room';
export interface Container {
  id: string;
  kind: ContainerKind;
  name: string;
  parentId: string | null;
  childrenIds: string[];
  properties?: Record<string, unknown>;
}

export interface ArchitectureModel {
  meta: {
    schemaVersion: string;
    subject: string;
    repo: string | null;
    revision: string;
    shortRevision: string;
    branch: string;
    capturedAt: string;
    generator: string;
    generatedFiles: string[];
    honestyNotice: string;
  };
  counts: Record<string, number>;
  containers: Container[];
  nodes: GraphNode[];
  edges: GraphEdge[];
  designedOnlyEdges?: GraphEdge[];
  ports: GraphPort[];
  sourceRoles: Record<string, string>;
}

/* ── 需求契约 (spec: requirement_contract) ─────────────────────────── */
export interface Requirement {
  id: string;                 // REQ-xxx
  intent: string;
  measurableAcceptance: string[];
  nonGoals: string[];
  provenance: string;          // 证据 id 或 'Designed'
  evidenceIds: string[];
  affectedDimensions: Dimension[];
  linkedTestIds: string[];
  status: ImplementationStatus;
}

export type Dimension = '0D' | '1D' | '2D' | '3D' | '4D' | '5D' | 'Prototype';

/* ── 1D Prompt (spec: 1D.schemas) ──────────────────────────────────── */
export interface PromptRecord {
  promptId: string;
  version: string;
  objective: string;
  requirementIds: string[];
  nodeEdgeIds: string[];
  caseIds: string[];
  inputs: string[];
  constraints: string[];
  outputs: string[];
  acceptance: string[];
  evidenceIds: string[];
}

/* ── 4D/5D Case (spec: 4D.cases / simulation_and_causality.scenarios) ─ */
export type CaseKind =
  | 'Normal' | 'Abnormal' | 'Boundary' | 'Concurrent' | 'Delay' | 'Timeout'
  | 'DuplicateDelivery' | 'OutOfOrder' | 'FailurePropagation' | 'Retry'
  | 'Recovery' | 'Compensation' | 'ResourceContention' | 'Combined';

export interface CaseRecord {
  caseId: string;
  kind: CaseKind;
  title: string;
  origin: ScenarioOrigin;
  description: string;
  startNodeId: string;
  precondition?: string;
  steps: CaseStep[];
  expectedOutcome: string;
  nodeIds: string[];
  edgeIds: string[];
  evidenceIds: string[];
}

export interface CaseStep {
  seq: number;
  actor: string;              // node id
  action: string;
  target?: string;            // node id
  edgeId?: string;
  guard?: string;
  timeoutMs?: number;
  retry?: { max: number; backoffMs: number };
  expect?: string;
}

/* ── 因果规则 (spec: causal-rules) ─────────────────────────────────── */
export interface CausalRule {
  ruleId: string;
  name: string;
  scopeNodeIds: string[];
  when: string;               // guard 表达式
  then: string;               // 动作
  else?: string;
  timeoutMs?: number;
  retry?: { max: number; backoffMs: number };
  compensation?: string;
  evidenceIds: string[];
  origin: ScenarioOrigin;
}

/* ── 事件 / Trace ───────────────────────────────────────────────────── */
export interface SimulationEvent {
  eventId: string;
  caseId: string;
  logicalTime: number;
  sequence: number;
  sourceNodeId: string | null;
  targetNodeId: string | null;
  edgeId: string | null;
  previousState: string | null;
  nextState: string | null;
  ruleId: string | null;
  evidenceIds: string[];
}

export interface EventTrace {
  traceId: string;
  caseId: string;
  seed: number;
  modelRevision: string;
  startedAt: string;
  events: SimulationEvent[];
  finalStates: Record<string, string>;
  outcome: 'SUCCESS' | 'FAILURE' | 'TIMEOUT' | 'COMPENSATED' | 'BLOCKED';
  snapshots: { logicalTime: number; states: Record<string, string> }[];
}

/* ── 5D 干预 (spec: 5D.operations / simulation_and_causality.interventions) ─ */
export type InterventionKind =
  | 'DisableNode' | 'DisableEdge' | 'BypassNode' | 'ChangeGuard' | 'ChangeTimeout'
  | 'ChangeRetry' | 'DelayMessage' | 'InjectFault' | 'AddInteractiveElement'
  | 'RewireGraph';

export interface Intervention {
  interventionId: string;
  kind: InterventionKind;
  targetId: string;
  label: string;
  value?: unknown;
  origin: ScenarioOrigin;
}

export interface ScenarioGraphDelta {
  deltaId: string;
  baseRevision: string;
  appliedInterventions: Intervention[];
  addedNodeIds: string[];
  removedEdgeIds: string[];
  rewiredEdges: { from: string; to: string; replacedEdgeId: string; newEdgeId: string }[];
  note: string;
}

export interface CausalExplanation {
  explanationId: string;
  baselineCaseId: string;
  branchCaseId: string;
  firstDivergence: { logicalTime: number; sequence: number; eventA: string; eventB: string } | null;
  /** 分歧点の状態迁移 (A / B) — 同一 event ID でも内容が違���うるため必須 */
  firstDivergenceDetail?: { a: string; b: string; aEvent: string; bEvent: string } | null;
  affectedNodeIds: string[];
  affectedEdgeIds: string[];
  propagationPath: string[];
  recovery: string | null;
  compensation: string | null;
  terminalDiff: Record<string, { baseline: string; branch: string }>;
  /** 片側のみに存在するノード (構造差分。終態差異とは区別する) */
  structuralOnlyNodeIds?: string[];
  assumptions: string[];
  evidenceIds: string[];
  conclusion: 'DIFFERENT' | 'NO_DIFFERENCE' | 'UNKNOWN';
}

/* ── 原型 (spec: prototype_workbench) ──────────────────────────────── */
export type WidgetType =
  | 'Button' | 'Input' | 'Textarea' | 'Select' | 'Radio' | 'Checkbox' | 'Tabs'
  | 'List' | 'Table' | 'Card' | 'Dialog' | 'Drawer' | 'Toast' | 'Navigation'
  | 'Loading' | 'Empty' | 'Error';

export type ActionKind =
  | 'Navigate' | 'OpenTab' | 'SwitchTab' | 'SetState' | 'SetVariable' | 'ShowHide'
  | 'Validate' | 'SubmitMock' | 'OpenDialog' | 'CloseDialog' | 'Filter' | 'Sort'
  | 'Load' | 'Fail' | 'Retry' | 'TriggerCase';

export interface ProtoAction {
  actionId: string;
  kind: ActionKind;
  label: string;
  target?: string;
  payload?: Record<string, unknown>;
  nextState?: string;
  guard?: string;
}

export interface ProtoComponent {
  id: string;
  widget: WidgetType;
  x: number; y: number; w: number; h: number;
  props: Record<string, unknown>;
  actionIds: string[];
  stateKey?: string;
}

export interface ProtoScreen {
  id: string;
  name: string;
  route: string;
  componentIds: string[];
  actionIds: string[];
  states: Record<string, string>;
  variant?: string;
}

export interface PrototypeModel {
  meta: { schemaVersion: string; origin: ScenarioOrigin; note: string };
  screens: ProtoScreen[];
  components: ProtoComponent[];
  actions: ProtoAction[];
  fixtures: Record<string, unknown>;
  links: {
    screenId: string; componentId: string;
    requirementIds: string[]; nodeIds: string[]; caseIds: string[]; promptIds: string[];
  }[];
}

/* ── 追踪链 (spec: trace_chain) ────────────────────────────────────── */
export interface Traceability {
  requirements: Requirement[];
  promptToRequirement: { promptId: string; requirementIds: string[] }[];
  nodeToRequirement: { nodeId: string; requirementIds: string[] }[];
  caseToNode: { caseId: string; nodeIds: string[] }[];
  testToCase: { testId: string; caseIds: string[] }[];
}

/* ── 空间投影 (spec: 3D) ───────────────────────────────────────────── */
export interface SpatialRoom {
  id: string;
  name: string;
  floorId: string;
  bounds: { x: number; y: number; z: number; w: number; h: number; d: number };
  nodeIds: string[];
  semantics: 'physical' | 'logical';   // spec: 虚拟空间不得冒充物理部署事实
  evidenceIds: string[];
}

export interface SpatialModel {
  meta: { schemaVersion: string; handedness: 'left'; note: string };
  building: { id: string; name: string; floors: { id: string; name: string; y: number }[] };
  rooms: SpatialRoom[];
}
