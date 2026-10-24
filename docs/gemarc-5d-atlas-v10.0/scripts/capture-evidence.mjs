#!/usr/bin/env node
/**
 * capture-evidence.mjs — RGS (RustGameServer) 真实证据采集器
 *
 * 目的: 为 5D Atlas 的 models/ 层产出**可复核**的证据与工程模型.
 * 规则 (per Universal GemArc 5D Design Atlas v10.0 `implementation_gate`):
 *   - Implemented 必须带 path:startLine-endLine + revision
 *   - 只有 pom/依赖/目录/接口定义/DTO/Mock/TODO **不能**证明业务行为已实现
 *   - 没有源码不能把 Unknown 判成 Absent
 *
 * 用法: node scripts/capture-evidence.mjs [--repo <path>] [--out <dir>]
 * 输出: models/evidence.json, models/architecture.json
 *        (schema.json は scripts/generate-meta-models.mjs が生成する)
 */

import { readFileSync, writeFileSync, readdirSync, statSync, existsSync } from 'node:fs';
import { join, relative, dirname, sep } from 'node:path';
import { execFileSync } from 'node:child_process';

const argv = process.argv.slice(2);
const argOf = (k, d) => { const i = argv.indexOf(k); return i >= 0 ? argv[i + 1] : d; };

const REPO = argOf('--repo', join(process.cwd(), '..', '..'));
const OUT = argOf('--out', join(process.cwd(), 'models'));

/* ── git 事实 ─────────────────────────────────────────────────────────── */
const git = (...args) =>
  execFileSync('git', ['-C', REPO, ...args], { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 }).trim();

const REVISION = git('rev-parse', 'HEAD');
const SHORT_REV = REVISION.slice(0, 12);
const BRANCH = git('rev-parse', '--abbrev-ref', 'HEAD');
const REMOTES = git('remote', '-v').split('\n').filter(Boolean)
  .map(l => { const [name, url] = l.split(/\s+/); return { name, url: url.replace(/\s+\(.*\)$/, '') }; });

/* ── 路径工具 ─────────────────────────────────────────────────────────── */
const rel = (abs) => relative(REPO, abs).split(sep).join('/');

function walk(dir, filter, out = []) {
  if (!existsSync(dir)) return out;
  for (const e of readdirSync(dir, { withFileTypes: true })) {
    const p = join(dir, e.name);
    if (e.isDirectory()) {
      if (e.name === 'target' || e.name === 'node_modules' || e.name === '.git') continue;
      walk(p, filter, out);
    } else if (!filter || filter(p)) out.push(p);
  }
  return out;
}

const linesOf = (p) => readFileSync(p, 'utf8').split(/\r?\n/);

/**
 * stripRustComments — 逐行剥离 Rust 注释, 保留行号对齐 (返回同长度的数组, 注释位替换为等长空白)。
 *
 * 为什么必须做: 直接正则匹配 `XServiceClient` 会命中
 *   - `/// - 封装 ...CardServiceClient`      (文档注释)
 *   - `// MERGE (...) via: "replay_proto::replay_service_client::ReplayServiceClient::new"` (内嵌脚本注释)
 * 这类"注释里的意图"**不是**已实现的调用, 若标 Implemented 就是伪造实现证据
 * (per v10.0 `implementation_gate`: 只有 pom/依赖/目录/接口定义/DTO/Mock/TODO 都不能证明行为已实现,
 *  注释比这些更弱)。
 *
 * 处理: 行注释 (斜杠斜杠, 含文档注释) ; 块注释 (斜杠星号 ... 星号斜杠, 可跨行) ;
 *       字符串 "..." ; 原生字符串 r#"..."# ; 字符 'x'
 *       字符串里的双斜杠 (如 "http://x") 不算注释。
 */
