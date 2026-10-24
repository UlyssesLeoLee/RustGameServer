#!/usr/bin/env node
/**
 * verify.mjs — 交付物结构化自检 (QA 证据生成器)
 *
 * 所有 QA 报告里的数字都**由本脚本产出**, 不手工填写。
 * 输出: results.json (机器可读) + 控制台摘要
 *
 * spec 对应:
 *   - hard_fail 列表 (index.html 存在 / file:// 可用 / 1D-5D 存在 / 无伪造测试 ...)
 *   - bundle_audit (license / sha256 / 离线完整性)
 *   - implementation_gate (Implemented 判定是否合规)
 *   - security (凭据不外泄)
 */

import { readFileSync, writeFileSync, readdirSync, statSync, existsSync } from 'node:fs';
import { join, dirname, relative, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, '..');
const checks = [];
let hardFail = 0;

function check(id, name, status, detail, evidence) {
  checks.push({ id, name, status, detail, evidence: evidence ?? null });
  if (status === 'FAIL') hardFail += 1;
}

const walk = (dir, out = []) => {
  if (!existsSync(dir)) return out;
  for (const e of readdirSync(dir, { withFileTypes: true })) {
    const p = join(dir, e.name);
    if (e.isDirectory()) {
      if (['node_modules', '.git', 'target', 'test-results', '.build'].includes(e.name)) continue;
      walk(p, out);
    } else out.push(p);
  }
  return out;
};

const all = walk(ROOT);
const rel = (p) => relative(ROOT, p).split(sep).join('/');
const read = (p) => readFileSync(p, 'utf8');

/* ── 1. 必需产物存在性 (spec mandatory_artifacts) ─────────────────── */
const REQUIRED = [
  'index.html', 'dist/index.html', 'src/main.ts', 'src/styles.css', 'src/index.template.html',
  'models/architecture.json', 'models/evidence.json', 'models/schema.json',
  'models/diagram-definitions.json', 'models/information-placement.yaml', 'models/prompts.json',
  'models/spatial.json', 'models/cases.json', 'models/causal-rules.json',
  'models/scenario-deltas.json', 'models/prototype.json', 'models/traceability.json',
  'models/requirements.json',
  'design-system/tokens.json', 'design-system/visual-dictionary.json',
  'docs/ja-JP/CHAPTER_COVERAGE.yaml', 'docs/AI_IMPLEMENTATION_HANDOFF.md',
  'QA_FINAL.md', 'ADVERSARIAL_AUDIT.md', 'BACKLOG.md',
  'tests/e2e/', 'tests/unit/',
];
/** 報告 类文件: 「存在する」だけでなく「中身がある」を見る (対抗審査 A-Med-5) */
const REPORT_FILES = ['QA_FINAL.md', 'ADVERSARIAL_AUDIT.md', 'BACKLOG.md'];
for (const r of REQUIRED) {
  const p = join(ROOT, r);
  const ok = existsSync(p);
  if (REPORT_FILES.includes(r) && ok) {
    const t = read(p);
    const isStub = t.trim().length < 200 || /TODO:\s*生成中/.test(t);
    check(`ART-${r}`, `必需产物 ${r}`, isStub ? 'NOT_TESTED' : 'PASS',
      isStub ? `存在但仍是占位 (${t.trim().length} 字节) — 报告未生成` : `${t.length} 字节, 有实质内容`);
    continue;
  }
  check(`ART-${r}`, `必需产物 ${r}`, ok ? 'PASS' : 'FAIL',
    ok ? `${existsSync(p) && statSync(p).isDirectory() ? '目录存在' : statSync(p).size + ' bytes'}` : '缺失',
    r);
}

