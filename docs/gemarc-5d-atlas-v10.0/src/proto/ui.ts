/**
 * ui.ts — Prototype 工作区的局部 DOM 助手 + 注入式样式
 *
 * 边界 (per Work Package 指令):
 *   - 不修改 src/styles.css。需要的样式以 <style id="px-styles"> 注入,
 *     所有选择器以 .px- 前缀限定, 卸载时整块移除, 不污染其它维度。
 *   - 不用 fetch / eval / 动态 import。
 */

export type Attrs = Record<string, string | number | boolean | null | undefined>;

/** 创建元素 (attrs 中 text= 写文本, html= 写 HTML) */
export function h<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  attrs: Attrs = {},
  children: (Node | string)[] = [],
): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  for (const [k, v] of Object.entries(attrs)) {
    if (v === null || v === undefined || v === false) continue;
    if (k === 'class') node.className = String(v);
    else if (k === 'text') node.textContent = String(v);
    else if (k === 'html') node.innerHTML = String(v);
    else if (v === true) node.setAttribute(k, '');
    else node.setAttribute(k, String(v));
  }
  for (const c of children) node.appendChild(typeof c === 'string' ? document.createTextNode(c) : c);
  return node;
}

/** 清空并写入子节点 */
export function fill(node: HTMLElement, ...children: (Node | string)[]) {
  node.textContent = '';
  for (const c of children) node.appendChild(typeof c === 'string' ? document.createTextNode(c) : c);
  return node;
}

/** 读取 CSS 变量 (令牌真源在 core/tokens.ts) */
export function token(name: string, fallback = ''): string {
  const v = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  return v || fallback;
}

/* ── Toast ─────────────────────────────────────────────────────────── */

export type ToastKind = 'ok' | 'warn' | 'err' | 'info';

export function toast(text: string, kind: ToastKind = 'info'): void {
  const root = document.getElementById('toast-root');
  if (!root) return;
  const cls = kind === 'ok' ? 'toast toast--ok' : kind === 'warn' ? 'toast toast--warn' : kind === 'err' ? 'toast toast--err' : 'toast';
  const el = h('div', { class: cls, role: 'status' }, [text]);
  root.appendChild(el);
  setTimeout(() => { el.remove(); }, 4200);
}

/* ── Modal ─────────────────────────────────────────────────────────── */

export interface ModalHandle {
  root: HTMLElement;
  close(): void;
}

/** 打开模态框 (复用 app shell 的 #modal-root, 与 d1 同款) */
export function openModal(title: string, body: Node | string, opts: { wide?: boolean } = {}): ModalHandle {
  const root = document.getElementById('modal-root');
  if (!root) return { root: document.createElement('div'), close: () => {} };
  root.hidden = false;
  const modal = h('div', {
    class: 'modal',
    role: 'dialog',
    'aria-modal': 'true',
    'aria-label': title,
    style: opts.wide ? 'max-width:min(1180px,94vw)' : '',
  }, [
    h('div', { class: 'modal-head' }, [
      h('strong', { text: title }),
      h('button', { class: 'icon-btn', 'aria-label': '关闭', 'data-px-close': '' }, ['✕']),
    ]),
    h('div', { class: 'modal-body' }, [typeof body === 'string' ? h('div', { html: body }) : body]),
  ]);
  root.textContent = '';
  root.appendChild(modal);
  const close = () => { root.hidden = true; root.textContent = ''; };
  modal.querySelector('[data-px-close]')?.addEventListener('click', close);
  const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') { close(); document.removeEventListener('keydown', onKey); } };
  document.addEventListener('keydown', onKey);
  const focusTarget = modal.querySelector<HTMLElement>('button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])');
  focusTarget?.focus();
  return { root: modal, close };
}

/* ── 注入式样式 ───────────────────────────────────────────────────── */

const STYLE_ID = 'px-styles';

