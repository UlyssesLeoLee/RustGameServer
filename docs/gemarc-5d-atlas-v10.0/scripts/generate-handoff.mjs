#!/usr/bin/env node
/**
 * generate-handoff.mjs — 生成 AI 実装交接契约 (docs/AI_IMPLEMENTATION_HANDOFF.md)
 *
 * spec `formal_design_and_handoff.ai_handoff`:
 *   "为每个实现任务输出 objective、requirementIds、sourceEvidence、targetModules、
 *    inputOutput、rules、failureModes、acceptanceTests、dependency 和 rollback/ops。"
 *
 * 逐条从 models/ 派生, 不手工编造。源缺失处写明 TBD。
 */

import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, '..');
const M = join(ROOT, 'models');
const rd = (f) => JSON.parse(readFileSync(join(M, f), 'utf8'));

const arch = rd('architecture.json');
const ev = rd('evidence.json');
const reqs = rd('requirements.json');
const prompts = rd('prompts.json');
const cases = rd('cases.json');
const rules = rd('causal-rules.json');
const EV = new Map(ev.records.map((e) => [e.id, e]));

const svc = arch.nodes.filter((n) => n.kind === 'Service');
const api = arch.nodes.filter((n) => n.kind === 'Api');
const tbl = arch.nodes.filter((n) => n.kind === 'Table');
const promptByReq = new Map();
for (const p of prompts.prompts) for (const r of p.requirementIds) promptByReq.set(r, p);

const L = [];
L.push(`# AI 実装交接契約 (AI Implementation Handoff Contract)

> **対象**: ${arch.meta.subject}
> **ソース Revision**: \`${arch.meta.shortRevision}\` (branch \`${arch.meta.branch}\`)
> **モデル採取**: ${arch.meta.capturedAt}
> **生成**: \`scripts/generate-handoff.mjs\` (モデルから自動派生)
> **注意**: 各タスクの sourceEvidence はすべて \`file:startLine-endLine\` + revision を持つ。
> ここに無い事実は本リポジトリから**確認できなかった**という 뜻であり、捏造して補完しないこと。

## 0. 読み方

各タスクは 10 フィールド固定:
\`objective\` / \`requirementIds\` / \`sourceEvidence\` / \`targetModules\` / \`inputOutput\` /
\`rules\` / \`failureModes\` / \`acceptanceTests\` / \`dependency\` / \`rollback-ops\`

`);

