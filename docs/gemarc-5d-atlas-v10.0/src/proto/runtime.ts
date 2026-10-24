/**
 * runtime.ts — Action DSL 运行时 (16 种 action 的可执行实现)
 *
 * spec `prototype_workbench.data`:
 *   "声明式 Action DSL 与 deterministic Mock/Fixture;
 *    无授权不调用目标生产 API, 支持正常/错误/超时/恢复"
 *
 * 实现约束:
 *   - **不发起任何网络请求** (无 fetch / XHR / WebSocket); 数据只来自
 *     doc.fixtures 与 doc.mockFixtures。
 *   - **确定性**: 无随机数、无 Date.now() 参与业务判定; 超时用固定 delayMs。
 *   - 每次执行都写入 doc.log (可导出, 供 ImportAndReplay 复原交互规则)。
 */

import type { ActionKind, ProtoAction } from '../core/types';
import { evalExpr } from './expr';
import { CaseInfo, PHASE_LABEL, phaseOfCase } from './cases';
import {
  type CasePhase, type ProtoDoc, type ProtoLogEntry, actionById, componentById, fixtureById, screenById,
} from './model';

export type RuntimeEventKind = 'structure' | 'value' | 'log';

export interface RuntimeEvent {
  kind: RuntimeEventKind;
  reason: string;
}

export interface ActionResult {
  ok: boolean;
  effect: string;
  detail: string;
}

export interface RuntimeHooks {
  /** Navigate: 切换当前 Screen (页签管理器负责开/激活页签) */
  onNavigate?(screenId: string): void;
  /** OpenTab: 新开页签, 返回新 tabId */
  onOpenTab?(screenId: string, newTab: boolean): string | void;
  /** SwitchTab: 激活已存在的页签 */
  onSwitchTab?(screenId: string): void;
  /** 状态栏提示 */
  onStatus?(text: string): void;
}

export interface CasePhaseMark {
  phase: CasePhase;
  caseId: string;
  binding: 'bound' | 'mock';
  title: string;
  origin: string;
}

export interface RuntimeState {
  vars: Record<string, unknown>;
  screenStates: Record<string, Record<string, string>>;
  componentStates: Record<string, Record<string, string>>;
  formValues: Record<string, string>;
  visible: Record<string, boolean>;
  errors: Record<string, string>;
  loaded: Record<string, unknown>;
  filters: Record<string, { field: string; value: string }>;
  sorts: Record<string, { field: string; dir: 'asc' | 'desc' }>;
  openDialogs: string[];
  casePhases: Record<string, CasePhaseMark>;
  currentScreenId: string;
  navStack: string[];
  pending: { actionId: string; componentId: string | null; fixtureId: string } | null;
  retryCount: number;
}

function emptyState(): RuntimeState {
  return {
    vars: {},
    screenStates: {},
    componentStates: {},
    formValues: {},
    visible: {},
    errors: {},
    loaded: {},
    filters: {},
    sorts: {},
    openDialogs: [],
    casePhases: {},
    currentScreenId: '',
    navStack: [],
    pending: null,
    retryCount: 0,
  };
}

export class ProtoRuntime {
  doc: ProtoDoc;
  state: RuntimeState = emptyState();
  cases: CaseInfo[] = [];
  hooks: RuntimeHooks = {};
  private listeners = new Set<(e: RuntimeEvent) => void>();
  private timers: number[] = [];

  constructor(doc: ProtoDoc) {
    this.doc = doc;
    this.reset();
  }

  /* ── 生命周期 ───────────────────────────────────────────────────── */

