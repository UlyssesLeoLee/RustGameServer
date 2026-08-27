// state.js - 全局状态 + 订阅发布
// 设计文档: docs/03-详细设计书.md §2.2

const initial = {
  engine: 'all',           // 'all' | 'unity' | 'unreal' | 'godot' | 'physis'
  category: 'all',         // 'all' | <engine-specific category id>
  search: '',              // user search input
  selectedApiId: null,     // current modal target; null = closed
  modalTab: 'overview',    // 'overview' | 'params' | 'examples' | 'caveats' | 'diff'
  modalExampleEngine: 'unity', // 'unity' | 'unreal' | 'godot' | 'physis'
  data: null,              // 加载后的 api.json
  categories: null,        // 加载后的 categories.json
  i18n: null               // 加载后的 i18n.json
};

let state = { ...initial };
const subscribers = new Set();

export function getState() { return state; }

export function setState(patch) {
  state = { ...state, ...patch };
  subscribers.forEach(fn => fn(state));
}

export function subscribe(fn) {
  subscribers.add(fn);
  return () => subscribers.delete(fn);
}

export function reset() { setState(initial); }
