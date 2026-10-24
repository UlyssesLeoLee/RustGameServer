/**
 * demo.ts — 示范层 (Designed / Hypothetical) 生成器
 *
 * spec `prototype_workbench.source_fallback`:
 *   "没有目标页面资料时生成至少 3 个明确为 Designed/Hypothetical 的可互动示范 Tab,
 *    不伪造目标产品"
 *
 * 现状 (git 实证): models/prototype.json 的 meta.origin = 'HypotheticalExperiment',
 * RGS 仓无对应 Web 前端源码 (gm-console 仅有 envoy 静态托管), 因此本工作区的
 * 3 个 Screen 全部是**假设性示范**, 不是 RGS 真实界面。
 *
 * 本文件在既有 3 个 Screen / 6 个 Component / 3 个 Action 之上追加一层**显式标注**
 * 的示范实体, 目的是让 spec 要求的 17 种 widget 与 16 种 action 全部有可运行的
 * 实例 (spec rejection: "只有静态截图" / "没有多个 Tab" / "表单不可输入")。
 *
 * 诚实约束:
 *   - 生成的实体一律带 `props.demo = true` / `meta.demoEntities` 登记。
 *   - 界面在示范实体的位置显示"示范"徽标, 不冒充真实产品。
 *   - 不新增任何指向真实生产 API 的行为。
 */

import type { ActionKind, ProtoAction, ProtoComponent, WidgetType } from '../core/types';
import type { ProtoDoc } from './model';

interface DemoComponentSpec {
  id: string;
  widget: WidgetType;
  w: number;
  h: number;
  props: Record<string, unknown>;
  actionIds?: string[];
  stateKey?: string;
}

interface DemoActionSpec {
  id: string;
  kind: ActionKind;
  label: string;
  target?: string;
  payload?: Record<string, unknown>;
  guard?: string;
  nextState?: string;
}

/* ── 16 种 action 的示范实例 (每种至少 1 个) ──────────────────────── */

const DEMO_ACTIONS: DemoActionSpec[] = [
  { id: 'ACT-D01', kind: 'Navigate', label: '打开故障详情', target: 'SCR-002', payload: { screenId: 'SCR-002' } },
  { id: 'ACT-D02', kind: 'Navigate', label: '返回机台总览', target: 'SCR-001', payload: { screenId: 'SCR-001' } },
  { id: 'ACT-D03', kind: 'SubmitMock', label: '提交查询 (正常 Mock)', payload: { fixtureId: 'FX-OK', componentId: 'CMP-003' }, guard: 'GRD-003' },
  { id: 'ACT-D04', kind: 'SubmitMock', label: '提交查询 (错误 Mock)', payload: { fixtureId: 'FX-ERR', componentId: 'CMP-003' } },
  { id: 'ACT-D05', kind: 'SubmitMock', label: '提交查询 (超时 Mock)', payload: { fixtureId: 'FX-TMO', componentId: 'CMP-003' } },
  { id: 'ACT-D06', kind: 'Retry', label: '重试上次请求', payload: {} },
  { id: 'ACT-D07', kind: 'OpenTab', label: '新增页签 (班次对账)', payload: { screenId: 'SCR-003', newTab: true } },
  { id: 'ACT-D08', kind: 'SwitchTab', label: '切换到机台总览页签', payload: { screenId: 'SCR-001' } },
  { id: 'ACT-D09', kind: 'Sort', label: '按心跳排序', payload: { componentId: 'CMP-003', field: '心跳', dir: 'asc' }, guard: 'GRD-002' },
  { id: 'ACT-D10', kind: 'SetVariable', label: '记录查询条件', payload: { name: 'query', value: 'A-02' } },
  { id: 'ACT-D11', kind: 'SetState', label: '切换看板状态', payload: { key: 'grid', value: 'ready', scope: 'screen' } },
  { id: 'ACT-D12', kind: 'Load', label: '重新加载机台表', payload: { componentId: 'CMP-003' } },
  { id: 'ACT-D13', kind: 'Fail', label: '模拟加载失败', payload: { componentId: 'CMP-003', message: '加载失败 (确定性 Mock)' } },
  { id: 'ACT-D14', kind: 'TriggerCase', label: '触发 CASE-006 熔断失败传播', payload: { caseId: 'CASE-006' } },
  { id: 'ACT-D15', kind: 'OpenDialog', label: '打开故障处置对话框', payload: { componentId: 'CMP-D29' }, guard: 'GRD-001' },
  { id: 'ACT-D16', kind: 'CloseDialog', label: '关闭故障处置对话框', payload: { componentId: 'CMP-D29' } },
  { id: 'ACT-D17', kind: 'ShowHide', label: '显示/隐藏机台表', payload: { componentId: 'CMP-003', visible: 'toggle' } },
  { id: 'ACT-D18', kind: 'Filter', label: '只看故障机台', payload: { componentId: 'CMP-003', field: '状态', value: 'fault' } },
  { id: 'ACT-D19', kind: 'Validate', label: '校验表单', payload: { componentId: 'CMP-D02', rules: [{ field: 'CMP-D02', required: true, message: '请输入机台编号' }] } },
];