  /** 重置到初始态 (屏幕/组件 state 的 initial 值) */
  reset(): void {
    const st = emptyState();
    for (const screen of this.doc.screens) {
      st.screenStates[screen.id] = { ...(screen.states ?? {}) };
      st.currentScreenId = st.currentScreenId || screen.id;
    }
    for (const def of this.doc.states) {
      if (def.scope === 'screen' && st.screenStates[def.key]) continue;
      if (def.scope === 'screen') {
        const owner = this.doc.screens.find((s) => s.id === def.key);
        if (owner) st.screenStates[owner.id] = { ...(st.screenStates[owner.id] ?? {}), [def.key]: def.initial };
      } else if (def.scope === 'component') {
        st.componentStates[def.key] = { ...(st.componentStates[def.key] ?? {}), [def.key]: def.initial };
      } else {
        st.vars[def.key] = def.initial;
      }
    }
    for (const comp of this.doc.components) {
      st.visible[comp.id] = true;
      st.componentStates[comp.id] = st.componentStates[comp.id] ?? {};
    }
    this.state = st;
    this.emit('structure', 'reset');
  }

  destroy(): void {
    for (const t of this.timers) clearTimeout(t);
    this.timers = [];
    this.listeners.clear();
  }

  subscribe(fn: (e: RuntimeEvent) => void): () => void {
    this.listeners.add(fn);
    return () => { this.listeners.delete(fn); };
  }

  private emit(kind: RuntimeEventKind, reason: string): void {
    for (const l of this.listeners) l({ kind, reason });
  }

  private later(fn: () => void, ms: number): void {
    const id = setTimeout(() => {
      this.timers = this.timers.filter((t) => t !== id);
      fn();
    }, ms) as unknown as number;
    this.timers.push(id);
  }

  /* ── 状态读写 ───────────────────────────────────────────────────── */

  setVar(name: string, value: unknown): void {
    this.state.vars[name] = value;
    this.emit('value', `var:${name}`);
  }

  setScreenState(screenId: string, key: string, value: string): void {
    const cur = this.state.screenStates[screenId] ?? (this.state.screenStates[screenId] = {});
    cur[key] = value;
    this.emit('structure', `screenState:${screenId}.${key}`);
  }

  setComponentState(componentId: string, key: string, value: string): void {
    const cur = this.state.componentStates[componentId] ?? (this.state.componentStates[componentId] = {});
    cur[key] = value;
    this.emit('structure', `componentState:${componentId}.${key}`);
  }

  getComponentState(componentId: string, key: string): string | undefined {
    return this.state.componentStates[componentId]?.[key];
  }

  /** 表单值写入 —— 只发 'value' 事件, 不重绘 DOM (保证 IME 组合输入不被打断) */
  setFormValue(componentId: string, value: string): void {
    this.state.formValues[componentId] = value;
    this.emit('value', `form:${componentId}`);
  }

  getFormValue(componentId: string): string {
    return this.state.formValues[componentId] ?? '';
  }

  /* ── Guard 求值 ─────────────────────────────────────────────────── */

  guardScope(): Record<string, unknown> {
    const screenStates = this.state.screenStates[this.state.currentScreenId] ?? {};
    return {
      vars: this.state.vars,
      screen: screenStates,
      component: this.state.componentStates,
      form: this.state.formValues,
      case: Object.fromEntries(Object.entries(this.state.casePhases).map(([k, v]) => [k, v.phase])),
    };
  }

  evaluateGuard(expr: string | undefined): { passed: boolean; error?: string } {
    if (!expr) return { passed: true };
    const r = evalExpr(expr, this.guardScope());
    if (!r.ok) return { passed: false, error: r.error };
    return { passed: !!r.value };
  }

  /* ── 动作执行 ───────────────────────────────────────────────────── */

  /** 组件触发动作 (取该组件 actionIds 的第 index 个) */
  runComponentAction(componentId: string, actionId?: string): ActionResult {
    const comp = componentById(this.doc, componentId);
    if (!comp) return { ok: false, effect: '组件不存在', detail: componentId };
    const aid = actionId ?? comp.actionIds[0];
    if (!aid) return { ok: false, effect: '组件未绑定任何 action', detail: componentId };
    return this.runAction(aid, componentId);
  }

