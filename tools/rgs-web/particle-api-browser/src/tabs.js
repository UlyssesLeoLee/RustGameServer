// tabs.js - 一级(引擎) + 二级(类别) 选项卡
// 设计文档: docs/03-详细设计书.md §2.4

import { t } from './i18n.js';

const ENGINES = ['all', 'unity', 'unreal', 'godot', 'physis'];

export function renderEngineTabs(state, container) {
  container.innerHTML = ENGINES.map(e => {
    const active = state.engine === e;
    return `<button type="button" role="tab"
            class="engine-tab ${active ? 'active' : ''}"
            data-engine="${e}"
            aria-selected="${active}">
            ${t(state, `engine.${e}`)}
          </button>`;
  }).join('');
}

export function renderCategoryTabs(state, container) {
  let cats;
  if (state.engine === 'all') {
    // 合并所有引擎的类别（去重 + order 排序）
    const map = new Map();
    for (const engine of ['unity', 'unreal', 'godot', 'physis']) {
      for (const c of state.categories.categories[engine] || []) {
        if (!map.has(c.id)) map.set(c.id, { id: c.id, label: c.label, order: c.order });
      }
    }
    cats = [...map.values()].sort((a, b) => a.order - b.order);
  } else {
    cats = state.categories.categories[state.engine] || [];
  }

  // 注: 切换 engine 时由 main.js 主动 setState({ category: 'all' }) 重置, 这里只读不写

  container.innerHTML = cats.map(c => {
    const active = state.category === c.id;
    return `<button type="button" role="tab"
            class="cat-tab ${active ? 'active' : ''}"
            data-category="${c.id}"
            aria-selected="${active}">
            ${c.label}
          </button>`;
  }).join('');
}

export function getCategoryLabel(state, engine, categoryId) {
  const cats = state.categories.categories[engine] || [];
  return cats.find(c => c.id === categoryId)?.label ?? categoryId;
}
