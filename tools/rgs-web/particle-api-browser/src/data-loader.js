// data-loader.js - 加载 + 校验三个 JSON 数据
// 设计文档: docs/03-详细设计书.md §2.3

export async function loadAll({ baseUrl = '' } = {}) {
  const [data, categories, i18n] = await Promise.all([
    fetchJson(`${baseUrl}data/api.json`),
    fetchJson(`${baseUrl}data/categories.json`),
    fetchJson(`${baseUrl}data/i18n.json`)
  ]);
  validateData(data, categories);
  validateCategories(categories);
  validateI18n(i18n);
  return { data, categories, i18n };
}

async function fetchJson(url) {
  const res = await fetch(url, { cache: 'no-cache' });
  if (!res.ok) throw new Error(`HTTP ${res.status} loading ${url}`);
  return res.json();
}

function validateData(data, categories) {
  if (!Array.isArray(data.apis)) {
    throw new Error('api.json: "apis" 必须是数组');
  }
  const seenIds = new Set();
  for (const api of data.apis) {
    if (!api.id) throw new Error('api.json: api 缺少 id 字段');
    if (seenIds.has(api.id)) throw new Error(`api.json: 重复的 id "${api.id}"`);
    seenIds.add(api.id);

    if (!api.primaryEngine) throw new Error(`api.json: ${api.id} 缺少 primaryEngine`);
    const engineCats = categories.categories && categories.categories[api.primaryEngine];
    if (!engineCats) {
      throw new Error(`api.json: ${api.id} 引用了未知引擎 "${api.primaryEngine}"`);
    }
    if (!engineCats.find(c => c.id === api.category)) {
      throw new Error(`api.json: ${api.id} 引用了未知类别 "${api.category}" (engine: ${api.primaryEngine})`);
    }
    if (!api.name) throw new Error(`api.json: ${api.id} 缺少 name`);
    if (!api.summary) throw new Error(`api.json: ${api.id} 缺少 summary`);
  }
}

function validateCategories(categories) {
  if (!categories || typeof categories !== 'object' || !categories.categories) {
    throw new Error('categories.json: 缺少 "categories" 字段');
  }
  for (const [engine, cats] of Object.entries(categories.categories)) {
    if (!Array.isArray(cats)) {
      throw new Error(`categories.json: "${engine}" 必须是数组`);
    }
    const seen = new Set();
    for (const c of cats) {
      if (!c.id) throw new Error(`categories.json: ${engine} 下有类别缺少 id`);
      if (seen.has(c.id)) throw new Error(`categories.json: ${engine} 下重复类别 id "${c.id}"`);
      seen.add(c.id);
    }
  }
}

function validateI18n(i18n) {
  if (!i18n || typeof i18n !== 'object' || !i18n.strings || typeof i18n.strings !== 'object') {
    throw new Error('i18n.json: 缺少 "strings" 字段');
  }
}
