// i18n.js - 文案模板替换
// 设计文档: docs/03-详细设计书.md §1.5

export function t(state, key, vars = {}) {
  // 防御: i18n / strings 缺失时降级返回 key 而非抛错
  if (!state || !state.i18n || !state.i18n.strings) return key;
  let s = state.i18n.strings[key] ?? key;
  if (typeof s !== 'string') return key;
  for (const [k, v] of Object.entries(vars)) {
    s = s.replace(`{${k}}`, v);
  }
  return s;
}