function stripRustComments(srcLines) {
  const out = new Array(srcLines.length);
  let inBlock = false;
  for (let i = 0; i < srcLines.length; i++) {
    const line = srcLines[i];
    let res = '';
    let j = 0;
    while (j < line.length) {
      if (inBlock) {
        const end = line.indexOf('*/', j);
        if (end === -1) { res += ' '.repeat(line.length - j); j = line.length; }
        else { res += ' '.repeat(end - j + 2); j = end + 2; inBlock = false; }
        continue;
      }
      const ch = line[j];
      // 原始字符串 r#"..."# / r##"..."##
      if (ch === 'r' && /r(#+)?"/.test(line.slice(j, j + 4))) {
        const m = /^r(#+)?"/.exec(line.slice(j));
        const hashes = m[1] || '';
        const close = '"' + hashes;
        const end = line.indexOf(close, j + m[0].length);
        if (end === -1) { res += line.slice(j); j = line.length; }
        else { res += line.slice(j, end + close.length); j = end + close.length; }
        continue;
      }
      if (ch === '"') {
        let k = j + 1;
        while (k < line.length) {
          if (line[k] === '\\') { k += 2; continue; }
          if (line[k] === '"') { k++; break; }
          k++;
        }
        res += line.slice(j, k); j = k; continue;
      }
      if (ch === "'") {
        // 字符字面量或生命周期 ('a); 生命周期不应吞掉后续内容
        const m = /^'(\\.|[^\\'])'/.exec(line.slice(j));
        if (m) { res += m[0]; j += m[0].length; continue; }
        res += ch; j++; continue;
      }
      if (ch === '/' && line[j + 1] === '/') { res += ' '.repeat(line.length - j); j = line.length; continue; }
      if (ch === '/' && line[j + 1] === '*') {
        inBlock = true; res += '  '; j += 2; continue;
      }
      res += ch; j++;
    }
    out[i] = res;
  }
  return out;
}

function lineOfMatch(content, re, from = 0) {
  for (let i = from; i < content.length; i++) if (re.test(content[i])) return i + 1;
  return -1;
}

/**
 * stripRustStrings — 文字列リテラル ("..." / r#"..."#) を空白に置換する。
 * コードから取り除いた**コードだけ**残す (コメントは別関数で処理)。
 *
 * なぜ必要か: `&"<Arc<Mutex<Option<EconomyServiceClient<Channel>>>>>"` のような
 * 文字列リテラル内にも型名が出現しうる。文字列内の言及は「使用点」ではない。
 */
function stripRustStrings(srcLines) {
  const out = new Array(srcLines.length);
  for (let i = 0; i < srcLines.length; i++) {
    const line = srcLines[i];
    let res = '';
    let j = 0;
    while (j < line.length) {
      const ch = line[j];
      if (ch === 'r' && /r(#+)?"/.test(line.slice(j, j + 4))) {
        const m = /^r(#+)?"/.exec(line.slice(j));
        const hashes = m[1] || '';
        const close = '"' + hashes;
        const end = line.indexOf(close, j + m[0].length);
        if (end === -1) { res += ' '.repeat(line.length - j); j = line.length; }
        else { res += ' '.repeat(end + close.length - j); j = end + close.length; }
        continue;
      }
      if (ch === '"') {
        let k = j + 1;
        while (k < line.length) {
          if (line[k] === '\\') { k += 2; continue; }
          if (line[k] === '"') { k++; break; }
          k++;
        }
        res += ' '.repeat(k - j); j = k; continue;
      }
      if (ch === "'") {
        const m = /^'(\\.|[^\\'])'/.exec(line.slice(j));
        if (m) { res += ' '.repeat(m[0].length); j += m[0].length; continue; }
      }
      res += ch; j++;
    }
    out[i] = res;
  }
  return out;
}

/* ── SQL コメント除去 ──────────────────────────────────────────────── */
/**
 * stripSqlComments — SQL の行コメント (ハイフン 2 連) とブロックコメント
 * (スラッシュ+アスタリスク ... アスタリスク+スラッシュ) を除去する。
 *
 * なぜ必要か: 生の正規表現で `CREATE TABLE` を拾うと、
 *   `--   - CREATE TABLE IF NOT EXISTS 互換 fresh DB ...` のような
 *   **コメント行**も拾われ、表名が "IF" という**存在しないエンティティ**が生成される。
 *   実測で 24 個の幻影テーブル (name = "IF") が発生していた (対抗審査で発覚)。
 *
 * 此外 SQL の字符串リテラル '...' 内のCREATE TABLE も除外する。
 */
function stripSqlComments(src) {
  let out = '';
  let i = 0;
  const n = src.length;
  while (i < n) {
    const c = src[i];
    const c2 = src[i + 1];
    if (c === '-' && c2 === '-') {
      while (i < n && src[i] !== '\n') i++;
      continue;
    }
    if (c === '/' && c2 === '*') {
      i += 2;
      while (i < n && !(src[i] === '*' && src[i + 1] === '/')) i++;
      i += 2;
      // 改行は残す (行番号を保つ)
      out += ' ';
      continue;
    }
    if (c === "'") {
      // 文字列リテラル: 内容を空白に置換 (CREATE TABLE を含んでいても拾わない)
      i++;
      while (i < n) {
        if (src[i] === "'" && src[i + 1] === "'") { i += 2; continue; }  // '' は escape
        if (src[i] === "'") { i++; break; }
        i++;
      }
      out += " '' ";
      continue;
    }
    if (c === '\n') { out += '\n'; i++; continue; }
    out += c; i++;
  }
  return out;
}