const CSS = `
.px-root{display:flex;flex-direction:column;height:100%;min-height:0;background:var(--bg)}
.px-subtabs{display:flex;gap:2px;flex:0 0 auto;padding:4px 8px;background:var(--surface);border-bottom:1px solid var(--edge);overflow-x:auto}
.px-subtab{height:28px;padding:0 12px;font-size:12px;font-weight:600;white-space:nowrap;background:transparent;border:1px solid transparent;border-radius:var(--r-sm);color:var(--text-2);cursor:pointer}
.px-subtab:hover{background:var(--raised);color:var(--text)}
.px-subtab[aria-selected="true"]{background:var(--raised);color:var(--text);border-color:var(--focus)}
.px-screentabs{display:flex;align-items:center;gap:4px;flex:0 0 auto;padding:4px 8px;background:var(--surface);border-bottom:1px solid var(--edge);overflow-x:auto;min-height:38px}
.px-screentab{display:inline-flex;align-items:center;gap:6px;height:26px;padding:0 6px 0 10px;font-size:12px;white-space:nowrap;background:var(--raised);border:1px solid var(--edge);border-radius:var(--r-sm);color:var(--text-2);cursor:pointer}
.px-screentab[aria-selected="true"]{color:var(--text);border-color:var(--focus)}
.px-screentab .px-tab-x{border:0;background:transparent;color:inherit;cursor:pointer;font-size:12px;line-height:1;padding:2px 4px;border-radius:4px}
.px-screentab .px-tab-x:hover{background:var(--fail);color:#fff}
.px-body{flex:1 1 auto;min-height:0;overflow:auto;padding:10px}
.px-pane{display:none}
.px-pane.is-active{display:block}
.px-banner{border:1px solid var(--warn);background:color-mix(in srgb,var(--warn) 14%,transparent);border-radius:var(--r-md);padding:8px 10px;font-size:12px;color:var(--text);margin-bottom:10px}
.px-banner strong{color:var(--warn)}
.px-banner code{font-size:11px}
.px-note{font-size:11px;color:var(--text-2)}
.px-cols{display:grid;grid-template-columns:repeat(auto-fit,minmax(300px,1fr));gap:10px}
.px-row{display:flex;gap:6px;align-items:center;flex-wrap:wrap}
.px-kv{display:grid;grid-template-columns:104px 1fr;gap:3px 8px;font-size:12px}
.px-kv dt{color:var(--text-2)}
.px-kv dd{margin:0;word-break:break-word}
.px-chip{display:inline-block;padding:1px 7px;border-radius:999px;font-size:10px;font-weight:600;border:1px solid currentColor}
.px-chip--bound{color:var(--success)}
.px-chip--mock{color:var(--warn)}
.px-chip--demo{color:var(--info)}
.px-chip--real{color:var(--success)}
.px-empty-slot{padding:24px;text-align:center;color:var(--text-2);font-size:12px;border:1px dashed var(--edge);border-radius:var(--r-md)}

/* ── 预览 (真实 DOM) ─────────────────────────────────────────────── */
.px-preview{display:flex;justify-content:center;padding:10px 0}
.px-frame{position:relative;border:1px solid var(--edge);border-radius:var(--r-md);background:var(--surface);overflow:hidden;box-shadow:var(--shadow-2)}
.px-frame[data-viewport="desktop"]{width:min(1440px,100%)}
.px-frame[data-viewport="tablet"]{width:min(768px,100%)}
.px-frame[data-viewport="mobile"]{width:min(390px,100%)}
.px-viewportbar{display:flex;align-items:center;gap:6px;padding:4px 8px;background:var(--raised);border-bottom:1px solid var(--edge);font-size:11px;color:var(--text-2)}
.px-canvas{position:relative;overflow:auto}
.px-canvas[data-layout="fixed"]{height:auto;min-height:420px}
.px-canvas[data-layout="fluid"]{display:flex;flex-direction:column;gap:8px;padding:8px;height:auto;min-height:300px}
.px-widget{position:relative}
.px-canvas[data-layout="fixed"] .px-widget{position:absolute}
.px-canvas[data-layout="fluid"] .px-widget{position:static;width:auto!important;height:auto!important;min-height:34px}
.px-widget[data-hidden="true"]{display:none}
.px-widget[data-hotspot="true"]{outline:1px dashed var(--focus);outline-offset:1px}
.px-widget-inner{width:100%;height:100%;overflow:auto}
.px-canvas[data-layout="fluid"] .px-widget-inner{height:auto;overflow:visible}
.px-btn{display:inline-flex;align-items:center;justify-content:center;height:28px;padding:0 12px;font-size:12px;background:var(--raised);border:1px solid var(--edge);border-radius:var(--r-sm);color:var(--text);cursor:pointer}
.px-btn:hover{border-color:var(--focus)}
.px-btn--primary{background:var(--focus);color:var(--bg);border-color:var(--focus)}
.px-field{display:flex;flex-direction:column;gap:3px;padding:4px}
.px-field label{font-size:11px;color:var(--text-2)}
.px-input,.px-textarea,.px-select{width:100%;background:var(--raised);color:var(--text);border:1px solid var(--edge);border-radius:var(--r-sm);padding:4px 6px;font:inherit;font-size:12px}
.px-input:focus,.px-textarea:focus,.px-select:focus{border-color:var(--focus)}
.px-input[aria-invalid="true"],.px-textarea[aria-invalid="true"]{border-color:var(--fail)}
.px-err{font-size:11px;color:var(--fail)}
.px-card{width:100%;height:100%;border:1px solid var(--edge);border-radius:var(--r-md);background:var(--raised);padding:8px;overflow:auto}
.px-card h4{margin:0 0 6px;font-size:12px}
.px-card .px-metric{font-size:22px;font-weight:700}
.px-table{width:100%;border-collapse:collapse;font-size:12px}
.px-table th,.px-table td{text-align:left;padding:4px 6px;border-bottom:1px solid var(--edge)}
.px-table th{color:var(--text-2);font-size:11px;text-transform:uppercase}
.px-list{margin:0;padding:0 0 0 16px;font-size:12px}
.px-drawer{width:100%;height:100%;border:1px solid var(--focus);border-radius:var(--r-md);background:var(--raised);padding:8px;overflow:auto}
.px-dialog{position:absolute;inset:12px;margin:auto;max-width:92%;max-height:86%;border:1px solid var(--focus);border-radius:var(--r-lg);background:var(--surface);box-shadow:var(--shadow-3);padding:10px;z-index:20;overflow:auto}
.px-dialog:not([open]){display:none}
.px-dialog h4{margin:0 0 8px;font-size:13px}
.px-toast{width:100%;height:100%;border-left:3px solid var(--info);border-radius:var(--r-sm);background:var(--raised);padding:6px 8px;font-size:12px}
.px-loading{display:flex;align-items:center;gap:8px;width:100%;height:100%;padding:8px;font-size:12px;color:var(--text-2)}
.px-loading::before{content:"";width:12px;height:12px;border-radius:50%;border:2px solid var(--edge);border-top-color:var(--focus);animation:px-spin 900ms linear infinite}
@keyframes px-spin{to{transform:rotate(360deg)}}
.px-empty{width:100%;height:100%;display:grid;place-items:center;font-size:12px;color:var(--text-2);border:1px dashed var(--edge);border-radius:var(--r-sm)}
.px-error{width:100%;height:100%;border:1px solid var(--fail);border-radius:var(--r-sm);background:color-mix(in srgb,var(--fail) 12%,transparent);padding:6px 8px;font-size:12px;color:var(--fail)}
.px-nav{display:flex;gap:6px;flex-wrap:wrap;width:100%;height:100%;align-items:center}
.px-nav button{background:transparent;border:0;color:var(--focus);cursor:pointer;font-size:12px;text-decoration:underline;padding:2px 0}
.px-tabstrip{display:flex;gap:2px;border-bottom:1px solid var(--edge)}
.px-tabstrip button{background:transparent;border:0;border-bottom:2px solid transparent;color:var(--text-2);font-size:11px;padding:4px 8px;cursor:pointer}
.px-tabstrip button[aria-selected="true"]{color:var(--text);border-bottom-color:var(--focus)}
.px-phase{display:flex;align-items:center;gap:6px;margin-bottom:4px;padding:3px 6px;border-radius:var(--r-sm);font-size:11px;border:1px solid var(--edge);background:var(--raised)}
.px-phase[data-phase="error"]{border-color:var(--fail);color:var(--fail)}
.px-phase[data-phase="waiting"]{border-color:var(--warn);color:var(--warn)}
.px-phase[data-phase="success"]{border-color:var(--success);color:var(--success)}
.px-phase[data-phase="compensating"]{border-color:var(--info);color:var(--info)}
.px-phase[data-phase="blocked"]{border-color:var(--fail);color:var(--fail)}
.px-mockbadge{position:absolute;top:0;right:0;z-index:5;font-size:9px;padding:0 4px;border-radius:0 0 0 5px;background:var(--warn);color:#20160a}
.px-hotspot-tag{position:absolute;bottom:0;left:0;z-index:5;font-size:9px;padding:0 4px;background:var(--focus);color:var(--bg);border-radius:0 5px 0 0}

/* ── Konva 编辑器 ────────────────────────────────────────────────── */
.px-editor-wrap{position:relative;border:1px solid var(--edge);border-radius:var(--r-md);background:var(--surface);overflow:auto;min-height:420px}
.px-toolrow{display:flex;gap:4px;flex-wrap:wrap;align-items:center;margin-bottom:8px}
.px-canvas-host{position:relative}
.px-layers{font-size:12px}
.px-layer-item{display:flex;align-items:center;gap:6px;padding:3px 6px;border-radius:var(--r-sm);cursor:pointer;border:1px solid transparent}
.px-layer-item:hover{background:var(--raised)}
.px-layer-item[aria-selected="true"]{background:var(--raised);border-color:var(--focus)}
.px-layer-item code{font-size:10px;color:var(--text-2)}
.px-log{max-height:280px;overflow:auto;font:11px/16px var(--mono)}
.px-log td{vertical-align:top}
.px-log tr[data-ok="false"]{color:var(--fail)}
.px-tbl-wrap{overflow:auto}
`;

let styleEl: HTMLStyleElement | null = null;

/** 注入工作区样式 (幂等) */
export function installStyles(): void {
  if (styleEl && styleEl.isConnected) return;
  const el = document.createElement('style');
  el.id = STYLE_ID;
  el.textContent = CSS;
  document.head.appendChild(el);
  styleEl = el;
}

/** 移除工作区样式 (卸载时调用) */
export function removeStyles(): void {
  styleEl?.remove();
  styleEl = null;
}

/** 下载一段文本 (无 fetch, 纯 Blob + a[download]) */
export function downloadText(filename: string, text: string, mime = 'application/json'): void {
  const blob = new Blob([text], { type: `${mime};charset=utf-8` });
  const a = document.createElement('a');
  a.href = URL.createObjectURL(blob);
  a.download = filename;
  a.click();
  setTimeout(() => URL.revokeObjectURL(a.href), 1000);
}

/** 读取本地文件 (FileReader, 非 fetch) */
export function readFileText(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const fr = new FileReader();
    fr.onload = () => resolve(String(fr.result ?? ''));
    fr.onerror = () => reject(new Error('FileReader 失败'));
    fr.readAsText(file, 'utf-8');
  });
}