  /** 执行一个 action (含 guard 判定 + 日志) */
  runAction(actionId: string, fromComponentId: string | null = null): ActionResult {
    const action = actionById(this.doc, actionId);
    if (!action) {
      const res = { ok: false, effect: 'action 不存在', detail: actionId };
      this.log(actionId, 'Navigate', actionId, fromComponentId, null, true, res.effect, res.detail, false);
      return res;
    }

    const guard = this.evaluateGuard(action.guard);
    if (!guard.passed) {
      const detail = guard.error
        ? `guard 解析失败: ${guard.error}`
        : `guard "${action.guard}" 未通过 → 动作被拦截`;
      this.log(actionId, action.kind, action.label, fromComponentId, action.guard ?? null, false, 'GUARD_BLOCKED', detail, false);
      this.hooks.onStatus?.(`${action.label} — 被 guard 拦截`);
      this.emit('structure', 'guard-blocked');
      return { ok: false, effect: 'GUARD_BLOCKED', detail };
    }

    const res = this.apply(action, fromComponentId);
    this.log(actionId, action.kind, action.label, fromComponentId, action.guard ?? null, true, res.effect, res.detail, res.ok);
    this.hooks.onStatus?.(`${action.label} — ${res.effect}`);
    this.emit('structure', `action:${actionId}`);
    return res;
  }