/* ── 证据容器 ─────────────────────────────────────────────────────────── */
const evidence = [];
let evSeq = 0;
function addEvidence({ kind, path, startLine, endLine, revision = REVISION, note, snippet }) {
  evSeq += 1;
  const id = `EV-${String(evSeq).padStart(4, '0')}`;
  const rec = {
    id,
    kind,                                     // SourceCode | TestTrace | RuntimeLog | Document | PublicObservation | UserRequirement
    fileLine: `${path}:${startLine}-${endLine}`,
    path,
    startLine,
    endLine,
    revision: revision.slice(0, 12),
    note,
  };
  if (snippet) rec.snippet = snippet.slice(0, 400);
  evidence.push(rec);
  return rec;
}

/* ── 业务域归属 (per AGENTS.md §0: 6 域 + 平台层 + 工具 crate) ─────────── */
const DOMAIN_OF = {
  'match-service': 'Core', 'battle-service': 'Core', 'card-service': 'Core',
  'replay-service': 'Core', 'replay-extra-service': 'Core', 'scene-service': 'Core',
  'pvp-full-service': 'Core',
  'network-gateway': 'Interaction', 'gm-backend': 'Interaction', 'gm-extra-service': 'Interaction',
  'player-service': 'Data', 'economy-service': 'Data', 'guild-service': 'Data',
  'social-service': 'Data', 'social-extra-service': 'Data',
  'leaderboard-service': 'Data', 'leaderboard-extra-service': 'Data', 'activity-service': 'Data',
  'admin-service': 'Security', 'i18n-service': 'Security',
  'shared-platform': 'Infrastructure', 'cluster-ops': 'Infrastructure', 'function-plane': 'Infrastructure',
  'rgs-arc-olu': 'External', 'rgs-certgen': 'External', 'rgs-hello': 'External',
  'rgs-asset-download': 'External', 'rgs-overflow-alert': 'External',
  'rgs-testkit': 'Operations', 'rgs-flash-mock': 'Operations',
};
/** 域归属的证据: AGENTS.md §0 原文行 */
const AGENTS_DOMAIN_EVIDENCE = (() => {
  const p = join(REPO, 'AGENTS.md');
  if (!existsSync(p)) return null;
  const L = linesOf(p);
  const i = L.findIndex(l => l.includes('6 域') || (l.includes('架构') && l.includes('**6 域**')));
  if (i < 0) return null;
  return addEvidence({
    kind: 'Document', path: 'AGENTS.md', startLine: i + 1, endLine: i + 1,
    note: '业务域划分 (6 域 + 平台层 + 工具 crate) 原文出处', snippet: L[i],
  });
})();

/* ── 1. crates → Service 节点 ─────────────────────────────────────────── */
const nodes = [];
const edges = [];
const ports = [];
const containers = [];
let nodeSeq = 0, edgeSeq = 0, portSeq = 0;
const nid = (p) => `N-${String(++nodeSeq).padStart(3, '0')}-${p}`;

const crateDirs = readdirSync(join(REPO, 'crates'), { withFileTypes: true })
  .filter(d => d.isDirectory()).map(d => d.name).sort();

