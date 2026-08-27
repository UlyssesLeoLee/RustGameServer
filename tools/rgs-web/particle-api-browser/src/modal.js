// modal.js - 详情弹窗 + 焦点陷阱
// 设计文档: docs/03-详细设计书.md §2.6

import { escapeHtml } from './util.js';
import { renderCodeBlock } from './codeblock.js';
import { t } from './i18n.js';
import { setState } from './state.js';

const TABS = [
  { id: 'overview', key: 'tab.overview' },
  { id: 'params',   key: 'tab.params' },
  { id: 'examples', key: 'tab.examples' },
  { id: 'caveats',  key: 'tab.caveats' },
  { id: 'diff',     key: 'tab.diff' }
];

const EXAMPLE_ENGINES = ['unity', 'unreal', 'godot', 'physis'];

export function openModal(apiId) {
  setState({ selectedApiId: apiId, modalTab: 'overview', modalExampleEngine: 'unity' });
  document.getElementById('modal-root').setAttribute('aria-hidden', 'false');
  // 焦点陷阱 - 下一帧聚焦第一个可聚焦元素
  requestAnimationFrame(() => trapFocus());
}

export function closeModal() {
  setState({ selectedApiId: null });
  document.getElementById('modal-root').setAttribute('aria-hidden', 'true');
  releaseFocus();
}

let _previousFocus = null;

function trapFocus() {
  const modal = document.getElementById('modal');
  if (!modal) return;
  _previousFocus = document.activeElement;
  const focusable = [...modal.querySelectorAll('button, [tabindex="0"]')];
  if (focusable[0]) focusable[0].focus();
}

function releaseFocus() {
  if (_previousFocus && typeof _previousFocus.focus === 'function') {
    _previousFocus.focus();
  }
  _previousFocus = null;
}

export function renderModal(state, container) {
  if (!state.selectedApiId) {
    container.innerHTML = '';
    return;
  }
  const api = state.data.apis.find(a => a.id === state.selectedApiId);
  if (!api) {
    container.innerHTML = '';
    return;
  }

  const tabsHtml = TABS.map(tb => {
    const active = state.modalTab === tb.id;
    return `<button type="button" role="tab"
            class="modal-tab ${active ? 'active' : ''}"
            data-modal-tab="${tb.id}"
            aria-selected="${active}">
            ${escapeHtml(t(state, tb.key))}
          </button>`;
  }).join('');

  container.innerHTML = `
    <div class="modal-overlay" data-close="1"></div>
    <div id="modal" class="modal" role="dialog" aria-modal="true" aria-labelledby="modal-title">
      <header class="modal-header">
        <h2 id="modal-title" class="modal-title">${escapeHtml(api.name)}</h2>
        <button type="button" class="modal-close" data-close="1" aria-label="${escapeHtml(t(state, 'modal.close'))}">×</button>
      </header>
      <div class="modal-meta">
        <span class="engine-badge engine-${escapeHtml(api.primaryEngine)}">${escapeHtml(api.primaryEngine)}</span>
        <span class="modal-signature">${escapeHtml(api.signature || '')}</span>
        ${(api.tags || []).map(tg => `<span class="tag">${escapeHtml(tg)}</span>`).join('')}
      </div>
      <nav class="modal-tabs" role="tablist">${tabsHtml}</nav>
      <div class="modal-body">${renderModalTabContent(state, api)}</div>
    </div>
  `;
}

function renderModalTabContent(state, api) {
  switch (state.modalTab) {
    case 'overview': return renderOverview(state, api);
    case 'params':   return renderParams(api);
    case 'examples': return renderExamples(state, api);
    case 'caveats':  return renderCaveats(api);
    case 'diff':     return renderDiff(api);
    default:         return '';
  }
}

function renderOverview(state, api) {
  return `<div class="modal-section modal-overview">
    <h3>${escapeHtml(t(state, 'tab.overview'))}</h3>
    <p>${escapeHtml(api.description || api.summary)}</p>
  </div>`;
}

function renderParams(api) {
  const params = api.parameters || [];
  if (params.length === 0) {
    return `<div class="modal-section"><p style="color:var(--fg-3); font-size:13px;">该 API 无参数。</p></div>`;
  }
  const rows = params.map(p => `
    <tr>
      <td class="param-name">${escapeHtml(p.name)}</td>
      <td class="param-type">${escapeHtml(p.type || '')}</td>
      <td>${p.required
        ? '<span class="param-req">required</span>'
        : '<span class="param-optional">optional</span>'}</td>
      <td>${escapeHtml(p.default ?? '—')}</td>
      <td>${escapeHtml(p.unit || '')}</td>
      <td>${escapeHtml(p.description || '')}</td>
    </tr>
  `).join('');
  return `<div class="modal-section">
    <h3>参数</h3>
    <table class="param-table">
      <thead>
        <tr>
          <th>名称</th><th>类型</th><th>必填</th><th>默认值</th><th>单位</th><th>说明</th>
        </tr>
      </thead>
      <tbody>${rows}</tbody>
    </table>
  </div>`;
}

function renderExamples(state, api) {
  const subTabs = EXAMPLE_ENGINES.map(e => {
    const has = api.examples && api.examples[e];
    const active = state.modalExampleEngine === e;
    return `<button type="button"
            class="ex-engine-tab ${active ? 'active' : ''}"
            data-ex-engine="${e}"
            ${has ? '' : 'disabled'}>
            ${e}${has ? '' : ' (无)'}
          </button>`;
  }).join('');

  const ex = api.examples && api.examples[state.modalExampleEngine];
  let codeBlock;
  if (ex) {
    codeBlock = renderCodeBlock(ex.language, ex.code);
  } else {
    codeBlock = `<div class="ex-empty">${escapeHtml(t(state, 'modal.noExamples'))}</div>`;
  }

  const notes = ex && ex.notes
    ? `<div class="ex-notes">💡 ${escapeHtml(ex.notes)}</div>`
    : '';

  return `<div class="modal-section">
    <h3>示例 · ${escapeHtml(state.modalExampleEngine)}</h3>
    <nav class="ex-engine-tabs" role="tablist">${subTabs}</nav>
    ${codeBlock}
    ${notes}
  </div>`;
}

function renderCaveats(api) {
  const list = api.caveats || [];
  if (list.length === 0) {
    return `<div class="modal-section"><p style="color:var(--fg-3); font-size:13px;">暂无注意事项。</p></div>`;
  }
  return `<div class="modal-section">
    <h3>注意事项</h3>
    <ul class="modal-caveats">
      ${list.map(c => `<li>${escapeHtml(c)}</li>`).join('')}
    </ul>
  </div>`;
}

function renderDiff(api) {
  const diffs = api.engineDifferences || [];
  if (diffs.length === 0) {
    return `<div class="modal-section"><p style="color:var(--fg-3); font-size:13px;">暂无跨引擎差异数据。</p></div>`;
  }
  const rows = diffs.map(d => `
    <tr>
      <th><span class="engine-badge engine-${escapeHtml(d.engine)}">${escapeHtml(d.engine)}</span></th>
      <td>${escapeHtml(d.behavior || '')}</td>
      <td>${escapeHtml(d.note || '')}</td>
    </tr>
  `).join('');
  return `<div class="modal-section modal-diff">
    <h3>引擎差异</h3>
    <table>
      <thead>
        <tr><th>引擎</th><th>行为</th><th>备注</th></tr>
      </thead>
      <tbody>${rows}</tbody>
    </table>
  </div>`;
}
