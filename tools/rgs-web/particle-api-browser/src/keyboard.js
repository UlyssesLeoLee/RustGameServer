// keyboard.js - 全局快捷键
// 设计文档: docs/03-详细设计书.md §2.8

export function installGlobalKeys({ onSearchFocus, onCloseModal }) {
  // 全局: / 聚焦搜索
  document.addEventListener('keydown', (e) => {
    const tag = (e.target.tagName || '').toUpperCase();
    if (tag === 'INPUT' || tag === 'TEXTAREA') return;
    if (e.key === '/') {
      e.preventDefault();
      onSearchFocus?.();
    }
  });

  // 弹窗内: Esc 关闭
  document.getElementById('modal-root').addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
      e.stopPropagation();
      onCloseModal?.();
    }
  });

  // 弹窗内: Tab 焦点循环
  document.getElementById('modal-root').addEventListener('keydown', (e) => {
    if (e.key !== 'Tab') return;
    const modal = document.getElementById('modal');
    if (!modal) return;
    const focusable = [...modal.querySelectorAll('button, [tabindex="0"]')];
    if (focusable.length === 0) return;
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (e.shiftKey && document.activeElement === first) {
      e.preventDefault();
      last.focus();
    } else if (!e.shiftKey && document.activeElement === last) {
      e.preventDefault();
      first.focus();
    }
  });
}

export function installListKeys(listEl) {
  listEl.addEventListener('keydown', (e) => {
    const items = [...listEl.querySelectorAll('.api-item')];
    if (items.length === 0) return;
    const idx = items.indexOf(document.activeElement);
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      items[Math.min(idx + 1, items.length - 1)]?.focus();
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      items[Math.max(idx - 1, 0)]?.focus();
    } else if (e.key === 'Home') {
      e.preventDefault();
      items[0]?.focus();
    } else if (e.key === 'End') {
      e.preventDefault();
      items[items.length - 1]?.focus();
    } else if (e.key === 'Enter' && idx >= 0) {
      e.preventDefault();
      items[idx].click();
    }
  });
}