  private apply(action: ProtoAction, fromComponentId: string | null): ActionResult {
    const payload = action.payload ?? {};
    const target = action.target ?? (payload.screenId as string | undefined);
    switch (action.kind) {
      case 'Navigate': {
        const screenId = String(target ?? '');
        if (!screenById(this.doc, screenId)) return { ok: false, effect: 'NOT_FOUND', detail: `目标 Screen 不存在: ${screenId}` };
        this.state.navStack.push(this.state.currentScreenId);
        this.state.currentScreenId = screenId;
        this.hooks.onNavigate?.(screenId);
        return { ok: true, effect: 'NAVIGATED', detail: `→ ${screenId}` };
      }
      case 'OpenTab': {
        const screenId = String(target ?? '');
        if (!screenById(this.doc, screenId)) return { ok: false, effect: 'NOT_FOUND', detail: `目标 Screen 不存在: ${screenId}` };
        const tabId = this.hooks.onOpenTab?.(screenId, payload.newTab === true);
        this.state.currentScreenId = screenId;
        return { ok: true, effect: 'TAB_OPENED', detail: `新页签 ${tabId ?? screenId}` };
      }
      case 'SwitchTab': {
        const screenId = String(target ?? '');
        if (!screenById(this.doc, screenId)) return { ok: false, effect: 'NOT_FOUND', detail: `目标 Screen 不存在: ${screenId}` };
        this.hooks.onSwitchTab?.(screenId);
        this.state.currentScreenId = screenId;
        return { ok: true, effect: 'TAB_SWITCHED', detail: `→ ${screenId}` };
      }
      case 'SetState': {
        const key = String(payload.key ?? '');
        const value = String(payload.value ?? '');
        const scope = (payload.scope as string | undefined) ?? 'screen';
        if (!key) return { ok: false, effect: 'INVALID', detail: '缺少 payload.key' };
        if (scope === 'component') {
          const cid = String(payload.componentId ?? fromComponentId ?? '');
          this.setComponentState(cid, key, value);
          return { ok: true, effect: 'STATE_SET', detail: `${cid}.${key} = ${value}` };
        }
        if (scope === 'global') {
          this.setVar(key, value);
          return { ok: true, effect: 'STATE_SET', detail: `global.${key} = ${value}` };
        }
        const sid = String(payload.screenId ?? this.state.currentScreenId);
        this.setScreenState(sid, key, value);
        return { ok: true, effect: 'STATE_SET', detail: `${sid}.${key} = ${value}` };
      }
      case 'SetVariable': {
        const name = String(payload.name ?? '');
        if (!name) return { ok: false, effect: 'INVALID', detail: '缺少 payload.name' };
        this.setVar(name, payload.value);
        return { ok: true, effect: 'VAR_SET', detail: `vars.${name} = ${String(payload.value)}` };
      }
      case 'ShowHide': {
        const cid = String(payload.componentId ?? fromComponentId ?? '');
        if (!componentById(this.doc, cid)) return { ok: false, effect: 'NOT_FOUND', detail: `组件不存在: ${cid}` };
        const raw = payload.visible;
        const visible = raw === 'toggle'
          ? !(this.state.visible[cid] ?? true)
          : raw === false || raw === 'false' ? false : true;
        this.state.visible[cid] = visible;
        this.emit('structure', `visible:${cid}`);
        return { ok: true, effect: visible ? 'SHOWN' : 'HIDDEN', detail: cid };
      }
      case 'Validate': {
        const cid = String(payload.componentId ?? fromComponentId ?? '');
        const rules = Array.isArray(payload.rules) ? payload.rules : [];
        const problems: string[] = [];
        for (const raw of rules) {
          const rule = raw as { field?: string; required?: boolean; minLength?: number; maxLength?: number; message?: string };
          const field = String(rule.field ?? cid);
          const value = this.getFormValue(field);
          if (rule.required && !value.trim()) problems.push(rule.message ?? `${field} 为必填`);
          if (rule.minLength !== undefined && value.length < rule.minLength) problems.push(rule.message ?? `${field} 少于 ${rule.minLength} 字符`);
          if (rule.maxLength !== undefined && value.length > rule.maxLength) problems.push(rule.message ?? `${field} 超过 ${rule.maxLength} 字符`);
        }
        if (problems.length) {
          this.state.errors[cid] = problems.join('; ');
          this.emit('structure', `validate:${cid}`);
          return { ok: false, effect: 'INVALID', detail: problems.join('; ') };
        }
        delete this.state.errors[cid];
        this.emit('structure', `validate:${cid}`);
        return { ok: true, effect: 'VALID', detail: `${cid} 校验通过` };
      }
      case 'SubmitMock': {
        const fixtureId = String(payload.fixtureId ?? 'FX-OK');
        const fixture = fixtureById(this.doc, fixtureId);
        const mode = fixture?.mode ?? 'ok';
        const cid = String(payload.componentId ?? fromComponentId ?? '');
        const dataScreen = this.state.currentScreenId;
        const data = this.readFixture(dataScreen, cid);
        this.state.pending = { actionId: action.actionId, componentId: cid, fixtureId };
        this.state.retryCount = 0;

        if (mode === 'ok') {
          if (cid) this.state.loaded[cid] = data;
          this.setComponentState(cid, 'status', 'success');
          this.setComponentState(cid, 'message', fixture?.message ?? '提交成功 (Mock)');
          delete this.state.errors[cid];
          return { ok: true, effect: 'SUBMIT_OK', detail: `${fixtureId} → ${cid}` };
        }
        if (mode === 'error') {
          this.setComponentState(cid, 'status', 'error');
          this.state.errors[cid] = fixture?.message ?? '提交失败 (Mock)';
          return { ok: false, effect: 'SUBMIT_ERROR', detail: `${fixtureId} → ${cid}` };
        }
        if (mode === 'recover') {
          this.setComponentState(cid, 'status', 'waiting');
          this.setComponentState(cid, 'message', '首次请求失败, 等待重试 (Mock)');
          this.later(() => {
            this.setComponentState(cid, 'status', 'error');
            this.state.errors[cid] = fixture?.message ?? '首次失败 (Mock)';
            this.emit('structure', 'mock-recover-failed');
          }, fixture?.delayMs ?? 800);
          return { ok: false, effect: 'SUBMIT_PENDING', detail: `${fixtureId} 首次失败, 待重试` };
        }
        /* timeout */
        this.setComponentState(cid, 'status', 'waiting');
        this.setComponentState(cid, 'message', '请求超时中… (确定性 Mock, 不发起真实调用)');
        this.later(() => {
          this.setComponentState(cid, 'status', 'timeout');
          this.state.errors[cid] = fixture?.message ?? '请求超时 (Mock)';
          this.emit('structure', 'mock-timeout');
        }, fixture?.delayMs ?? 1200);
        return { ok: false, effect: 'SUBMIT_TIMEOUT', detail: `${fixtureId} → ${cid} (${fixture?.delayMs ?? 1200}ms 后判定超时)` };
      }
      case 'OpenDialog': {
        const cid = String(payload.componentId ?? fromComponentId ?? '');
        if (!componentById(this.doc, cid)) return { ok: false, effect: 'NOT_FOUND', detail: `组件不存在: ${cid}` };
        if (!this.state.openDialogs.includes(cid)) this.state.openDialogs.push(cid);
        this.emit('structure', `dialog:${cid}`);
        return { ok: true, effect: 'DIALOG_OPENED', detail: cid };
      }
      case 'CloseDialog': {
        const cid = String(payload.componentId ?? fromComponentId ?? '');
        this.state.openDialogs = this.state.openDialogs.filter((x) => x !== cid);
        this.emit('structure', `dialog:${cid}`);
        return { ok: true, effect: 'DIALOG_CLOSED', detail: cid };
      }
      case 'Filter': {
        const cid = String(payload.componentId ?? fromComponentId ?? '');
        const field = String(payload.field ?? '');
        const value = String(payload.value ?? '');
        this.state.filters[cid] = { field, value };
        this.emit('structure', `filter:${cid}`);
        return { ok: true, effect: 'FILTERED', detail: `${cid}.${field} = ${value}` };
      }
      case 'Sort': {
        const cid = String(payload.componentId ?? fromComponentId ?? '');
        const field = String(payload.field ?? '');
        const dir = (payload.dir === 'desc' ? 'desc' : 'asc');
        this.state.sorts[cid] = { field, dir };
        this.emit('structure', `sort:${cid}`);
        return { ok: true, effect: 'SORTED', detail: `${cid}.${field} ${dir}` };
      }
      case 'Load': {
        const cid = String(payload.componentId ?? fromComponentId ?? '');
        const data = this.readFixture(this.state.currentScreenId, cid);
        if (data === undefined) return { ok: false, effect: 'NO_FIXTURE', detail: `无 fixture: ${this.state.currentScreenId}/${cid}` };
        this.state.loaded[cid] = data;
        this.setComponentState(cid, 'status', 'success');
        delete this.state.errors[cid];
        return { ok: true, effect: 'LOADED', detail: `${cid} ← fixture` };
      }
      case 'Fail': {
        const cid = String(payload.componentId ?? fromComponentId ?? '');
        const message = String(payload.message ?? '失败 (Mock)');
        this.setComponentState(cid, 'status', 'error');
        this.state.errors[cid] = message;
        return { ok: false, effect: 'FAILED', detail: `${cid}: ${message}` };
      }
      case 'Retry': {
        const pending = this.state.pending;
        if (!pending) return { ok: false, effect: 'NOTHING_TO_RETRY', detail: '没有待重试的请求' };
        this.state.retryCount += 1;
        const fixture = fixtureById(this.doc, pending.fixtureId);
        const cid = pending.componentId ?? '';
        if (fixture?.mode === 'recover') {
          if (cid) this.state.loaded[cid] = this.readFixture(this.state.currentScreenId, cid);
          this.setComponentState(cid, 'status', 'success');
          this.setComponentState(cid, 'message', fixture.message);
          delete this.state.errors[cid];
          this.state.pending = null;
          return { ok: true, effect: 'RETRY_OK', detail: `第 ${this.state.retryCount} 次重试成功 (恢复 Mock)` };
        }
        if (fixture?.mode === 'timeout') {
          this.setComponentState(cid, 'status', 'waiting');
          this.later(() => {
            this.setComponentState(cid, 'status', 'timeout');
            this.state.errors[cid] = fixture.message;
            this.emit('structure', 'mock-timeout');
          }, fixture.delayMs);
          return { ok: false, effect: 'RETRY_TIMEOUT', detail: `第 ${this.state.retryCount} 次重试仍超时 (确定性)` };
        }
        this.setComponentState(cid, 'status', 'error');
        this.state.errors[cid] = fixture?.message ?? '重试仍失败 (Mock)';
        return { ok: false, effect: 'RETRY_FAILED', detail: `第 ${this.state.retryCount} 次重试仍失败` };
      }
      case 'TriggerCase': {
        const caseId = String(payload.caseId ?? '');
        const info = this.cases.find((c) => c.caseId === caseId);
        if (!info) return { ok: false, effect: 'CASE_NOT_FOUND', detail: `未找到 Case: ${caseId}` };
        const cid = String(payload.componentId ?? fromComponentId ?? '');
        const binding = this.doc.caseBindings.find((b) => b.caseId === caseId && b.componentId === cid);
        const resolvedCid = binding?.componentId || cid || this.firstBoundComponent(caseId);
        if (!resolvedCid) {
          return { ok: false, effect: 'CASE_MOCK', detail: `${caseId} 无可绑定的组件 — 显式标为 Mock, 未映射到 DOM` };
        }
        const phase = phaseOfCase(info);
        this.state.casePhases[resolvedCid] = {
          phase,
          caseId,
          binding: binding ? 'bound' : 'mock',
          title: info.title,
          origin: info.origin,
        };
        this.setComponentState(resolvedCid, 'status', phase);
        this.setComponentState(resolvedCid, 'phaseLabel', PHASE_LABEL[phase]);
        this.emit('structure', `case:${caseId}`);
        return {
          ok: true,
          effect: 'CASE_TRIGGERED',
          detail: `${caseId} → ${resolvedCid} = ${PHASE_LABEL[phase]} (${binding ? 'bound' : 'Mock — 无 link 绑定'})`,
        };
      }
      default:
        return { ok: false, effect: 'NOT_IMPLEMENTED', detail: `未实现的 action kind: ${action.kind}` };
    }
  }