let t = 0;
for (const r of reqs.requirements) {
  t += 1;
  const crate = r._crate;
  const callee = r._callee;
  const s = svc.find((x) => x.properties.crate === crate);
  const apis = api.filter((a) => a.properties.crate === crate);
  const tables = tbl.filter((x) => x.properties.crate === crate);
  const p = promptByReq.get(r.id);
  const relatedCases = cases.cases.filter((c) => c.nodeIds.includes(s?.id)).slice(0, 3);
  const relatedRules = rules.rules.filter((ru) => s && ru.scopeNodeIds.includes(s.id));

  const evidence = r.evidenceIds
    .map((id) => EV.get(id))
    .filter(Boolean)
    .slice(0, 6)
    .map((e) => `- \`${e.fileLine}\` (rev \`${e.revision}\`, ${e.kind}) — ${e.note}`)
    .join('\n') || '- **TBD**: 証拠なし';

  L.push(`---
## TASK-${String(t).padStart(3, '0')}${callee ? ` — ${crate} → ${callee} 呼び出し` : ` — ${crate}`}

### objective
${r.intent}

### requirementIds
${r.requirementIds?.length ? r.requirementIds.join(', ') : r.id}
${p ? `\n関連 Prompt: \`${p.promptId}\` (v${p.version})` : ''}

### sourceEvidence
${evidence}

### targetModules
${s ? `- \`crates/${crate}/\` (crate \`${s.qualifiedName}\` v${s.properties.version}, .rs ${s.properties.rustFiles} 件, テスト関数 ${s.properties.testFunctions} 件)` : '- **TBD**: 対象 crate 未特定'}
${apis.length ? apis.map((a) => `  - \`${a.qualifiedName}\` — \`${a.properties.protoFile}:${a.properties.serviceLineRange[0]}-${a.properties.serviceLineRange[1]}\` (${a.properties.rpcCount} メソッド)`).join('\n') : ''}
${tables.length ? tables.map((x) => `  - テーブル \`${x.name}\` — \`${x.properties.migrationFile}:${x.properties.lineRange[0]}-${x.properties.lineRange[1]}\``).join('\n') : ''}
${s?.properties.deployed ? `- 配備: \`${s.properties.deployment?.manifest}\` (${s.properties.deployment?.kind})` : '- 配備定義: **Unknown** (k8s manifest に該当なし。Undeployed とは断定しない)'}

### inputOutput
- **入力**: ${p?.inputs?.join(' / ') ?? '**TBD** — 入力契約が未定義 (業務要件ドキュメント未整備)'}
- **出力**: ${p?.outputs?.join(' / ') ?? '**TBD** — 出力契約が未定義'}

### rules
${relatedRules.length
    ? relatedRules.map((ru) => `- \`${ru.ruleId}\` ${ru.name}: \`when ${ru.when}\` → \`then ${ru.then}\`${ru.timeoutMs ? ` (timeout ${ru.timeoutMs}ms)` : ''}${ru.retry ? ` (retry max=${ru.retry.max}, backoff=${ru.retry.backoffMs}ms)` : ''}`).join('\n')
    : p?.constraints?.map((c) => `- ${c}`).join('\n') ?? '- **TBD**: 業務ルール未定義'}

### failureModes
${relatedCases.length
    ? relatedCases.map((c) => `- **${c.kind}** (${c.caseId}, ${c.origin}): ${c.expectedOutcome} — ${c.description.slice(0, 120)}`).join('\n')
    : '- **TBD**: 障害モード未定義'}

### acceptanceTests
${r.measurableAcceptance.map((a) => `- [ ] ${a}`).join('\n')}
${relatedCases.length ? relatedCases.map((c) => `- [ ] Case \`${c.caseId}\` (${c.kind}) を再現可能にすること`).join('\n') : ''}
${r.nonGoals.map((ng) => `- 非目標: ${ng}`).join('\n')}

### dependency
${s ? `- 上流: \`crates/${crate}/Cargo.toml\` の依存 (src 実測)` : '- **TBD**'}
${callee ? `- 呼び出し先: \`crates/${callee}/\` (実使用点あり — 依存が実在する証明)` : '- 呼び出し先: なし / **TBD**'}

### rollback-ops
- ロールバック: 該当コード差分を revert (ビルド/デプロイは本図集のスコープ外)
- 運用: ${s?.properties.deployed ? `k8s \`${s.properties.deployment?.manifest}\` の Deployment をロールバック` : '配備定義が **Unknown** のため運用手順は **TBD**'}
- 監視: 該当 crate の \`shared-platform/outbox_metrics_reporter.rs\` 系メトリクス (存在すれば)
- 資格情報は環境変数経由。**値を出力しない**

`);
}

L.push(`---
## 付録: 全体統計

| 項目 | 件数 |
|---|---|
| 実装タスク | ${t} |
| 要件 | ${reqs.requirements.length} |
| プロンプト | ${prompts.prompts.length} |
| Case | ${cases.cases.length} |
| 因果ルール | ${rules.rules.length} |
| 証拠レコード | ${arch.counts.evidenceRecords} |
| サービス (crate) | ${svc.length} |
| gRPC サービス | ${api.length} |
| テーブル | ${tbl.length} |

## 付��: 実装時に守るべき共通規則

1. **証拠なしの事実を作らない。** 対象 crate に存在しない API / テーブル / 設定名を書かない。
2. **資格情報を出力しない。** 環境変数参照の形でのみ扱う。
3. **PK を持つこと。** 既存テーブルへの追加 стол は FK を明示し、\`REFERENCES\` を書く。
4. **可逆性。** 破壊的変更 (DROP / 削除) は、可観測性翘い Anterior を残してから行う。
5. **追跡可能性。** 実装は \`REQ-xxx\` / \`CASE-xxx\` / \`file:line\` を引数として持ち、
   成果物の PR 記述と本表を相互参照できるようにする。
`);

mkdirSync(join(ROOT, 'docs'), { recursive: true });
writeFileSync(join(ROOT, 'docs', 'AI_IMPLEMENTATION_HANDOFF.md'), L.join('\n'), 'utf8');
console.log(JSON.stringify({ tasks: t, requirements: reqs.requirements.length, out: 'docs/AI_IMPLEMENTATION_HANDOFF.md' }, null, 2));
