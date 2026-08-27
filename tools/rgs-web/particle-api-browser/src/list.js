// list.js - 列表过滤 + 渲染
// 设计文档: docs/03-详细设计书.md §2.5

import { escapeHtml } from './util.js';
import { getCategoryLabel } from './tabs.js';
import { t } from './i18n.js';

export function getFilteredApis(state) {
  if (!state.data) return [];
  let apis = state.data.apis;

  if (state.engine !== 'all') {
    apis = apis.filter(a => a.primaryEngine === state.engine);
  }
  if (state.category !== 'all') {
    apis = apis.filter(a => a.category === state.category);
  }
  if (state.search) {
    const q = state.search.toLowerCase().trim();
    apis = apis.filter(a => {
      if (a.name.toLowerCase().includes(q)) return true;
      if (a.summary.toLowerCase().includes(q)) return true;
      if (a.description && a.description.toLowerCase().includes(q)) return true;
      if (a.tags && a.tags.some(tag => tag.toLowerCase().includes(q))) return true;
      return false;
    });
  }
  return apis;
}

export function renderList(state, container) {
  const apis = getFilteredApis(state);
  if (apis.length === 0) {
    container.innerHTML = `<li class="list-empty">${escapeHtml(t(state, 'list.empty'))}</li>`;
    return;
  }
  container.innerHTML = apis.map(api => `
    <li class="api-item"
        data-api-id="${escapeHtml(api.id)}"
        role="option"
        tabindex="0"
        aria-selected="false">
      <div class="api-item-header">
        <div class="api-name">${escapeHtml(api.name)}</div>
        <div class="api-tags">
          ${(api.tags || []).slice(0, 3).map(tg => `<span class="tag">${escapeHtml(tg)}</span>`).join('')}
        </div>
      </div>
      <div class="api-summary">${escapeHtml(api.summary)}</div>
      <div class="api-meta">
        <span class="engine-badge engine-${escapeHtml(api.primaryEngine)}">${escapeHtml(api.primaryEngine)}</span>
        <span>·</span>
        <span>${escapeHtml(getCategoryLabel(state, api.primaryEngine, api.category))}</span>
      </div>
    </li>
  `).join('');
}