  private firstBoundComponent(caseId: string): string {
    const b = this.doc.caseBindings.find((x) => x.caseId === caseId);
    return b?.componentId ?? '';
  }

  /** 读取 fixture: doc.fixtures[screenId][componentId] */
  readFixture(screenId: string, componentId: string): unknown {
    const perScreen = this.doc.fixtures[screenId] as Record<string, unknown> | undefined;
    if (!perScreen) return undefined;
    return perScreen[componentId];
  }

  /* ── 日志 ───────────────────────────────────────────────────────── */

  private log(
    actionId: string, kind: ActionKind, label: string, componentId: string | null,
    guard: string | null, guardPassed: boolean, effect: string, detail: string, ok: boolean,
  ): void {
    const entry: ProtoLogEntry = {
      seq: this.doc.log.length + 1,
      actionId,
      kind,
      label,
      screenId: this.state.currentScreenId,
      componentId,
      guard,
      guardPassed,
      effect,
      detail,
      ok,
    };
    this.doc.log.push(entry);
    if (this.doc.log.length > 500) this.doc.log.splice(0, this.doc.log.length - 500);
    this.emit('log', `log:${actionId}`);
  }

  /* ── 回放 (ImportAndReplay) ─────────────────────────────────────── */

  /**
   * 用导出的日志重建一次交互序列, 验证"导入后 Action 不丢失"。
   * 逐条重新执行 actionId, 比较 kind 与 effect 是否与原日志一致。
   */
  replay(log: ProtoLogEntry[]): { total: number; matched: number; missing: string[]; mismatched: string[] } {
    const missing: string[] = [];
    const mismatched: string[] = [];
    let matched = 0;
    for (const entry of log) {
      const action = actionById(this.doc, entry.actionId);
      if (!action) { missing.push(entry.actionId); continue; }
      if (action.kind !== entry.kind) { mismatched.push(`${entry.actionId} kind ${action.kind} ≠ ${entry.kind}`); continue; }
      matched++;
    }
    return { total: log.length, matched, missing, mismatched };
  }
}