/* ── 17 种 widget 的示范实例 ──────────────────────────────────────── */

const DEMO_COMPONENTS: DemoComponentSpec[] = [
  /* 输入类 (真 DOM, IME 正常) */
  { id: 'CMP-D02', widget: 'Input', w: 190, h: 36, props: { label: '机台编号', placeholder: '如 A-02', demo: true }, stateKey: 'query' },
  { id: 'CMP-D03', widget: 'Textarea', w: 190, h: 60, props: { label: '处理备注', placeholder: '多行文本', demo: true }, stateKey: 'note' },
  { id: 'CMP-D04', widget: 'Select', w: 190, h: 36, props: { label: '状态筛选', options: ['全部', 'online', 'fault', 'offline'], demo: true }, stateKey: 'statusFilter' },
  { id: 'CMP-D05', widget: 'Radio', w: 190, h: 56, props: { label: '严重程度', options: ['低', '中', '高'], demo: true }, stateKey: 'severity' },
  { id: 'CMP-D06', widget: 'Checkbox', w: 190, h: 36, props: { label: '只看故障机台', demo: true }, stateKey: 'onlyFault' },

  /* 按钮类 */
  { id: 'CMP-D07', widget: 'Button', w: 190, h: 34, props: { label: '打开故障详情', demo: true }, actionIds: ['ACT-D01'] },
  { id: 'CMP-D08', widget: 'Button', w: 190, h: 34, props: { label: '提交查询 (正常 Mock)', demo: true }, actionIds: ['ACT-D03'] },
  { id: 'CMP-D09', widget: 'Button', w: 190, h: 34, props: { label: '提交查询 (错误 Mock)', demo: true }, actionIds: ['ACT-D04'] },
  { id: 'CMP-D10', widget: 'Button', w: 190, h: 34, props: { label: '提交查询 (超时 Mock)', demo: true }, actionIds: ['ACT-D05'] },
  { id: 'CMP-D11', widget: 'Button', w: 190, h: 34, props: { label: '重试上次请求', demo: true }, actionIds: ['ACT-D06'] },
  { id: 'CMP-D12', widget: 'Button', w: 190, h: 34, props: { label: '校验表单', demo: true }, actionIds: ['ACT-D19'] },
  { id: 'CMP-D13', widget: 'Button', w: 190, h: 34, props: { label: '只看故障机台', demo: true }, actionIds: ['ACT-D18'] },
  { id: 'CMP-D14', widget: 'Button', w: 190, h: 34, props: { label: '按心跳排序', demo: true }, actionIds: ['ACT-D09'] },
  { id: 'CMP-D15', widget: 'Button', w: 190, h: 34, props: { label: '重新加载机台表', demo: true }, actionIds: ['ACT-D12'] },
  { id: 'CMP-D16', widget: 'Button', w: 190, h: 34, props: { label: '模拟加载失败', demo: true }, actionIds: ['ACT-D13'] },
  { id: 'CMP-D17', widget: 'Button', w: 190, h: 34, props: { label: '触发 CASE-006 熔断失败传播', demo: true }, actionIds: ['ACT-D14'] },
  { id: 'CMP-D18', widget: 'Button', w: 190, h: 34, props: { label: '新增页签 (班次对账)', demo: true }, actionIds: ['ACT-D07'] },
  { id: 'CMP-D19', widget: 'Button', w: 190, h: 34, props: { label: '切换到机台总览页签', demo: true }, actionIds: ['ACT-D08'] },
  { id: 'CMP-D20', widget: 'Button', w: 190, h: 34, props: { label: '显示/隐藏机台表', demo: true }, actionIds: ['ACT-D17'] },
  { id: 'CMP-D21', widget: 'Button', w: 190, h: 34, props: { label: '记录查询条件', demo: true }, actionIds: ['ACT-D10'] },
  { id: 'CMP-D22', widget: 'Button', w: 190, h: 34, props: { label: '切换看板状态', demo: true }, actionIds: ['ACT-D11'] },
  { id: 'CMP-D33', widget: 'Button', w: 190, h: 34, props: { label: '打开故障处置对话框', demo: true }, actionIds: ['ACT-D15'] },
  { id: 'CMP-D34', widget: 'Button', w: 190, h: 34, props: { label: '关闭故障处置对话框', demo: true }, actionIds: ['ACT-D16'] },
  { id: 'CMP-D37', widget: 'Button', w: 190, h: 34, props: { label: '回到机台总览', demo: true }, actionIds: ['ACT-D02'] },

  /* 展示类 */
  { id: 'CMP-D23', widget: 'Tabs', w: 190, h: 90, props: { tabs: [{ id: 'all', label: '全部', content: '全部机台 (示范)' }, { id: 'fault', label: '故障', content: '故障机台 (示范)' }], demo: true }, stateKey: 'tab' },
  { id: 'CMP-D24', widget: 'List', w: 190, h: 90, props: { items: ['A-02 心跳超时', 'A-07 门禁未闭合', 'A-11 币值不足'], demo: true } },
  { id: 'CMP-D25', widget: 'Navigation', w: 190, h: 40, props: { items: [{ label: '机台总览', screenId: 'SCR-001' }, { label: '故障详情', screenId: 'SCR-002' }, { label: '班次对账', screenId: 'SCR-003' }], demo: true } },
  { id: 'CMP-D26', widget: 'Loading', w: 190, h: 60, props: { message: '加载中 (示范)', demo: true } },
  { id: 'CMP-D27', widget: 'Empty', w: 190, h: 60, props: { message: '暂无数据 (示范)', demo: true } },
  { id: 'CMP-D28', widget: 'Error', w: 190, h: 60, props: { message: '加载失败 (示范)', demo: true } },
  { id: 'CMP-D29', widget: 'Dialog', w: 380, h: 180, props: { title: '故障处置确认 (示范)', body: '这是真实 DOM <dialog> 元素, 可用 ESC 关闭。', demo: true } },
  { id: 'CMP-D30', widget: 'Drawer', w: 380, h: 180, props: { title: '详情抽屉 (示范)', body: '抽屉内容由 fixture 填充。', demo: true } },
  { id: 'CMP-D31', widget: 'Card', w: 190, h: 90, props: { title: '指标卡 (示范)', metric: '128', body: '在线机台', demo: true } },
  { id: 'CMP-D32', widget: 'Card', w: 190, h: 90, props: { title: '故障摘要 (示范)', metric: '3', body: '故障机台', demo: true } },
  { id: 'CMP-D35', widget: 'Table', w: 380, h: 120, props: { columns: ['机台', '状态', '心跳'], rows: [['A-01', 'online', '2s'], ['A-02', 'fault', '41s'], ['A-03', 'offline', '12m']], demo: true } },
  { id: 'CMP-D36', widget: 'Toast', w: 190, h: 60, props: { message: '操作提示 (示范)', demo: true } },
];

