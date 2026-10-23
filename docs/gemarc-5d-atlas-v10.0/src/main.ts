/**
 * main.ts — 入口
 *
 * 所有模型 JSON 在构建期内联 (esbuild json loader), 运行时零网络请求。
 * 严禁在此处或任何模块中引入 fetch / import() / eval (spec `forbidden_at_runtime`)。
 */

import architecture from '../models/architecture.json';
import evidenceDoc from '../models/evidence.json';
import promptsDoc from '../models/prompts.json';
import requirementsDoc from '../models/requirements.json';
import casesDoc from '../models/cases.json';
import rulesDoc from '../models/causal-rules.json';
import spatialDoc from '../models/spatial.json';
import traceabilityDoc from '../models/traceability.json';
import prototypeDoc from '../models/prototype.json';

import { App, type Dimension } from './core/app';
import { DIMENSIONS } from './dims';
import { cssVariables } from './core/tokens';

type AnyRecord = Record<string, unknown>;

declare global {
  interface Window {
    __ATLAS_PROMPTS?: unknown[];
    __ATLAS_MODELS?: AnyRecord;
  }
}

function boot() {
  /* 令牌 CSS 变量注入 (tokens.ts 是唯一真源) */
  const style = document.createElement('style');
  style.textContent = `:root{${cssVariables()}}`;
  document.head.appendChild(style);

  const model = architecture as unknown as Parameters<typeof App.prototype.constructor>[0];
  const app = new App(model, (evidenceDoc as unknown as { records: never[] }).records);

  /* 其它模型通过全局注入, 供各维度模块读取 (避免层层透传) */
  window.__ATLAS_PROMPTS = (promptsDoc as unknown as { prompts: unknown[] }).prompts;
  window.__ATLAS_MODELS = {
    requirements: (requirementsDoc as unknown as { requirements: unknown[] }).requirements,
    prompts: (promptsDoc as unknown as { prompts: unknown[] }).prompts,
    cases: (casesDoc as unknown as { cases: unknown[] }).cases,
    rules: (rulesDoc as unknown as { rules: unknown[] }).rules,
    spatial: spatialDoc,
    traceability: traceabilityDoc,
    prototype: prototypeDoc,
  };

  for (const d of DIMENSIONS) app.register(d);

  /* 顶栏: 真实来源信息 (revision / branch / subject) */
  app.renderSourceInfo();

  /* ── 顶栏: 维度导航 (0D 不在此 — 它是内部门禁, 由 d0 自行挂指示器) ─── */
  for (const btn of document.querySelectorAll<HTMLButtonElement>('.dim-tab')) {
    const dim = btn.dataset.dim as Dimension;
    if (dim === '0D') { btn.remove(); continue; }
    btn.addEventListener('click', () => app.go(dim));
  }

  /* ── 0D 门禁: 挂一次即可 (app.go('0D') 会 mount d0) ─────────── */
  app.go('0D');

  /* ── 主题 ────────────────────────────────────────────────── */
  const themeBtn = document.getElementById('btn-theme')!;
  themeBtn.addEventListener('click', () => {
    const cur = document.documentElement.getAttribute('data-theme') === 'light' ? 'dark' : 'light';
    document.documentElement.setAttribute('data-theme', cur);
    app.status(`主题: ${cur}`);
  });

  /* ── 导入 / 导出 (内存模式 + JSON, 不用 IndexedDB) ────────── */
  const fileInput = document.getElementById('file-input') as HTMLInputElement;
  document.getElementById('btn-import')!.addEventListener('click', () => fileInput.click());
  fileInput.addEventListener('change', async () => {
    const f = fileInput.files?.[0];
    if (!f) return;
    try {
      const text = await f.text();
      JSON.parse(text);   // 只做可解析性校验; 真正替换模型需要重建 App
      app.status(`已读取 ${f.name} (${text.length} 字节) — 校验通过`);
    } catch (e) {
      app.status(`导入失败: ${(e as Error).message}`);
    }
    fileInput.value = '';
  });
  document.getElementById('btn-export')!.addEventListener('click', () => {
    const blob = new Blob([JSON.stringify(app.model, null, 2)], { type: 'application/json' });
    const a = document.createElement('a');
    a.href = URL.createObjectURL(blob);
    a.download = `architecture-${app.model.meta.shortRevision}.json`;
    a.click();
    setTimeout(() => URL.revokeObjectURL(a.href), 1000);
    app.status('已导出 architecture.json');
  });

  /* ── 全局搜索 ────────────────────────────────────────────── */
  const search = document.getElementById('global-search-input') as HTMLInputElement;
  search.addEventListener('input', () => {
    const r = app.search(search.value);
    app.status(search.value
      ? `搜索 "${search.value}": ${r.nodes.length} 节点 / ${r.edges.length} 边`
      : `模型: ${app.model.counts.crates} crates · ${app.model.counts.grpcMethods} rpc · ${app.model.counts.tables} 表 · ${app.model.counts.evidenceRecords} 证据`);
  });  document.addEventListener('keydown', (e) => {
    if (e.key === '/' && document.activeElement !== search) { e.preventDefault(); search.focus(); }
  });

  /* ── 侧栏 / 检查器 抽屉开关 ─────────────────────────────────── */
  /* 狭い画面ではサイドバー/インスペクタが position:fixed の抽屉になり、
     既存の折りたたみボタンは**その抽屉の中に**あるため画面外.Operation できない。
     そのためヘッダーに常時表示のハンドル (btn-toggle-left / right) を置き、
     ここからは常に到達できるようにする (对抗審査の指摘)。 */
  const appEl = document.getElementById('app')!;
  const isNarrow = () => window.matchMedia('(max-width: 1023px)').matches;
  const setDrawer = (side: 'left' | 'right', open: boolean) => {
    const attr = `data-${side}`;
    if (open) {
      appEl.setAttribute(attr, 'open');
      appEl.removeAttribute('data-left-collapsed');
    } else {
      appEl.setAttribute(attr, 'collapsed');
    }
    for (const b of document.querySelectorAll<HTMLButtonElement>(`[data-drawer="${side}"]`)) {
      b.setAttribute('aria-expanded', String(open));
    }
  };
  const drawerOpen = (side: 'left' | 'right') => appEl.getAttribute(`data-${side}`) === 'open';
  const toggleDrawer = (side: 'left' | 'right') => {
    const open = !drawerOpen(side);
    setDrawer(side, open);
    app.status(`${side === 'left' ? '目录' : '检查器'}抽屉: ${open ? '打开' : '关闭'}`);
  };
  document.getElementById('btn-toggle-left')?.addEventListener('click', () => toggleDrawer('left'));
  document.getElementById('btn-toggle-right')?.addEventListener('click', () => toggleDrawer('right'));
  document.getElementById('btn-collapse-left')?.addEventListener('click', () => toggleDrawer('left'));
  document.getElementById('btn-collapse-right')?.addEventListener('click', () => toggleDrawer('right'));
  /* 幅が広いときは inner の折りたたみ (列を 0 にする) を使う。
     狭いときは outer の抽屉 (position:fixed) を使う。 */
  for (const b of document.querySelectorAll<HTMLButtonElement>('[data-drawer="left"],[data-drawer="right"]')) {
    b.setAttribute('data-drawer', b.dataset.drawer!);
  }

  app.renderLegend();
  app.go('2D');
  app.status(`就绪 — ${app.model.meta.subject} @ ${app.model.meta.shortRevision} (${app.model.meta.branch}) · ${app.model.counts.evidenceRecords} 条证据`);
}

if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', boot);
else boot();
