// main.js - 入口
// 设计文档: docs/03-详细设计书.md §4

import { loadAll } from './data-loader.js';
import { getState, setState, subscribe } from './state.js';
import { renderEngineTabs, renderCategoryTabs } from './tabs.js';
import { renderList } from './list.js';
import { renderModal, openModal, closeModal } from './modal.js';
import { installGlobalKeys, installListKeys } from './keyboard.js';
import { t } from './i18n.js';

const els = {
  banner:     document.getElementById('banner'),
  engineTabs: document.getElementById('engine-tabs'),
  catTabs:    document.getElementById('cat-tabs'),
  apiList:    document.getElementById('api-list'),
  search:     document.getElementById('search-input'),
  modalRoot:  document.getElementById('modal-root'),
  listCount:  document.getElementById('list-count'),
  footerVer:  document.getElementById('footer-version'),
  appTitle:   document.getElementById('app-title')
};

async function bootstrap() {
  try {
    const { data, categories, i18n } = await loadAll();
    setState({ data, categories, i18n });
  } catch (e) {
    showBanner(e.message);
    return;
  }

  // 静态文案
  els.appTitle.textContent = t(getState(), 'app.title');
  els.search.placeholder = t(getState(), 'search.placeholder');
  els.footerVer.textContent = t(getState(), 'footer.version');

  // 键盘
  installGlobalKeys({
    onSearchFocus: () => els.search.focus(),
    onCloseModal:  () => closeModal()
  });
  installListKeys(els.apiList);

  // 引擎 tab 点击
  els.engineTabs.addEventListener('click', (e) => {
    const btn = e.target.closest('[data-engine]');
    if (!btn) return;
    const newEngine = btn.dataset.engine;
    // 切换 engine 时重置 category 为 'all'
    setState({ engine: newEngine, category: 'all' });
  });

  // 类别 tab 点击
  els.catTabs.addEventListener('click', (e) => {
    const btn = e.target.closest('[data-category]');
    if (!btn) return;
    setState({ category: btn.dataset.category });
  });

  // 搜索输入
  els.search.addEventListener('input', (e) => {
    setState({ search: e.target.value });
  });

  // 列表点击 → 打开弹窗
  els.apiList.addEventListener('click', (e) => {
    const item = e.target.closest('[data-api-id]');
    if (item) openModal(item.dataset.apiId);
  });

  // 弹窗事件代理
  els.modalRoot.addEventListener('click', (e) => {
    if (e.target.dataset.close === '1') {
      closeModal();
      return;
    }
    const tab = e.target.closest('[data-modal-tab]');
    if (tab) {
      setState({ modalTab: tab.dataset.modalTab });
      return;
    }
    const ex = e.target.closest('[data-ex-engine]');
    if (ex && !ex.disabled) {
      setState({ modalExampleEngine: ex.dataset.exEngine });
    }
  });

  // 订阅渲染
  subscribe(() => renderAll());

  // 首次渲染
  setState({}); // 触发订阅
}

function renderAll() {
  const s = getState();

  renderEngineTabs(s, els.engineTabs);
  renderCategoryTabs(s, els.catTabs);
  renderList(s, els.apiList);
  renderModal(s, els.modalRoot);

  const visible = els.apiList.querySelectorAll('.api-item').length;
  els.listCount.textContent = t(s, 'list.count', { n: visible });
}

function showBanner(msg) {
  els.banner.classList.add('error');
  els.banner.textContent = `数据加载失败: ${msg}`;
}

bootstrap();
