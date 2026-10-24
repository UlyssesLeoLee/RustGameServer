#!/usr/bin/env node
// 修補 ADVERSARIAL_AUDIT.md 的章節編號缺口:
// §1.5.3 之后直接跟 H1–H6 表格, 缺少 "## 2. High 6 件" 標題, 導致編號 §1.5 → §3 跳號.
//
// 策略: 錨定 "| **H1** |" 行, 向上找最近的表頭行 "| # | 出所 | 問題 | 根本原因 | 処置 | 驗證 |",
// 在其前插入標題行. 冪等: 已存在標題則不動.
//
// 用法: node tests/tools/fix-audit-heading.mjs [--check]
import fs from 'node:fs';
import path from 'node:path';

const ROOT = path.resolve(process.cwd());
const TARGET = path.join(ROOT, 'ADVERSARIAL_AUDIT.md');
const HEADING = '## 2. High 6 件';

if (!fs.existsSync(TARGET)) {
  console.error(`[fix-audit-heading] not found: ${TARGET}`);
  process.exit(1);
}

const raw = fs.readFileSync(TARGET, 'utf8');
const lines = raw.split(/\r?\n/);

const h1Idx = lines.findIndex((l) => l.startsWith('| **H1** |'));
if (h1Idx < 0) {
  console.error('[fix-audit-heading] anchor "| **H1** |" not found — file shape changed');
  process.exit(1);
}

// 已有標題 → 冪等退出
if (lines.some((l) => l.trim() === HEADING)) {
  console.log('[fix-audit-heading] heading already present — no change (idempotent)');
  process.exit(0);
}

// 从 H1 行向上找表頭行
let headIdx = -1;
for (let i = h1Idx; i >= 0; i--) {
  if (lines[i].startsWith('| # | 出所 |')) {
    headIdx = i;
    break;
  }
}
if (headIdx < 0) {
  console.error('[fix-audit-heading] table header row not found');
  process.exit(1);
}

// 標題前留一個空行
const insertAt = headIdx;
lines.splice(insertAt, 0, HEADING, '');

const out = lines.join('\n');
if (process.argv.includes('--check')) {
  console.log('[fix-audit-heading] --check: would insert heading');
  process.exit(0);
}

fs.writeFileSync(TARGET, out, 'utf8');
console.log(`[fix-audit-heading] inserted "${HEADING}" at line ${insertAt + 1}`);