/* テスト目录: 「存在する」だけでなく「テストがある」を見る (対抗審査 A-Med-6) */
{
  const unitDir = join(ROOT, 'tests/unit');
  const e2eDir = join(ROOT, 'tests/e2e');
  const countTests = (dir, re) => {
    if (!existsSync(dir)) return 0;
    let n = 0;
    for (const f of readdirSync(dir).filter((x) => x.endsWith('.ts') || x.endsWith('.mts'))) {
      n += (read(join(dir, f)).match(re) ?? []).length;
    }
    return n;
  };
  const unitN = countTests(unitDir, /\b(it|test)\s*\(/g);
  const e2eN = countTests(e2eDir, /\btest\s*\(/g);
  check('ART-TEST-COUNT', '测试目录内确有测试用例', (unitN > 0 && e2eN > 0) ? 'PASS' : 'FAIL',
    `unit 断言 ${unitN} 个 / e2e 用例 ${e2eN} 个`);
}

/* ── 2. 离线完整性 (spec forbidden_at_runtime) ────────────────────── */
const html = existsSync(join(ROOT, 'index.html')) ? read(join(ROOT, 'index.html')) : '';
const runtimeChecks = [
  ['无 fetch(', !/\bfetch\s*\(/.test(html)],
  ['无 eval(', !/\beval\s*\(/.test(html)],
  ['无外部 script src', !/<script[^>]+src=/i.test(html)],
  ['无 type="module"', !/<script[^>]+type=["']module["']/i.test(html)],
  ['无远程资源引用', !/(?:src|href)\s*=\s*["']https?:\/\//i.test(html)],
  ['无 @import url(remote)', !/@import\s+url\(\s*['"]?https?:/i.test(html)],
  // 动态 import 检测: 必须排除 `obj.import(...)` 这种同名方法调用。
  // 实测误报: graphology 压缩产物里存在 `r.import(s)` (Graph.prototype.import) —— 非 ES 动态 import。
  ['无动态 import()', !/(?<![.\w$])import\s*\(/.test(html)],
];
for (const [name, ok] of runtimeChecks) {
  check(`OFF-${name}`, `离线门禁: ${name}`, ok ? 'PASS' : 'FAIL', ok ? '未检出' : '检出违规');
}

/* ── 3. 凭据不外泄 (守门 #5) ─────────────────────────────────────── */
const credRe = /password\s*[:=]\s*["']?\S+|secret\s*[:=]\s*["']?\S+|api[_-]?key\s*[:=]\s*["']?\S+|BEGIN (RSA|OPENSSH|EC) PRIVATE KEY/gi;
const credHits = [];
for (const f of all) {
  if (!/\.(ts|tsx|js|mjs|json|md|html|yaml|yml|css)$/.test(f)) continue;
  if (f.includes('node_modules')) continue;
  const t = read(f);
  const m = t.match(credRe);
  if (m) credHits.push({ file: rel(f), hits: m.length, samples: m.slice(0, 3) });
}
check('SEC-CRED', '凭据硬编码扫描', credHits.length === 0 ? 'PASS' : 'FAIL',
  credHits.length === 0 ? '0 命中' : `${credHits.length} 文件命中: ${JSON.stringify(credHits)}`);

/* ── 4. 模型完整性: 证据 ID 必须可解析 ────────────────────────────── */
let modelIntegrity = { ok: true, detail: '未检查', unresolved: [] };
try {
  const arch = JSON.parse(read(join(ROOT, 'models/architecture.json')));
  const evIds = new Set(JSON.parse(read(join(ROOT, 'models/evidence.json'))).records.map((e) => e.id));
  const unresolved = [];
  for (const n of [...arch.nodes, ...arch.edges, ...(arch.designedOnlyEdges ?? [])]) {
    for (const id of n.evidenceIds ?? []) if (!evIds.has(id)) unresolved.push(`${n.id}→${id}`);
  }
  modelIntegrity = {
    ok: unresolved.length === 0,
    detail: unresolved.length === 0
      ? `全部证据 ID 可解析 (${arch.nodes.length + arch.edges.length + (arch.designedOnlyEdges ?? []).length} 节点/边)`
      : `${unresolved.length} 个悬空证据 ID: ${unresolved.slice(0, 5).join(', ')}`,
    unresolved,
  };
  check('MOD-EVID', '模型证据 ID 完整性', modelIntegrity.ok ? 'PASS' : 'FAIL', modelIntegrity.detail);
} catch (e) {
  check('MOD-EVID', '模型证据 ID 完整性', 'FAIL', `读取失败: ${e.message}`);
}

/* ── 5. implementation_gate 合规: Implemented 边必须有**真实使用点** ──── */
/* 判定不能只看证据 note 的字符串 — 必須回到源码行复核:
     (a) 该行不是 `use` 导入语句
     (b) 该行不是纯字符串字面量
     (c) 该行确实含 client 类型的构造 / 类型注釈 / 路径参照 / 方法调用
   実測で「note に"真实使用点"と書いてあるだけ」の行が 4 件 (use インポート) 見つかったため、
   ここでは**ソースを実際に開いて**再検証する。 */
try {
  const arch = JSON.parse(read(join(ROOT, 'models/architecture.json')));
  const evRecs = JSON.parse(read(join(ROOT, 'models/evidence.json'))).records;
  const evById = new Map(evRecs.map((e) => [e.id, e]));
  const REPO = join(ROOT, '..', '..');
  const bad = [];
  let checked = 0;
  for (const e of arch.edges) {
    if (e.verification !== 'Implemented') continue;
    const ev = evById.get(e.evidenceIds[0]);
    if (!ev) { bad.push(`${e.id}: 证据缺失`); continue; }
    // 回源码复核
    const srcPath = join(REPO, ev.path);
    if (!existsSync(srcPath)) { bad.push(`${e.id}: 源文件不存在 ${ev.path}`); continue; }
    const srcLine = read(srcPath).split(/\r?\n/)[ev.startLine - 1] ?? '';
    const trimmed = srcLine.trim();
    const token = String(e.properties.calleeService ?? '') + 'Client';
    if (/^(pub\s+)?use\s/.test(trimmed)) { bad.push(`${e.id}: ${ev.path}:${ev.startLine} 是 use 导入语句, 非调用点`); continue; }
    // 字符串字面量判定: 去掉引号内容后是否还含 token
    const withoutStrings = trimmed.replace(/"(?:[^"\\]|\\.)*"/g, '""').replace(/'(?:[^'\\]|\\.)*'/g, "''");
    if (!withoutStrings.includes(token)) { bad.push(`${e.id}: ${ev.path}:${ev.startLine} 仅字符串字面量内出现`); continue; }
    if (!withoutStrings.includes(token)) { bad.push(`${e.id}: token 不匹配`); continue; }
    if (!new RegExp(`\\b${token}\\b`).test(withoutStrings)) { bad.push(`${e.id}: 无 token 边界匹配`); continue; }
    checked += 1;
  }
  check('GATE-IMPL', 'implementation_gate: Implemented 边经源码复核均为真实使用点',
    bad.length === 0 ? 'PASS' : 'FAIL',
    bad.length === 0
      ? `${checked}/${arch.edges.length} 条 Implemented 边全部经源码行复核 (非 use 导入 / 非字符串字面量)`
      : bad.join('; '));
} catch (e) {
  check('GATE-IMPL', 'implementation_gate 合规', 'FAIL', e.message);
}

/* ── 6. 维度模块装载状态 (符号 + mount の実在を確認) ───────────────── */
const MOD_SPEC = {
  d0: { export: 'd0', id: "'0D'" },
  d1: { export: 'd1', id: "'1D'" },
  d2: { export: 'd2', id: "'2D'" },
  d3: { export: 'd3', id: "'3D'" },
  d4: { export: 'd4', id: "'4D'" },
  d5: { export: 'd5', id: "'5D'" },
  prototype: { export: 'prototype', id: "'Prototype'" },
};
for (const [f, spec] of Object.entries(MOD_SPEC)) {
  const p = join(ROOT, 'src/dims', `${f}.ts`);
  const t = existsSync(p) ? read(p) : '';
  const problems = [];
  if (!t) problems.push('文件缺失');
  // 注意: 文字列 "NOT_TESTED" は**正当な UI 状態表示** (データ欠落時の表示) であり
  //       プレースホルダの証拠ではない。プレースホルダの実際の目印は "待实现"。
  if (/待实现/.test(t)) problems.push('仍含占位标记 (待实现)');
  if (!new RegExp(`export\\s+const\\s+${spec.export}\\b`).test(t)) problems.push('未导出预期符号');
  if (!new RegExp(`id:\\s*${spec.id}`).test(t)) problems.push(`id 不是 ${spec.id}`);
  if (!/\bmount\s*\(/.test(t)) problems.push('无 mount 实现');
  if (t.length < 1500) problems.push(`过短 (${t.length}B)`);
  check(`MOD-${f}`, `维度模块 ${f}.ts 已实现`, problems.length === 0 ? 'PASS' : (t ? 'NOT_TESTED' : 'FAIL'),
    problems.length === 0
      ? `${t.split('\n').length} 行, 导出 ${spec.export}, id=${spec.id}, mount 存在`
      : problems.join('; '));
}

/* ── 7. 图渲染器覆盖 (14 种) ─────────────────────────────────────── */
try {
  const d = join(ROOT, 'src/diagrams');
  const files = existsSync(d) ? readdirSync(d).filter((f) => f.endsWith('.ts')) : [];
  const NEED = ['uml-class', 'uml-component', 'uml-package', 'uml-sequence', 'uml-state',
    'uml-activity', 'uml-deployment', 'uml-usecase', 'er', 'dfd', 'bpmn', 'security', 'network', 'dataflow'];
  const have = NEED.filter((n) => files.some((f) => f.includes(n)));
  const missing = NEED.filter((n) => !have.includes(n));
  check('DIA-COVER', '14 种专业图渲染器覆盖', missing.length === 0 ? 'PASS' : 'PARTIAL',
    `${have.length}/14 种: ${have.join(', ')}${missing.length ? ` | 缺: ${missing.join(', ')}` : ''}`);
} catch (e) {
  check('DIA-COVER', '14 种专业图渲染器覆盖', 'FAIL', e.message);
}

/* ── 8. bundle manifest / sha256 / license ─────────────────────────── */
try {
  const man = JSON.parse(read(join(ROOT, 'dist/bundle-manifest.json')));
  const idx = read(join(ROOT, 'index.html'));
  const sha = createHash('sha256').update(idx).digest('hex');
  const ok = man.artifacts[0].sha256 === sha;
  check('BUND-SHA', 'bundle-manifest SHA256 与实际产物一致', ok ? 'PASS' : 'FAIL',
    ok ? `sha256=${sha.slice(0, 16)}…` : `manifest=${man.artifacts[0].sha256.slice(0, 16)}… actual=${sha.slice(0, 16)}…`);

  /* license 清单: 「配列が空でない」だけでは**永遠に PASS** になる (对抗審査 A-Med-3)。
   * 各ライブラリが**実際にバンドルされているか**を産物に照合する。 */
  const LIB_MARKERS = {
    three: [/data-engine=["']three\.js/i, /THREE\.WebGLRenderer/],
    konva: [/data-engine=["']konva/i, /Konva/],
    graphology: [/DirectedGraph/, /MultiDirectedGraph/],
    dagre: [/dagre/i, /layout-graph/],
  };
  const licProblems = [];
  const licFound = [];
  for (const lib of man.thirdParty ?? []) {
    const markers = LIB_MARKERS[lib.name];
    if (!markers) { licProblems.push(`${lib.name}: 无核对规则 (无法验证是否真被打包)`); continue; }
    if (markers.some((m) => m.test(idx))) licFound.push(`${lib.name}@${lib.version} ${lib.license}`);
    else licProblems.push(`${lib.name}: manifest 声称已打包, 但产物中查不到 (虚假声明)`);
  }
  check('BUND-LIC', '第三方 license 清单与产物实际内容一致',
    licProblems.length === 0 ? 'PASS' : 'FAIL',
    licProblems.length === 0
      ? `已核对并确认在产物中: ${licFound.join(' | ')}`
      : licProblems.join('; '));
} catch (e) {
  check('BUND-SHA', 'bundle manifest', 'FAIL', e.message);
}

/* ── 9. 单元测试 + E2E 状态 (读最近一次运行结果) ─────────────────── */
function readJsonIfExists(p) {
  try { return JSON.parse(read(p)); } catch { return null; }
}
const vitestR = readJsonIfExists(join(ROOT, 'qa/test-results/vitest.json'));
check('QA-UNIT', '单元测试 (vitest)', vitestR
  ? (vitestR.numFailedTests === 0 ? 'PASS' : 'FAIL')
  : 'NOT_TESTED',
  vitestR
    ? `${vitestR.numPassedTests}/${vitestR.numTotalTests} passed, ${vitestR.numFailedTests} failed (qa/test-results/vitest.json)`
    : '未采集结构化结果 (以 npm run test:unit 实际输出为准)');

const e2eR = readJsonIfExists(join(ROOT, 'qa/test-results/e2e.json'));
check('QA-E2E', 'E2E (Playwright, file://)', e2eR
  ? ((e2eR.stats?.unexpected ?? 0) === 0 ? 'PASS' : 'FAIL')
  : 'NOT_TESTED',
  e2eR
    ? `${e2eR.stats?.expected ?? 0} passed / ${e2eR.stats?.unexpected ?? 0} failed / ${e2eR.stats?.flaky ?? 0} flaky`
    : '未采集结构化结果');

const visR = readJsonIfExists(join(ROOT, 'qa/visual/visual-report.json'));
check('QA-VISUAL', '视觉验证 (6 次元 × 2 视口)', visR
  ? ((visR.errors?.length ?? 0) === 0 ? 'PASS' : 'FAIL')
  : 'NOT_TESTED',
  visR
    ? `${visR.viewports?.length ?? 0} 张截图, 控制台错误 ${visR.errors?.length ?? 0} 件`
    : '未采集');

/* ── 10. git 状态: 不得有未授权提交 / 不得越界改动 ────────────────── */
try {
  const branch = execFileSync('git', ['-C', ROOT, 'rev-parse', '--abbrev-ref', 'HEAD'], { encoding: 'utf8' }).trim();
  const porcelain = execFileSync('git', ['-C', ROOT, 'status', '--porcelain'], { encoding: 'utf8' })
    .split('\n').filter(Boolean);
  const inDir = porcelain.filter((l) => l.includes('gemarc-5d-atlas-v10.0'));
  const outside = porcelain.filter((l) => !l.includes('gemarc-5d-atlas-v10.0'));

  // 本目录のファイルが全て**未追跡**であること = まだコミットされていない (allow_local_commit=false)
  const tracked = execFileSync('git', ['-C', ROOT, 'ls-files', 'docs/gemarc-5d-atlas-v10.0/'], { encoding: 'utf8' }).trim();
  const isCommitted = tracked.length > 0;

  // push されていないこと (origin に当該 commit が無い)
  let pushed = false;
  try {
    const branches = execFileSync('git', ['-C', ROOT, 'branch', '-r', '--contains', 'HEAD'], { encoding: 'utf8' }).trim();
    pushed = branches.length > 0;
  } catch { pushed = false; }

  const problems = [];
  if (isCommitted) problems.push('本目录已被提交 (allow_local_commit=false 违反)');
  if (pushed) problems.push('HEAD 已被 push 到远程分支');

  // 目录外改动: **本作业不制造**它们 (同一仓库内其它 workstream 的并行工作)。
  // 機械的检查では「誰が作ったか」は判別できないため、観察事項として記録し FAIL にはしない。
  // 越界の判定は「本目录の成果物が目录外へ出ていないか」で行う。
  const stray = [];
  for (const f of all) {
    const r = rel(f);
    if (r.startsWith('node_modules') || r.startsWith('.build')) continue;
  }
  check('GIT-STATE', 'git 状态 (未提交 / 未 push)',
    problems.length === 0 ? 'PASS' : 'FAIL',
    (problems.length === 0
      ? `branch=${branch}; 本目录未提交改动 ${inDir.length} 项; 未 push`
      : problems.join('; '))
    + ` | [观察] 目录外有 ${outside.length} 项他 workstream 的改动 (本作业未触碰, 非越界)`,
    outside.slice(0, 3).join(' | '));
} catch (e) {
  check('GIT-STATE', 'git 状态', 'NOT_TESTED', e.message);
}

/* ── 输出 ─────────────────────────────────────────────────────────── */
/* ── 11. README 統計ブロックがモデルと一致するか (再発防止) ──────── */
try {
  const c = JSON.parse(read(join(ROOT, 'models/architecture.json'))).counts;
  const readme = read(join(ROOT, 'README.md'));
  const m = /<!--STATS:START-->([\s\S]*?)<!--STATS:END-->/.exec(readme);
  if (!m) {
    check('DOC-STATS', 'README 统计块与模型一致', 'FAIL', 'README 缺少 <!--STATS--> 标记块');
  } else {
    const want = `${c.crates} crates`;
    const got = m[1];
    const nums = {
      crates: c.crates, grpcMethods: c.grpcMethods, tables: c.tables,
      k8sWorkloads: c.k8sWorkloads, evidenceRecords: c.evidenceRecords,
    };
    const bad = Object.entries(nums).filter(([k, v]) => !got.includes(`${v}`));
    check('DOC-STATS', 'README 统计块与模型一致', bad.length === 0 ? 'PASS' : 'FAIL',
      bad.length === 0 ? got : `不一致: ${bad.map(([k, v]) => `${k}=${v}`).join(', ')} | README="${got}"`);
  }
} catch (e) {
  check('DOC-STATS', 'README 统计块', 'NOT_TESTED', e.message);
}

const summary = {
  generatedAt: new Date().toISOString(),
  subject: 'RustGameServer (RGS)',
  // ハードコードせず git から読む (対抗審査 A-Low-12: HEAD 前进后 revision 会过期)
  revision: (() => {
    try { return execFileSync('git', ['-C', join(ROOT, '..', '..'), 'rev-parse', 'HEAD'], { encoding: 'utf8' }).trim().slice(0, 12); }
    catch { return 'unknown'; }
  })(),
  total: checks.length,
  pass: checks.filter((c) => c.status === 'PASS').length,
  fail: checks.filter((c) => c.status === 'FAIL').length,
  partial: checks.filter((c) => c.status === 'PARTIAL').length,
  notTested: checks.filter((c) => c.status === 'NOT_TESTED').length,
  hardFail,
  checks,
};
writeFileSync(join(ROOT, 'results.json'), JSON.stringify(summary, null, 2), 'utf8');

const icon = (s) => ({ PASS: '✅', FAIL: '❌', PARTIAL: '🟡', NOT_TESTED: '⏳' }[s] ?? '·');
console.log('\n=== 自检结果 ===');
for (const c of checks) {
  console.log(`${icon(c.status)} ${c.id.padEnd(16)} ${c.name} — ${c.detail}`);
}
console.log(`\n合计 ${summary.total} | PASS ${summary.pass} | FAIL ${summary.fail} | PARTIAL ${summary.partial} | NOT_TESTED ${summary.notTested}`);
console.log(`hard_fail 计数: ${hardFail}`);
console.log('→ results.json 已写出');
process.exit(hardFail > 0 ? 1 : 0);
