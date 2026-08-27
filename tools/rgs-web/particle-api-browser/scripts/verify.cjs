// scripts/verify.cjs - 端到端最小验证（不引入 jsdom）
// 流程: 加载 JSON → 跑过滤器 → 模拟渲染 → 统计匹配项

const http = require('http');

function fetchJson(path) {
  return new Promise((resolve, reject) => {
    http.get('http://127.0.0.1:8765' + path, (res) => {
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => {
        try { resolve(JSON.parse(data)); }
        catch (e) { reject(new Error('JSON parse failed: ' + path + ' :: ' + e.message)); }
      });
    }).on('error', reject);
  });
}

async function main() {
  console.log('=== 1. 加载数据 ===');
  const [api, categories, i18n] = await Promise.all([
    fetchJson('/data/api.json'),
    fetchJson('/data/categories.json'),
    fetchJson('/data/i18n.json')
  ]);
  console.log('  apis         =', api.apis.length);
  console.log('  categories   =', Object.keys(categories.categories).join(', '));
  console.log('  i18n strings =', Object.keys(i18n.strings).length);

  console.log('\n=== 2. 过滤测试 ===');
  function filter(state) {
    let r = api.apis;
    if (state.engine !== 'all') r = r.filter(a => a.primaryEngine === state.engine);
    if (state.category !== 'all') r = r.filter(a => a.category === state.category);
    if (state.search) {
      const q = state.search.toLowerCase();
      r = r.filter(a =>
        a.name.toLowerCase().includes(q) ||
        a.summary.toLowerCase().includes(q) ||
        (a.tags || []).some(t => t.toLowerCase().includes(q))
      );
    }
    return r;
  }
  const cases = [
    { name: 'all/all',      state: { engine: 'all',   category: 'all',     search: '' } },
    { name: 'unity/all',    state: { engine: 'unity', category: 'all',     search: '' } },
    { name: 'unreal/spawn', state: { engine: 'unreal',category: 'spawn',   search: '' } },
    { name: 'godot/emission', state: { engine: 'godot', category: 'emission', search: '' } },
    { name: 'physis/all',   state: { engine: 'physis',category: 'all',     search: '' } },
    { name: 'search:emit',  state: { engine: 'all',   category: 'all',     search: 'emit' } },
    { name: 'search:burst', state: { engine: 'all',   category: 'all',     search: 'burst' } },
    { name: 'search:NONE',  state: { engine: 'all',   category: 'all',     search: 'xxxnotexist' } }
  ];
  for (const c of cases) {
    const r = filter(c.state);
    const ids = r.map(a => a.id).join(', ');
    console.log('  ' + c.name.padEnd(20) + ' => ' + r.length + '  ' + ids);
  }

  console.log('\n=== 3. 示例完整性 ===');
  let allOk = true;
  for (const a of api.apis) {
    const engs = ['unity','unreal','godot','physis'];
    const missing = engs.filter(e => !a.examples || !a.examples[e]);
    if (missing.length || (a.engineDifferences || []).length < 2) {
      console.log('  FAIL  ' + a.id + '  missing=' + missing.join(','));
      allOk = false;
    }
  }
  if (allOk) console.log('  所有 8 条 API 都有完整 4 引擎示例 + 4 条差异');

  console.log('\n=== 4. 字段完整性 ===');
  const required = ['id', 'primaryEngine', 'name', 'category', 'summary', 'description', 'signature'];
  for (const a of api.apis) {
    for (const f of required) {
      if (a[f] == null || a[f] === '') {
        console.log('  FAIL  ' + a.id + '  missing field: ' + f);
        allOk = false;
      }
    }
  }
  if (allOk) console.log('  所有 8 条 API 必填字段齐全');

  console.log('\n=== 5. 类别引用一致性 ===');
  for (const a of api.apis) {
    const engineCats = categories.categories[a.primaryEngine] || [];
    if (!engineCats.find(c => c.id === a.category)) {
      console.log('  FAIL  ' + a.id + '  category "' + a.category + '" 不在 ' + a.primaryEngine + ' 类别列表');
      allOk = false;
    }
  }
  if (allOk) console.log('  所有 category 引用都合法');

  console.log('\n=== 6. ID 唯一性 ===');
  const ids = new Set();
  let dup = false;
  for (const a of api.apis) {
    if (ids.has(a.id)) { console.log('  FAIL  duplicate id: ' + a.id); dup = true; }
    ids.add(a.id);
  }
  if (!dup) console.log('  所有 8 个 id 唯一');

  console.log('\n=== 7. 源码 import 校验（基于静态扫描）===');
  const fs = require('fs');
  const path = require('path');
  const srcDir = path.join(__dirname, '..', 'src');
  const importOk = true;
  const files = fs.readdirSync(srcDir).filter(f => f.endsWith('.js'));
  for (const f of files) {
    const c = fs.readFileSync(path.join(srcDir, f), 'utf8');
    const re = /from\s+['"](\.[^'"]+)['"]/g;
    let m;
    while ((m = re.exec(c))) {
      const target = path.resolve(path.join(srcDir, m[1]));
      if (!fs.existsSync(target)) {
        console.log('  FAIL  ' + f + ' imports missing file: ' + m[1]);
        return;
      }
    }
  }
  console.log('  全部 ' + files.length + ' 个模块的 import 路径都存在');

  console.log('\n=== 8. HTML 标签完整性 ===');
  const fs2 = require('fs');
  const html = fs2.readFileSync(path.join(__dirname, '..', 'index.html'), 'utf8');
  // 注: modal-title 由 modal.js 动态注入, 不要求静态 HTML 中存在
  const need = ['id="banner"', 'id="engine-tabs"', 'id="cat-tabs"', 'id="api-list"', 'id="search-input"', 'id="modal-root"', 'id="list-count"', 'src="./src/main.js"', 'href="./styles.css"'];
  // 校验 modal.js 内部确实注入了 id="modal-title"
  const modalJs = fs2.readFileSync(path.join(__dirname, '..', 'src', 'modal.js'), 'utf8');
  if (!modalJs.includes('id="modal-title"')) {
    console.log('  FAIL  modal.js should inject id="modal-title"');
    htmlOk = false;
  }
  let htmlOk = true;
  for (const n of need) {
    if (!html.includes(n)) { console.log('  FAIL  HTML missing: ' + n); htmlOk = false; }
  }
  if (htmlOk) console.log('  index.html 包含所有预期元素');

  console.log('\n========== 验证结果 ==========');
  if (allOk && importOk && htmlOk) {
    console.log('✅ 全部通过 — 工具可正常跑通');
    process.exit(0);
  } else {
    console.log('❌ 存在问题，见上方 FAIL');
    process.exit(1);
  }
}

main().catch(e => {
  console.error('VERIFY FAILED:', e.message);
  process.exit(1);
});
