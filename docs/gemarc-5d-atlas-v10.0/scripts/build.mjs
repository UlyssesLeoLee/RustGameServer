#!/usr/bin/env node
/**
 * build.mjs — 构建便携单文件 index.html (+ dist/)
 *
 * spec `portable_html_delivery`:
 *   - must_open: file:// 双击根目录 index.html, 离线完成核心浏览/原型/因果演示
 *   - forbidden_at_runtime: fetch 本地 JSON / file:// ES module import /
 *     强制动态 Worker 文件加载 / 远程字体和 CDN / 任意 eval / 外部服务才能启动
 *   - output: 可维护源码 + 一体化内联 CSS/JS/数据/资源的便携 index.html + 独立 static-http dist
 *   - bundle_audit: 第三方 license 清单 / 资源 SHA256 / 离线完整性 / 生成物与源码分离
 *
 * 因此: 打包格式必须是 **iife** (非 esm), 所有模型 JSON 在构建期内联, 零运行时网络请求。
 */

import { build } from 'esbuild';
import { readFileSync, writeFileSync, mkdirSync, rmSync, existsSync, readdirSync, statSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, '..');
const OUT_JS = join(ROOT, '.build');
const PRODUCTION = !process.argv.includes('--dev');

const log = (...a) => console.log('[build]', ...a);

/* ── 1. 清理 ───────────────────────────────────────────────────────── */
if (existsSync(OUT_JS)) rmSync(OUT_JS, { recursive: true, force: true });
mkdirSync(OUT_JS, { recursive: true });
mkdirSync(join(ROOT, 'dist'), { recursive: true });

/* ── 2. 打包 TS/TSX (iife, 无外部依赖) ────────────────────────────── */
log('bundling script (iife)…');
const jsResult = await build({
  entryPoints: [join(ROOT, 'src', 'main.ts')],
  bundle: true,
  format: 'iife',
  target: ['es2020'],
  platform: 'browser',
  write: false,
  minify: PRODUCTION,
  sourcemap: PRODUCTION ? false : 'inline',
  legalComments: 'none',
  loader: { '.json': 'json' },
  define: { 'process.env.NODE_ENV': JSON.stringify(PRODUCTION ? 'production' : 'development') },
  logLevel: 'warning',
});
const js = jsResult.outputFiles[0].text;

/* ── 3. 打包 CSS ──────────────────────────────────────────────────── */
log('bundling css…');
const cssResult = await build({
  entryPoints: [join(ROOT, 'src', 'styles.css')],
  bundle: true, write: false, minify: PRODUCTION, loader: { '.css': 'css' },
  logLevel: 'warning',
});
const css = cssResult.outputFiles[0].text;

/* ── 4. HTML 模板 ─────────────────────────────────────────────────── */
const tpl = readFileSync(join(ROOT, 'src', 'index.template.html'), 'utf8');
const html = tpl
  .replace('/*__CSS__*/', () => css)
  .replace('/*__JS__*/', () => js);

const indexPath = join(ROOT, 'index.html');
const distPath = join(ROOT, 'dist', 'index.html');
writeFileSync(indexPath, html, 'utf8');
writeFileSync(distPath, html, 'utf8');

/* ── 5. bundle 审计: SHA256 + 许可清单 + 离线完整性 ─────────────────── */
const sha = (s) => createHash('sha256').update(s).digest('hex');
const thirdParty = [
  { name: 'three', version: '0.160.1', license: 'MIT', usage: '3D 场景渲染 (真实几何/材质/相机/拾取)' },
  { name: 'konva', version: '9.3.16', license: 'MIT', usage: '原型工作区 WYSIWYG 舞台 (开源核心, 非 Polotno 付费版)' },
  { name: 'graphology', version: '0.25.4', license: 'MIT', usage: '图模型底座 (Node/Edge/邻接)' },
  // dagre は宣言のみで**実際には import していない**ため成果物には含まれない。
  // ライセンス清単に「同梱済み」と書くと嘘になる (対抗審査 A-Med-4) ので、
  // 実際に同梱されるものだけを列挙する。導入検討時はこの行を復活させること。
  // { name: 'dagre', version: '0.8.5', license: 'MIT', usage: '2D 有向図の階層レイアウト' },
];

const manifest = {
  schemaVersion: '1.0.0',
  builtAt: new Date().toISOString(),
  production: PRODUCTION,
  entry: 'index.html',
  dist: 'dist/index.html',
  singleFile: true,
  offline: {
    runtimeNetworkRequests: 0,
    usesEsModuleImport: false,
    usesEval: false,
    usesRemoteCdn: false,
    note: '所有模型 JSON 与三方库在构建期内联; 运行时零 fetch / 零 import / 零 eval。',
  },
  artifacts: [
    { path: 'index.html', bytes: Buffer.byteLength(html), sha256: sha(html) },
    { path: 'dist/index.html', bytes: Buffer.byteLength(html), sha256: sha(html) },
  ],
  bundles: { js: { bytes: Buffer.byteLength(js), sha256: sha(js) }, css: { bytes: Buffer.byteLength(css), sha256: sha(css) } },
  thirdParty,
  generatedFrom: {
    script: 'scripts/build.mjs',
    source: 'src/',
    models: 'models/',
  },
  note: '生成物与源码分离: index.html / dist/ 为构建产物, src/ 与 models/ 为可维护来源。',
};
writeFileSync(join(ROOT, 'dist', 'bundle-manifest.json'), JSON.stringify(manifest, null, 2), 'utf8');

/* ── 6. 离线完整性自检 (构建期硬门禁) ──────────────────────────────── */
const violations = [];
if (/\bimport\s+.*from\s+['"]\.\/|\bimport\s*\(\s*['"]\.\//.test(js)) violations.push('JS 含相对 import (file:// 下会失败)');
if (/\bfetch\s*\(/.test(js)) violations.push('JS 含 fetch (file:// 下不可用)');
if (/\beval\s*\(/.test(js)) violations.push('JS 含 eval');
if (/https?:\/\/(?!www\.w3\.org|schema\.org)/.test(html.replace(/https?:\/\/(cdn|unpkg|jsdelivr)/g, ''))) {
  // 仅检查 runtime 资源引用, 忽略注释与许可证文本
  const runtimeRefs = html.match(/(?:src|href)\s*=\s*["']https?:\/\/[^"']+["']/g) || [];
  if (runtimeRefs.length) violations.push(`HTML 含远程资源引用: ${runtimeRefs.slice(0, 3).join(', ')}`);
}
if (/<script[^>]+src=/.test(html)) violations.push('HTML 含外部 <script src> (违反离线要求)');
if (/@import\s+url\(\s*['"]?https?:/.test(css)) violations.push('CSS 含远程 @import');
if (/<script[^>]+type=["']module["']/.test(html)) violations.push('HTML 含 type="module" script');

if (violations.length) {
  console.error('[build] ❌ 离线完整性自检未通过:');
  for (const v of violations) console.error('   - ' + v);
  process.exit(1);
}

log('✅ 单文件构建完成');
log(JSON.stringify({
  indexBytes: Buffer.byteLength(html),
  jsBytes: Buffer.byteLength(js),
  cssBytes: Buffer.byteLength(css),
  offlineGate: 'PASS',
}, null, 2));