/** 示范组件 → 所属 Screen */
const DEMO_PLACEMENT: Record<string, string> = {
  'CMP-D35': 'SCR-002',
  'CMP-D32': 'SCR-002',
  'CMP-D33': 'SCR-002',
  'CMP-D34': 'SCR-002',
  'CMP-D36': 'SCR-003',
  'CMP-D37': 'SCR-003',
};

/**
 * 追加示范层到文档。幂等: 已存在的 ID 不重复添加。
 * @returns 新增的实体 ID 清单
 */
export function applyDemoLayer(doc: ProtoDoc): string[] {
  const added: string[] = [];

  /* Guard: 新增一条"未填机台编号则禁止提交" */
  if (!doc.guards.some((g) => g.id === 'GRD-003')) {
    doc.guards.push({
      id: 'GRD-003',
      name: '已填写机台编号',
      expr: "vars.query != ''",
      description: '示范 guard: 未填机台编号时禁止提交查询 (提交动作被拦截并提示)。',
      version: 1,
    });
    added.push('GRD-003');
  }

  /* Action */
  for (const spec of DEMO_ACTIONS) {
    if (doc.actions.some((a) => a.actionId === spec.id)) continue;
    const action: ProtoAction = {
      actionId: spec.id,
      kind: spec.kind,
      label: spec.label,
      target: spec.target,
      payload: { ...(spec.payload ?? {}), demo: true },
      nextState: spec.nextState,
      guard: spec.guard,
    };
    doc.actions.push(action);
    added.push(spec.id);
  }

  /* Component + 摆放 */
  let x = 24;
  let y = 470;
  let rowH = 0;
  for (const spec of DEMO_COMPONENTS) {
    if (doc.components.some((c) => c.id === spec.id)) continue;
    if (x + spec.w > 1180) { x = 24; y += rowH + 8; rowH = 0; }
    const comp: ProtoComponent = {
      id: spec.id,
      widget: spec.widget,
      x,
      y,
      w: spec.w,
      h: spec.h,
      props: { ...spec.props },
      actionIds: [...(spec.actionIds ?? [])],
      stateKey: spec.stateKey,
    };
    doc.components.push(comp);
    added.push(spec.id);

    const screenId = DEMO_PLACEMENT[spec.id] ?? 'SCR-001';
    const screen = doc.screens.find((s) => s.id === screenId);
    if (screen) {
      screen.componentIds.push(comp.id);
      if (!screen.actionIds.includes(spec.id)) {
        for (const aid of comp.actionIds) {
          if (!screen.actionIds.includes(aid)) screen.actionIds.push(aid);
        }
      }
    }
    x += spec.w + 8;
    rowH = Math.max(rowH, spec.h);
  }

  /* 归属到页面 */
  const page = doc.pages[0];
  if (page) {
    for (const s of doc.screens) {
      if (!page.screenIds.includes(s.id)) page.screenIds.push(s.id);
    }
  }

  for (const id of added) {
    if (!doc.meta.demoEntities.includes(id)) doc.meta.demoEntities.push(id);
  }
  return added;
}

/** 该实体是否属于示范层 */
export function isDemoEntity(doc: ProtoDoc, id: string): boolean {
  return doc.meta.demoEntities.includes(id);
}