const crateMeta = [];
for (const crate of crateDirs) {
  const cargoPath = join(REPO, 'crates', crate, 'Cargo.toml');
  if (!existsSync(cargoPath)) continue;
  const L = linesOf(cargoPath);
  const nameLine = L.findIndex(l => /^\s*name\s*=/.test(l));
  const name = nameLine >= 0 ? (L[nameLine].match(/=\s*["']([^"']+)/) || [])[1] : crate;
  const verLine = L.findIndex(l => /^\s*version\s*=/.test(l));
  const version = verLine >= 0 ? (L[verLine].match(/=\s*["']([^"']+)/) || [])[1] : '0.0.0';

  const srcDir = join(REPO, 'crates', crate, 'src');
  const rsFiles = walk(srcDir, p => p.endsWith('.rs'));
  const testFiles = rsFiles.filter(p => /[\\/](tests?)[\\/]/.test(p) || /_test\.rs$/.test(p));
  let testFns = 0;
  for (const f of rsFiles) {
    const c = readFileSync(f, 'utf8');
    testFns += (c.match(/#\[(?:tokio::)?test\]/g) || []).length;
  }
  const ev = addEvidence({
    kind: 'SourceCode', path: rel(cargoPath), startLine: 1, endLine: L.length,
    note: `crate 清单 (name=${name}, version=${version}); src .rs=${rsFiles.length}, 测试 .rs=${testFiles.length}, #[test] 计数=${testFns}`,
  });

  crateMeta.push({ crate, name, version, rsFiles: rsFiles.length, testFiles: testFiles.length, testFns, evidenceId: ev.id });
}

/* ── 2. proto → Api 节点 + Port ───────────────────────────────────────── */
const protoFiles = walk(join(REPO, 'crates'), p => p.endsWith('.proto'));
const apiNodes = [];
let totalRpc = 0;

for (const pf of protoFiles) {
  const L = linesOf(pf);
  const pkgLine = L.findIndex(l => /^\s*package\s+/.test(l));
  const pkg = pkgLine >= 0 ? (L[pkgLine].match(/package\s+([\w.]+)/) || [])[1] : 'unknown';

  // 收集 service 块
  let i = 0;
  while (i < L.length) {
    if (!/^\s*service\s+(\w+)\s*\{/.test(L[i])) { i++; continue; }
    const svcName = (L[i].match(/service\s+(\w+)/) || [])[1];
    const svcStart = i + 1;
    let depth = 0, j = i, svcEnd = i;
    for (; j < L.length; j++) {
      depth += (L[j].match(/\{/g) || []).length;
      depth -= (L[j].match(/\}/g) || []).length;
      if (depth === 0 && j > i) { svcEnd = j + 1; break; }
    }
    const crate = rel(pf).split('/')[1];
    const ev = addEvidence({
      kind: 'SourceCode', path: rel(pf), startLine: svcStart, endLine: svcEnd,
      note: `gRPC service ${pkg}.${svcName} 定义 (crate=${crate})`,
    });
    const rpcs = [];
    for (let k = svcStart; k <= svcEnd; k++) {
      const m = L[k - 1].match(/^\s*rpc\s+(\w+)\s*\(\s*([\w.]+)\s*\)\s*returns\s*\(\s*([\w.]+)\s*\)/);
      if (m) rpcs.push({ name: m[1], req: m[2], res: m[3], line: k });
    }
    totalRpc += rpcs.length;
    apiNodes.push({
      id: nid(`api-${svcName}`), kind: 'Api', name: svcName,
      qualifiedName: `${pkg}.${svcName}`, role: 'gRPC 服务定义',
      domainId: DOMAIN_OF[crate] || 'Unknown', parentId: null,
      portIds: [], properties: {
        crate, package: pkg, rpcCount: rpcs.length,
        protoFile: rel(pf), serviceLineRange: [svcStart, svcEnd],
      },
      // 依据 spec implementation_gate: proto 只有接口定义 → 设计存在, 行为未证明
      verification: 'Designed',
      evidenceIds: [ev.id],
    });
    // rpc 暴露为 Port
    for (const r of rpcs) {
      const pev = addEvidence({
        kind: 'SourceCode', path: rel(pf), startLine: r.line, endLine: r.line,
        note: `rpc ${r.name}(${r.req}) returns (${r.res})`,
      });
      ports.push({
        id: `P-${String(++portSeq).padStart(4, '0')}`,
        name: r.name, apiId: null, crate,
        requestType: r.req, responseType: r.res,
        direction: 'caller->callee',
        type: 'grpc-rpc',
        properties: { line: r.line },
        evidenceIds: [pev.id],
      });
    }
    i = svcEnd;
  }
}

/* ── 3. migrations → Table 节点 ───────────────────────────────────────── */
/* 同一 crate に `migrations/` と `tests/migrations/` の両方に同名の表が居る場合がある。
   どちらも採ると qualifiedName が重複し、ER 図に同じ表が二重に出る。
   本番用 `migrations/` を優先し、テスト用は表名の重複Court として**数えない**。 */
const sqlFiles = walk(join(REPO, 'crates'), p => p.endsWith('.sql'))
  .sort((a, b) => {
    const at = a.includes(`${sep}tests${sep}migrations${sep}`) ? 1 : 0;
    const bt = b.includes(`${sep}tests${sep}migrations${sep}`) ? 1 : 0;
    return at - bt || a.localeCompare(b);
  });
const tableNodes = [];
const seenTableKeys = new Set();
let totalTables = 0;
let duplicateTablesSkipped = 0;

for (const sf of sqlFiles) {
  const rawSrc = readFileSync(sf, 'utf8');
  // コメント除去 (幻影テーブル "IF" の発生防止)。行番号を保つため改行は残す。
  const stripped = stripSqlComments(rawSrc);
  const L = stripped.split(/\r?\n/);
  const rawL = rawSrc.split(/\r?\n/);
  const crate = rel(sf).split('/')[1];
  for (let i = 0; i < L.length; i++) {
    const m = L[i].match(/CREATE\s+TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?([\w."]+)/i);
    if (!m) continue;
    const tname = m[1].replace(/"/g, '');
    // 防御: 実在しえない名前 (IF / NOT / EXISTS 等) は取り込まない
    if (/^(IF|NOT|EXISTS|TABLE)$/i.test(tname)) continue;
    // 重複 (tests/migrations 側) は数えない
    const tableKey = `${crate}::${tname}`;
    if (seenTableKeys.has(tableKey)) {
      duplicateTablesSkipped += 1;
      continue;
    }
    seenTableKeys.add(tableKey);
    // 收集列定义直到 ')'
    let depth = 0, end = i;
    const cols = [];
    for (let k = i; k < L.length; k++) {
      depth += (L[k].match(/\(/g) || []).length;
      depth -= (L[k].match(/\)/g) || []).length;
      // 列定义: 缩进 + 列名 + 其余全部
      // ⚠️ 不能用 "[A-Z][A-Z0-9 ]*" 贪心匹配类型 —— 那会把 "UUID PRIMARY KEY"
      //    的 "PRIMARY KEY" 吞进 type, 导致 pk 判定失效 (实测 pkText=57 但 pkFlag=0)。
      //    正确做法: 列名之后整段都算 "约束区", PK/FK/NOT NULL 都从这段里正则提取。
      const cm = L[k].match(/^\s{4,}"?(\w+)"?\s+(.+?)\s*[,;]?\s*$/);
      if (cm && k > i && !/^\s*(PRIMARY|FOREIGN|UNIQUE|CHECK|CONSTRAINT|INDEX|KEY)\b/i.test(L[k])) {
        const name = cm[1];
        const rest = cm[2];
        const typeMatch = /^([A-Za-z][A-Za-z0-9 ]*(?:\([^)]*\))?(?:\[\])?)/.exec(rest);
        const type = (typeMatch ? typeMatch[1] : rest).trim();
        const fkMatch = /REFERENCES\s+([\w."]+)/i.exec(rest);
        cols.push({
          name,
          type,
          pk: /PRIMARY\s+KEY/i.test(rest),
          fk: fkMatch ? fkMatch[1].replace(/"/g, '') : null,
          notNull: /NOT\s+NULL/i.test(rest),
          unique: /UNIQUE/i.test(rest),
        });
      }
      if (depth === 0 && k > i) { end = k + 1; break; }
    }
    const ev = addEvidence({
      kind: 'SourceCode', path: rel(sf), startLine: i + 1, endLine: end,
      note: `CREATE TABLE ${tname} (${cols.length} 列, PK=${cols.filter(c => c.pk).length}, FK=${cols.filter(c => c.fk).length})`,
    });
    totalTables += 1;
    tableNodes.push({
      id: nid(`tbl-${crate}-${tname}`), kind: 'Table', name: tname,
      qualifiedName: `${crate}.${tname}`, role: '持久化表定义',
      domainId: DOMAIN_OF[crate] || 'Unknown', parentId: null,
      portIds: [],
      properties: {
        crate, columns: cols,
        pkColumns: cols.filter(c => c.pk).map(c => c.name),
        fkColumns: cols.filter(c => c.fk).map(c => ({ column: c.name, references: c.fk })),
        migrationFile: rel(sf), lineRange: [i + 1, end],
      },
      verification: 'Designed',
      evidenceIds: [ev.id],
    });
  }
}

/* ── 4. 真实跨服务调用边 (Rust 源码中的 client 引用) ──────────────────── */
const CLIENT_RE = /\b([A-Z][A-Za-z0-9]*Service)Client\b/g;
const serviceNameToCrate = new Map();
for (const a of apiNodes) {
  const svc = a.name.replace(/Service$/, '');
  serviceNameToCrate.set(svc.toLowerCase(), a.properties.crate);
}

const srcRs = walk(join(REPO, 'crates'), p => p.endsWith('.rs') && /[\\/]src[\\/]/.test(p));
const callEdges = [];
const commentedMentions = [];   // 注释中出现的 client 名字 → 单列为 "设计意图" 证据, 不冒充已实现

for (const f of srcRs) {
  const raw = linesOf(f);
  const code = stripRustComments(raw);   // ← コメント除去 (実使用点の判定に使う)
  const codeOnly = stripRustStrings(code); // ← さらに文字列リテラルも除去 (**真**の実使用点)
  const callerCrate = rel(f).split('/')[1];
  for (let i = 0; i < code.length; i++) {
    const m = /([A-Z][A-Za-z0-9]*Service)Client/.exec(code[i]);
    if (!m) continue;
    const targetSvc = m[1].replace(/Service$/, '');
    const targetCrate = serviceNameToCrate.get(targetSvc.toLowerCase());
    if (!targetCrate || targetCrate === callerCrate) continue;

    const token = m[0];                                  // e.g. "AdminServiceClient"
    const svcWord = m[1];                                // e.g. "AdminService"

    /* ── 実装ゲート: 下の 3 条件すべてを満たす行だけを「実使用点」とする ──
     * spec implementation_gate:
     *   "只有 pom、依赖、目录、接口定义、DTO、Mock、TODO 不能证明该业务行为已实现"
     * したがって:
     *   (1) `use` インポート文 = 型が使えることの証明であって 呼び出し点 の証明ではない → 除外
     *   (2) 文字列リテラル内の言及 = 使用点ではない → 除外
     *   (3) 実使用の形態 (型注釈 / ::new 構築 / パス参照 / メソッド呼び出し) であること
     */
    if (/^\s*(pub\s+)?use\s/.test(codeOnly[i])) continue;      // (1)
    if (!codeOnly[i].includes(token)) continue;                 // (2) 文字列内のみ
    if (!new RegExp(`\\b${token}\\b`).test(codeOnly[i])) continue;

    // 実使用の形態を判定
    const isCtor = new RegExp(`\\b${token}::new\\b`).test(codeOnly[i]);
    const isTypeUse = !isCtor && new RegExp(`\\b${token}\\s*<`).test(codeOnly[i]);
    const isPathRef = !isCtor && !isTypeUse && new RegExp(`::${token}\\b`).test(codeOnly[i]);
    const isCallSite = !isCtor && !isTypeUse && !isPathRef
      && new RegExp(`\\b${token}\\b\\s*\\.`).test(codeOnly[i]);
    if (!isCtor && !isTypeUse && !isPathRef && !isCallSite) continue;  // (3)

    const usageForm = isCtor ? 'ctor' : isTypeUse ? 'type-annotation'
      : isPathRef ? 'path-ref' : 'call-site';

    const ev = addEvidence({
      kind: 'SourceCode', path: rel(f), startLine: i + 1, endLine: i + 1,
      note: `跨服务 client 真实使用点 ${token} (caller=${callerCrate} → callee=${targetCrate}; 形态=${usageForm})`,
      snippet: raw[i].trim(),
    });
    callEdges.push({
      id: `E-${String(++edgeSeq).padStart(3, '0')}`,
      sourceId: null, targetId: null,
      sourcePortId: null, targetPortId: null,
      type: 'grpc-call', directed: true, guardId: null,
      properties: {
        callerCrate, calleeCrate: targetCrate, calleeService: svcWord,
        fileLine: `${rel(f)}:${i + 1}`,
        usageForm,
      },
      // コメント・文字列リテラル・use インポートを除外した上で実使用点で確認 → 実装の証明
      verification: 'Implemented',
      evidenceIds: [ev.id],
    });
  }

  // コメント内 / 文字列リテラル内の言及 → 設計意図 (Inferred) として別掲
  for (let i = 0; i < raw.length; i++) {
    const m = /([A-Z][A-Za-z0-9]*Service)Client/.exec(raw[i]);
    if (!m) continue;
    const targetSvc = m[1].replace(/Service$/, '');
    const targetCrate = serviceNameToCrate.get(targetSvc.toLowerCase());
    if (!targetCrate || targetCrate === callerCrate) continue;
    if (code[i].includes(m[0])) continue;         // コード中出现 = 既に実使用点として採用済み
    const inString = !codeOnly[i].includes(m[0]); // 文字列内のみか
    commentedMentions.push({
      callerCrate, calleeCrate: targetCrate, calleeService: m[1],
      fileLine: `${rel(f)}:${i + 1}`,
      mentionKind: inString ? 'string-literal' : 'comment',
    });
  }
}


/* ── 5. k8s 部署 → Container (Building/Floor/Room) ─────────────────────── */
const deployDir = join(REPO, 'docs', 'deploy', '01-k8s-manifests');
const deployments = [];
if (existsSync(deployDir)) {
  for (const f of readdirSync(deployDir).filter(x => /\.ya?ml$/.test(x)).sort()) {
    const p = join(deployDir, f);
    const L = linesOf(p);
    for (let i = 0; i < L.length; i++) {
      if (!/^\s*kind:\s*Deployment\s*$/.test(L[i]) && !/^\s*kind:\s*StatefulSet\s*$/.test(L[i])) continue;
      const kind = L[i].match(/kind:\s*(\w+)/)[1];
      let name = null, ns = null, replicas = null;
      for (let k = i; k < Math.min(i + 30, L.length); k++) {
        if (!name && /^\s*name:\s*(\S+)/.test(L[k])) name = L[k].match(/name:\s*(\S+)/)[1];
        if (!ns && /namespace:\s*(\S+)/.test(L[k])) ns = L[k].match(/namespace:\s*(\S+)/)[1];
        if (!replicas && /^\s*replicas:\s*(\d+)/.test(L[k])) replicas = +L[k].match(/replicas:\s*(\d+)/)[1];
      }
      if (!name || /PLACEHOLDER/.test(name)) continue;
      const ev = addEvidence({
        kind: 'SourceCode', path: rel(p), startLine: i + 1, endLine: Math.min(i + 30, L.length),
        note: `k8s ${kind} ${name}${ns ? ` (ns=${ns})` : ''}${replicas ? ` replicas=${replicas}` : ''}`,
      });
      deployments.push({ name, kind, namespace: ns, replicas, manifest: rel(p), line: i + 1, evidenceId: ev.id });
    }
  }
}

/* ── 6. 组装 architecture.json ────────────────────────────────────────── */
const crateToNode = new Map();
for (const c of crateMeta) {
  const id = nid(`svc-${c.crate}`);
  crateToNode.set(c.crate, id);
  const dep = deployments.find(d => d.name === c.crate);
  nodes.push({
    id, kind: 'Service', name: c.crate,
    qualifiedName: `rgs::${c.name}`, role: 'Rust crate / 业务服务单元',
    domainId: DOMAIN_OF[c.crate] || 'Unknown', parentId: null,
    portIds: [],
    properties: {
      crate: c.crate, packageName: c.name, version: c.version,
      rustFiles: c.rsFiles, testFiles: c.testFiles, testFunctions: c.testFns,
      deployed: !!dep,
      deployment: dep ? { kind: dep.kind, namespace: dep.namespace, replicas: dep.replicas, manifest: dep.manifest } : null,
      domainBasis: AGENTS_DOMAIN_EVIDENCE ? AGENTS_DOMAIN_EVIDENCE.id : null,
    },
    // crate 存在且有源码 → 存在性 Present; 但"业务行为已实现"不由 crate 存在性证明
    verification: 'Designed',
    evidenceIds: [c.evidenceId, ...(dep ? [dep.evidenceId] : []), ...(AGENTS_DOMAIN_EVIDENCE ? [AGENTS_DOMAIN_EVIDENCE.id] : [])],
  });
}

// 父链: Service -> Api (同 crate), Api/Table -> Room
for (const a of apiNodes) {
  a.parentId = crateToNode.get(a.properties.crate) || null;
  const parent = nodes.find(n => n.id === a.parentId);
  if (parent) parent.properties.childApiIds = [...(parent.properties.childApiIds || []), a.id];
}
for (const t of tableNodes) {
  t.parentId = crateToNode.get(t.properties.crate) || null;
  const parent = nodes.find(n => n.id === t.parentId);
  if (parent) parent.properties.childTableIds = [...(parent.properties.childTableIds || []), t.id];
}

// 真实调用边接线
const apiByName = new Map(apiNodes.map(a => [a.properties.crate, a]));
for (const e of callEdges) {
  const src = crateToNode.get(e.properties.callerCrate);
  const tgt = crateToNode.get(e.properties.calleeCrate);
  e.sourceId = src; e.targetId = tgt;
  const tgtApi = apiByName.get(e.properties.calleeCrate);
  if (tgtApi) {
    const p = ports.find(pp => pp.crate === e.properties.calleeCrate && pp.name.toLowerCase() === e.properties.calleeService.toLowerCase());
    if (p) { p.apiId = tgtApi.id; e.targetPortId = p.id; }
  }
}

// 仅在注释中出现 → 设计意图边 (Designed), 明确区别于 Implemented
const designedEdges = [];
const seenPair = new Set();
for (const cm of commentedMentions) {
  const key = `${cm.callerCrate}->${cm.calleeCrate}`;
  if (seenPair.has(key)) continue;
  // 已被真实实现覆盖的对不再重复列为"仅意图"
  if (callEdges.some(e => e.properties.callerCrate === cm.callerCrate && e.properties.calleeCrate === cm.calleeCrate)) continue;
  seenPair.add(key);
  const ev = addEvidence({
    kind: 'SourceCode', path: cm.fileLine.split(':')[0], startLine: +cm.fileLine.split(':').pop(), endLine: +cm.fileLine.split(':').pop(),
    note: `⚠️ 仅在 ${cm.mentionKind === 'string-literal' ? '字符串字面量' : '注释/文档'}中出现 ${cm.calleeService}Client, **源码中无真实调用点** → 设计意图 (Inferred), 不得标 Implemented`,
  });
  designedEdges.push({
    id: `E-D${String(designedEdges.length + 1).padStart(3, '0')}`,
    sourceId: crateToNode.get(cm.callerCrate) || null,
    targetId: crateToNode.get(cm.calleeCrate) || null,
    sourcePortId: null, targetPortId: null,
    type: 'grpc-call-designed', directed: true, guardId: null,
    properties: {
      callerCrate: cm.callerCrate, calleeCrate: cm.calleeCrate, calleeService: cm.calleeService,
      fileLine: cm.fileLine, note: `仅${cm.mentionKind === 'string-literal' ? '字符串字面量' : '注释/文档'}提及, 无真实调用点`,
      mentionKind: cm.mentionKind,
    },
    verification: 'Inferred',
    evidenceIds: [ev.id],
  });
}

/* 容器层级: System → Building(env) → Floor(domain) → Room(crate) */
containers.push({ id: 'C-SYSTEM', kind: 'System', name: 'RustGameServer', parentId: null, childrenIds: [] });
const buildingId = 'C-BUILDING-k3s';
containers.push({ id: buildingId, kind: 'Building', name: 'k3s / rust-game-server', parentId: 'C-SYSTEM', childrenIds: [], properties: { evidence: deployments[0]?.manifest || null } });
for (const dom of ['Core', 'Interaction', 'Data', 'Security', 'Infrastructure', 'External', 'Operations', 'Unknown']) {
  const fid = `C-FLOOR-${dom}`;
  containers.push({ id: fid, kind: 'Floor', name: dom, parentId: buildingId, childrenIds: [] });
}
for (const c of crateMeta) {
  const dom = DOMAIN_OF[c.crate] || 'Unknown';
  const rid = `C-ROOM-${c.crate}`;
  containers.push({ id: rid, kind: 'Room', name: c.crate, parentId: `C-FLOOR-${dom}`, childrenIds: [], properties: { crate: c.crate, domain: dom } });
}
for (const c of containers) if (c.parentId) {
  const p = containers.find(x => x.id === c.parentId);
  if (p) p.childrenIds.push(c.id);
}

const architecture = {
  meta: {
    schemaVersion: '1.0.0',
    subject: 'RustGameServer (RGS)',
    repo: REMOTES.find(r => r.name === 'origin')?.url || null,
    revision: REVISION,
    shortRevision: SHORT_REV,
    branch: BRANCH,
    capturedAt: new Date().toISOString(),
    generator: 'scripts/capture-evidence.mjs',
    generatedFiles: ['models/evidence.json', 'models/architecture.json'],
    honestyNotice: '本文件由脚本从 RGS 仓库自动采集。每条记录带 fileLine + revision。verification 字段遵循 implementation_gate: 仅有接口/DDL/配置 → Designed; 仅在真实调用点看到跨服务 client 引用 → Implemented。',
  },
  counts: {
    crates: crateMeta.length,
    protoFiles: protoFiles.length,
    grpcServices: apiNodes.length,
    grpcMethods: totalRpc,
    migrations: sqlFiles.length,
    tables: totalTables,
    duplicateTablesSkipped,
    k8sWorkloads: deployments.length,
    crossServiceCallSites: callEdges.length,
    designedOnlyCallPairs: designedEdges.length,
    evidenceRecords: evidence.length,
  },
  containers,
  nodes: [...nodes, ...apiNodes, ...tableNodes],
  edges: callEdges,
  designedOnlyEdges: designedEdges,
  ports,
  sourceRoles: {
    primary: 'RGS 仓库当前工作树 (crates/ + docs/deploy/)',
    baseline: 'git history (未采集, 标注 Unknown)',
    pattern: '无 (未提供参考仓库)',
    visualReference: '无 (未提供 UI 参考)',
  },
};

const evidenceDoc = {
  meta: {
    schemaVersion: '1.0.0', subject: 'RustGameServer (RGS)',
    revision: REVISION, shortRevision: SHORT_REV, branch: BRANCH,
    capturedAt: new Date().toISOString(),
    generator: 'scripts/capture-evidence.mjs',
    evidenceKinds: ['SourceCode', 'TestTrace', 'RuntimeLog', 'Document', 'PublicObservation', 'UserRequirement'],
    honestyNotice: '每条 Evidence 含 fileLine (path:startLine-endLine) 与 revision, 可独立 git/Read 复核。',
  },
  records: evidence,
};

writeFileSync(join(OUT, 'evidence.json'), JSON.stringify(evidenceDoc, null, 2));
writeFileSync(join(OUT, 'architecture.json'), JSON.stringify(architecture, null, 2));

console.log(JSON.stringify({
  revision: SHORT_REV, branch: BRANCH,
  counts: architecture.counts,
  out: [join(OUT, 'evidence.json'), join(OUT, 'architecture.json')],
}, null, 2));
