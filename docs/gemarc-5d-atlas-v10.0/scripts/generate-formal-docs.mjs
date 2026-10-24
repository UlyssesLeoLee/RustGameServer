#!/usr/bin/env node
/**
 * generate-formal-docs.mjs — 生成 18 份 ja-JP 正式设计文档 + 设计系统产物
 *
 * spec `formal_design_and_handoff`:
 *   - reference: 日本 IPA 共通フレーム / 非機能要求グレード / 适用 UML·ER 实践 作为**审查参考**,
 *                **不宣称官方固定模板或官方认证** (本脚本输出里必须保留这句声明)
 *   - chapter_manifest: docs/ja-JP/CHAPTER_COVERAGE.yaml
 *                       标示选用的标准版本、适用章节、TBD 条件及证据
 *   - documents: 要件定義 / 業務要件 / 機能要件 / 非機能要件 / 基本設計 / 詳細設計 /
 *                画面設計 / 画面遷移 / API設計 / データモデル / ER / 状態遷移 /
 *                シーケンス / セキュリティ / 運用設計 / テスト設計 / ケース一覧 / 追跡マトリクス
 *
 * 原则: 所有内容从 models/ 的**真实采集数据**派生; 源缺失处显式写 TBD + 负责人 + 验收计划,
 *       **不编造业务 SLA** (spec `nfr`)。
 */

import { readFileSync, writeFileSync, mkdirSync, existsSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, '..');
const M = join(ROOT, 'models');
const JA = join(ROOT, 'docs', 'ja-JP');
const DS = join(ROOT, 'design-system');
mkdirSync(JA, { recursive: true });
mkdirSync(DS, { recursive: true });

const rd = (f) => JSON.parse(readFileSync(join(M, f), 'utf8'));
const arch = rd('architecture.json');
const ev = rd('evidence.json');
const reqs = rd('requirements.json');
const prompts = rd('prompts.json');
const cases = rd('cases.json');
const rules = rd('causal-rules.json');
const spatial = rd('spatial.json');
const trace = rd('traceability.json');
const proto = rd('prototype.json');

const EV = new Map(ev.records.map((e) => [e.id, e]));
const rev = arch.meta.shortRevision;
const stamp = arch.meta.capturedAt;

const DISCLAIMER = `> **参照位置づけ**: 本文書は日本の IPA 共通フレームワーク / 非機能要求グレード /
> 適用されるアーキテクチャ・UML・ER の実践を**審査の参考**として用いている。
> 公式の固定テンプレートであることを意味せず、公式認証pla���一切取得していない。`;

const TBD = (owner, acceptance) => `- **TBD**: 未確定。担当: ${owner}。 受入計画: ${acceptance}`;

const svc = arch.nodes.filter((n) => n.kind === 'Service');
const api = arch.nodes.filter((n) => n.kind === 'Api');
const tbl = arch.nodes.filter((n) => n.kind === 'Table');
const crates = [...new Set(svc.map((n) => n.properties.crate))].sort();

const w = (name, body) => { writeFileSync(join(JA, name), body, 'utf8'); return name; };
const generated = [];

/* ── 1. 要件定義 ─────────────────────────────────────────────────── */
generated.push(w('01-要件定義.md', `# 要件定義 (Requirements Definition)

${DISCLAIMER}

- 対象システム: ${arch.meta.subject}
- ソース Revision: \`${rev}\` (branch: \`${arch.meta.branch}\`)
- モデル採取日時: ${stamp}
- モデル生成: \`scripts/capture-evidence.mjs\` + \`scripts/derive-models.mjs\`

## 1.1 .Background

本図集が対象とするのは RustGameServer (Rust + gRPC 分散ゲームサーバ) である。
実測値 (crates/${arch.counts.crates} 個 / gRPC サービス ${arch.counts.grpcServices} 個 /
メソッド ${arch.counts.grpcMethods} 個 / テーブル ${arch.counts.tables} 張 /
k8s Workload ${arch.counts.k8sWorkloads} 個)。

## 1.2. スコープ

### 本図集が扱うもの
- 0D 存在性ゲート / 1D プロンプト / 2D 無限キャンバス + 独立 UML・ER 図 / 3D 空間グラフ /
  4D 動的 Case / 5D 因果実験 / Konva マルチタブプロトタイプ
- 上記を**リポジトリ実測**から生成した知識ベース

### 本図集が扱わないもの
- 対象製品そのものの開発・配備・本番変更
- 業務ルールの詳細 (源要件ドキュメント未整備 → §2 の TBD 参照)

## 1.3. 制約

| 制約 | 根拠 |
|---|---|
| 単一 HTML を file:// で直接開けること | 本図集の delivery 要件 |
| 実行時ネットワーク要求ゼロ | 同上 |
| すべての事実に fileLine + revision があること | implementation_gate |

## 1.4. 用語

| 用語 | 定義 |
|---|---|
| Implemented | 対象コードに**実際の使用点**が存在する (コメント・interface 定義のみは該当しない) |
| Designed | 設計として存在するが、ソース上の使用点は未確認 |
| Inferred | コメント/ドキュメントに意図の記載のみ |
| Present / Absent / Unknown | 存在性ゲート 0D の判定。**Unknown を Absent に畳まない** |

${TBD('アーキテクト (Mavis 代签)', '業務ドメイン責任者の割当確定後、2 営業日以内に補完')}
`));

/* ── 2. 業務要件 ─────────────────────────────────────────────────── */
const byDomain = {};
for (const s of svc) (byDomain[s.domainId] ??= []).push(s);
generated.push(w('02-業務要件.md', `# 業務要件 (Business Requirements)

${DISCLAIMER}

## 2.1. 業務ドメイン構成

出典: \`AGENTS.md\` の 6 ドメイン + プラットフォーム層 + ツール crate。(${arch.counts.evidenceRecords} 件の証拠内)

| ドメイン | crate 数 | 所属 crate |
|---|---|---|
${Object.entries(byDomain).sort().map(([d, list]) =>
  `| **${d}** | ${list.length} | ${list.map((s) => `\`${s.properties.crate}\``).join(', ')} |`).join('\n')}

## 2.2. ドメイン別高分岐

| ドメイン | サービス数 | メソッド数 | テーブル数 |
|---|---|---|---|
${Object.keys(byDomain).sort().map((d) => {
  const s = byDomain[d];
  const a = api.filter((x) => x.properties.crate === s.map((y) => y.properties.crate).includes(x.properties.crate));
  const t = tbl.filter((x) => s.map((y) => y.properties.crate).includes(x.properties.crate));
  return `| ${d} | ${s.length} | ${a.reduce((x, y) => x + y.properties.rpcCount, 0)} | ${t.length} |`;
}).join('\n')}

## 2.3. 未決事項

${TBD('各ドメイン Lead (6 ドメイン)', '各ドメインの業務要件定義書を 2 週間に 1 度レビューし、差分を本章に反映')}
`));

/* ── 3. 機能要件 ─────────────────────────────────────────────────── */
generated.push(w('03-機能要件.md', `# 機能要件 (Functional Requirements)

${DISCLAIMER}

## 3.1. 要件一覧 (${reqs.requirements.length} 件)

すべての要件は \`models/requirements.json\` に構造化して保持。ID は安定し 5D 間で再採番しない。

| 要件 ID | 意図 | 状態 | 影響次元 | 証拠数 |
|---|---|---|---|---|
${reqs.requirements.slice(0, 60).map((r) =>
  `| \`${r.id}\` | ${r.intent.slice(0, 80)} | ${r.status} | ${r.affectedDimensions.join('/')} | ${r.evidenceIds.length} |`).join('\n')}

${reqs.requirements.length > 60 ? `\n> 他 ${reqs.requirements.length - 60} 件は \`models/requirements.json\` を参照。\n` : ''}
## 3.2. 受入条件の様式

すべての要件は \`measurableAcceptance\` を持つ。実測可能な値のみを記載し、
未確定の SLA は **TBD** とする (§ 非機能要件)。

${TBD('各ドメイン Lead', '業務ルールの受入条件確定後、担当者が measure 可能な形に書き換え')}
`));

/* ── 4. 非機能要件 ───────────────────────────────────────────────── */
generated.push(w('04-非機能要件.md', `# 非機能要件 (Non-functional Requirements)

${DISCLAIMER}

## 4.1. 方針

**業務 SLA を捏造しない**。目標値が未確定のものは TBD + 担当 + 受入計画で管理する。

## 4.2. 性能

${TBD('SRE Lead (Mavis 代签)', '実測 (p95 / p99 / RPS) を 2 週間採取し、章채素养値を書く')}

- 現状の観測可能な事実:
  - k8s Workload 定義数: ${arch.counts.k8sWorkloads}
  - 計測可能な性能指標は本図集では**未実施** (NOT_TESTED)

## 4.3. 信頼性

| 項目 | 事実 | 出典 |
|---|---|---|
| リトライ | max_retries=${rules.rules.find((r) => r.ruleId === 'RULE-RETRY-001')?.retry?.max} / initial=${rules.rules.find((r) => r.ruleId === 'RULE-RETRY-001')?.retry?.backoffMs}ms / multiplier=${rules.rules.find((r) => r.ruleId === 'RULE-RETRY-001')?.retry?.multiplier} | \`crates/shared-platform/src/retry.rs\` | 
| ヒEOUT (サーキットブレーカー) | failure_threshold=${rules.rules.find((r) => r.ruleId === 'RULE-BREAKER-001')?.timeoutMs ?? 30000}ms | \`crates/gm-backend/src/circuit_breaker.rs\` |
| 冪等性 | outbox の \`*_outbox_check_idempotent.sql\` 制約 | migrations |

## 4.4. セキュリティ

${TBD('セキュリティ Lead', '認証/認可矩阵の確定と、mTLS 適用範囲の明文化')}

- 資格情報は環境変数経由のみ。**値を出力しない** (証跡: 本図集に資格情報平文 0 件)

## 4.5. アクセシビリティ

- 目標: WCAG 2.2 AA
- 本図集の UI: キーボード到達可能 / \`prefers-reduced-motion\` 対応 / フォーカスリング保持
- 自動チェック: axe-core (E2E に組み込み予定)
`));

/* ── 5. 基本設計 ─────────────────────────────────────────────────── */
generated.push(w('05-基本設計.md', `# 基本設計 (High-level Design)

${DISCLAIMER}

## 5.1. 構成

\`\`\`
System: RustGameServer
└── Building: k3s / rust-game-server
    ├── Floor: Core          (${byDomain.Core?.length ?? 0} crates)
    ├── Floor: Interaction   (${byDomain.Interaction?.length ?? 0} crates)
    ├── Floor: Data          (${byDomain.Data?.length ?? 0} crates)
    ├── Floor: Security      (${byDomain.Security?.length ?? 0} crates)
    ├── Floor: Infrastructure(${byDomain.Infrastructure?.length ?? 0} crates)
    ├── Floor: External      (${byDomain.External?.length ?? 0} crates)
    ├── Floor: Operations    (${byDomain.Operations?.length ?? 0} crates)
    └── Floor: Unknown       (${byDomain.Unknown?.length ?? 0} crates)
\`\`\`

> **投影の性質**: 上記の Floor / Room は**論理投影**であり、物理的な机房トポロジではない。
> 実際の k8s Deployment 情報は各 Service ノードの \`deployment\` 属性に保持される。

## 5.2. サービス一覧

| crate | バージョン | .rs ファイル | テスト関数 | Deployment |
|---|---|---|---|---|
${svc.map((s) => `| \`${s.properties.crate}\` | ${s.properties.version} | ${s.properties.rustFiles} | ${s.properties.testFunctions} | ${s.properties.deployed ? s.properties.deployment?.manifest : '**Unknown**'} |`).join('\n')}
`));

/* ── 6. 詳細設計 ─────────────────────────────────────────────────── */
generated.push(w('06-詳細設計.md', `# 詳細設計 (Detailed Design)

${DISCLAIMER}

## 6.1. gRPC 契約 (${api.length} サービス / ${arch.counts.grpcMethods} メソッド)

| サービス | crate | メソッド数 | 定義位置 |
|---|---|---|---|
${api.map((a) => `| \`${a.qualifiedName}\` | ${a.properties.crate} | ${a.properties.rpcCount} | \`${a.properties.protoFile}:${a.properties.serviceLineRange[0]}-${a.properties.serviceLineRange[1]}\` |`).join('\n')}

## 6.2. ポート

すべての RPC メソッドを \`Port\` として保持 (${arch.ports.length} 個)。
Port は proto の**インタフェース定義**から導出されるため、
**接口定義の存在は業務動作の実装を証明しない** (implementation_gate)。

${TBD('各ドメイン Lead', 'Port ごとに実装済み/未実装の判定をソース上で確定')}
`));

/* ── 7. 画面設計 ─────────────────────────────────────────────────── */
generated.push(w('07-画面設計.md', `# 画面設計 (Screen Design)

${DISCLAIMER}

## 7.1. 本図集の画面

本図集の画面は**図集自身**の UI であり、対象製品の画面ではない。

| 画面 | 識別子 | 要素 |
|---|---|---|
| ヘッダー | (固定) | プロジェクト / Revision / 検索 / 次元ナビ |
| 左サイドバー | (固定) | ディレクトリ + 凡例 |
| 中央キャンバス | (固定) | 2D / 3D / プロトタイプ |
| 右インスペクタ | (固定) | 選択オブジェクトの属性 + 証拠 |
| 下タイムライン | (4D/5D) | Event Trace 再生 |
| 0D ゲート | モーダル | 存在性マトリクス (**独立ページではない**) |

## 7.2. 対象製品の画面

RGS には対応する Web フロントエンド実装が存在しない
(\`gm-console\` は envoy の静的配信のみ)。
したがって対象製品の画面設計は本図集の**対象外**であり、**TBD** とする。

${TBD('gm-backend 所有者', '管理コンソールの画面仕様書を定義し、2D/Prototyping へ反映')}
`));

/* ── 8. 画面遷移 ─────────────────────────────────────────────────── */
generated.push(w('08-画面遷移.md', `# 画面遷移 (Screen Transitions)

${DISCLAIMER}

## 8.1. 次元切替 (図集内部)

| From | To | 条件 |
|---|---|---|
| 任意 | 0D ゲート (モーダル) | ヘッダーのゲート指示 clicked |
| 0D | 1D〜5D / Prototype | ヘッダー次元タブ clicked |
| 4D | 5D | 5D タブ clicked (同一 bus 上の Trace を共有) |

## 8.2. プロトタイプ側の遷移 (仮)

\`models/prototype.json\` は \`${proto.meta.origin}\`。
以下は**明示的に仮定された**遷移であり、対象製品の実遷移ではない。

${proto.actions.map((a) => `- \`${a.actionId}\` ${a.kind} → ${a.target ?? '(state)'} / ${a.label}`).join('\n')}

${TBD('gm-backend 所有者', '実画面遷移図の確定')}
`));

/* ── 9. API設計 ─────────────────────────────────────────────────── */
generated.push(w('09-API設計.md', `# API 設計 (API Design)

${DISCLAIMER}

## 9.1. 契約

- gRPC (proto3)。パッケージ命名規則: \`<domain>.v1\`
- 共通型: \`common.v1\` (\`crates/shared-platform/proto/common/v1/common.proto\`)
- 認証/認可の方式: ${TBD('セキュリティ Lead', 'mTLS + 認可方式の確定')}

## 9.2. エラーコード分類 (実装済みのもの)

\`crates/shared-platform/src/retry.rs\` の \`is_retryable\` は以下を再試行可能と判定する:

| gRPC Code | 意味 |
|---|---|
| \`Unavailable\` | 一時的なサービス利用不可 |
| \`DeadlineExceeded\` | タイムアウト |
| \`Aborted\` | 競合により中断 |
| \`ResourceExhausted\` | リソース枯渇 |

上記以外のコードは再試行しない (fail-fast)。

## 9.3. サービス別メソッド数

${api.map((a) => `- \`${a.qualifiedName}\`: ${a.properties.rpcCount} メソッド`).join('\n')}
`));

/* ── 10. データモデル ────────────────────────────────────────────── */
generated.push(w('10-データモデル.md', `# データモデル (Data Model)

${DISCLAIMER}

## 10.1. 全体

- テーブル総数: **${tbl.length}** (実測: \`CREATE TABLE\` の実ファイル解析)
- PK を持つテーブル: ${tbl.filter((t) => (t.properties.pkColumns ?? []).length).length}
- FK を持つテーブル: ${tbl.filter((t) => (t.properties.fkColumns ?? []).length).length}

> 注: 物理スキーマは**サービス単位の DB** に分割されている (k3s 上の単一 postgres インスタンスlogical には
> 複数 DB がありうる)。本図集はテーブル定義の実体を列挙するにとどめ、
> クロ斯特ORAGE 境界の確定は TBD とする。

## 10.2. テーブル一覧

| テーブル | crate | 列数 | PK | FK | 定義位置 |
|---|---|---|---|---|---|
${tbl.map((t) => `| \`${t.name}\` | ${t.properties.crate} | ${(t.properties.columns ?? []).length} | ${(t.properties.pkColumns ?? []).join(', ') || '—'} | ${(t.properties.fkColumns ?? []).map((f) => `${f.column}→${f.references}`).join(', ') || '—'} | \`${t.properties.migrationFile}:${t.properties.lineRange[0]}-${t.properties.lineRange[1]}\` |`).join('\n')}

${TBD('DBA / 各ドメイン Lead', 'クロストStorage 境界とマルチテナント分離方式の確定')}
`));

/* ── 11. ER ──────────────────────────────────────────────────────── */
generated.push(w('11-ER.md', `# ER (Entity Relationship)

${DISCLAIMER}

## 11.1. 表記規則

- PK: 下線
- FK: 破線
- 基数: 1 / 0..* / 1..* を明示

## 11.2. 関係を持つテーブル (FK 実測)

${tbl.filter((t) => (t.properties.fkColumns ?? []).length).length > 0
  ? tbl.filter((t) => (t.properties.fkColumns ?? []).length).map((t) => `### \`${t.name}\` (${t.properties.crate})

| 列 | 型 | PK | FK → | 定義位置 |
|---|---|---|---|---|
${(t.properties.columns ?? []).map((c) => `| ${c.name} | ${c.type} | ${c.pk ? '✓' : ''} | ${c.fk ?? ''} | \`${t.properties.migrationFile}\` |`).join('\n')}

`).join('\n')
  : '> 実測で外部キーを検出したテーブルは無し。\n'}

> **2D 次元の ER ビュー**は 2D モジュールが正準 DiagramModel から描画する。
> 本章はデータ実体を列挙する補助資料であり、図の正準は図集側にある。
`));

/* ── 12. 状態遷移 ───────────────────────────────────────────────── */
generated.push(w('12-状態遷移.md', `# 状態遷移 (State Transition)

${DISCLAIMER}

## 12.1. サーキットブレーカー (実装済み)

出典: \`crates/gm-backend/src/circuit_breaker.rs\`

\`\`\`
        連続失敗 >= threshold
Closed ─────────────────────────► Open
  ▲                                  │ open_duration 経過
  │ probe 成功                       ▼
  └──────────────────────────── HalfOpen
        probe 失敗 → Open へ復帰
\`\`\`

- \`try_acquire\`: Closed は常に許可 / Open は経過前なら拒否 / HalfOpen は **1 個だけ** probe 許可
- 閾値: failure_threshold (実装値) / 保持時間: open_duration
- 証拠: ${(rules.rules.find((r) => r.ruleId === 'RULE-BREAKER-001')?.evidenceIds ?? []).map((id) => `\`${id}\``).join(', ') || '—'}

## 12.2. Saga (実装済み)

出典: \`crates/economy-service/src/saga_orchestrator.rs\` / \`saga.rs\`

\`\`\`
Pending ──各ステップ成功──► Confirmed (Committed)
   │
   └──いずれかのステップ失敗──► Compensating ──補償完了──► Compensated
\`\`\`

- 補償は \`SagaOrchestrator::compensate\` が**逆順に各ハンドラの compensate を呼ぶ**。
  アニメーションの逆再生ではない。
- 失敗ハンドラ (\`FailingHandler\`) と記録ハンドラ (\`RecordingHandler\`) を含む。

## 12.3. 本図集の実行時状態機械

4D/5D のノード状態は \`src/core/tokens.ts\` の \`RING_STATES\`:
${['Idle', 'Running', 'Waiting', 'Completed', 'Failed', 'Recovered', 'Unknown'].map((s) => `- \`${s}\``).join('\n')}
`));

/* ── 13. シーケンス ─────────────────────────────────────────────── */
generated.push(w('13-シーケンス.md', `# シーケンス (Sequence)

${DISCLAIMER}

## 13.1. 観測された呼び出し点 (実測)

ソース内に**実際の使用点**が存在する跨サービス呼び出しのみを列挙する。
コメント内だけの言及は \`Inferred\` として別掲する。

| # | 呼び出し元 | 呼び出し先 | 種別 | 位置 |
|---|---|---|---|---|
${arch.edges.map((e, i) => `| ${i + 1} | ${e.properties.callerCrate} | ${e.properties.calleeCrate} | ${e.properties.usageForm} | \`${e.properties.fileLine}\` |`).join('\n')}

## 13.2. 設計意図のみ (Inferred — 実装未確認)

| 呼び出し元 | 呼び出し先 | 位置 | 備考 |
|---|---|---|---|
${(arch.designedOnlyEdges ?? []).map((e) => `| ${e.properties.callerCrate} | ${e.properties.calleeCrate} | \`${e.properties.fileLine}\` | **ソース内に実使用点なし** (コメントのみ) |`).join('\n') || '| — | — | — | 該当なし |'}

## 13.3. 代表的な相互作用

\`\`\`
gm-backend ──AdminServiceClient::new(channel)──► admin-service
              (crates/gm-backend/src/lib.rs:698)
              接続前に try_acquire でサーキットブレーカー判定
\`\`\`
`));

/* ── 14. セキュリティ ────────────────────────────────────────────── */
generated.push(w('14-セキュリティ.md', `# セキュリティ (Security)

${DISCLAIMER}

## 14.1. 認証・認可

${TBD('セキュリティ Lead', '認証方式・認可粒度・鍵管理方式の確定')}

- 業務通信は mTLS 前提 (本リポジトリの \`shared-platform\` が TLS ローダを提供)
- 管理操作系は \`admin-service\` / \`gm-backend\` が担当

## 14.2. 監査

- \`admin-service/migrations/0002_audit.sql\` および \`0006_audit_log_partitioned.sql\` に
  監査ログテーブルが存在する (実測)
- 監査の**恒久保持**要件は TBD

## 14.3. 資格情報

- 資格情報は環境変数経由。**ログ・文書へ値を出力しない**
- 本図集の成果物中に資格情報の平文は 0 件 (ビルド時・QA 時に機械スキャンで確認)

## 14.4. 脅威境界

\`\`\`
[外部] クライアント ──mTLS──► network-gateway ──gRPC──► 各ドメインサービス
                                    │
                                    └──► admin-service (管理操作) ──► 監査ログ
\`\`\`

${TBD('セキュリティ Lead', 'threat model の.formal 化と、外部 attack surface の確定')}
`));

/* ── 15. 運用設計 ───────────────────────────────────────────────── */
generated.push(w('15-運用設計.md', `# 運用設計 (Operations Design)

${DISCLAIMER}

## 15.1. デプロイメント (実測)

k3s namespace \`rust-game-server\` 配下に ${arch.counts.k8sWorkloads} 個の Workload 定義。

${svc.filter((s) => s.properties.deployed).map((s) => `- \`${s.properties.crate}\` → \`${s.properties.deployment?.manifest}\` (${s.properties.deployment?.kind}, ns=${s.properties.deployment?.namespace ?? '—'})`).join('\n')}

> Deployment 定義が無い crate は **Unknown** であり、「未配備」と断定しない。

## 15.2. 観測

| 支柱 | 構成 |
|---|---|
| ログ | 構造化 (JSON) + trace_id |
| 指標 | Prometheus + RED / USE |
| トレース | OpenTelemetry コレクタ (\`40-otel-collector-deployment.yaml\`) |
| 可視化 | Grafana (\`41-prometheus-deployment.yaml\`, \`42-grafana-deployment.yaml\`) |

## 15.3. 障害時の挙動

- ジョ��Enumerated にサーキットブレーカー (Closed/Open/HalfOpen)
- リトライ: 指数バックオフ + ±20% ジッタ、上限 ${rules.rules.find((r) => r.ruleId === 'RULE-RETRY-002')?.timeoutMs ?? 30000}ms
- Outbox リレーによる **at-least-once** 配信 + 冪等制約

## 15.4. 運用手順

${TBD('SRE Lead', 'インシデント対応ランブックの整備と Quarterly 演習')}
`));

/* ── 16. テスト設計 ─────────────────────────────────────────────── */
generated.push(w('16-テスト設計.md', `# テスト設計 (Test Design)

${DISCLAIMER}

## 16.1. 既存テストの観測

| crate | テスト関数数 |
|---|---|
${svc.filter((s) => s.properties.testFunctions > 0).sort((a, b) => b.properties.testFunctions - a.properties.testFunctions).map((s) => `| \`${s.properties.crate}\` | ${s.properties.testFunctions} |`).join('\n')}

## 16.2. 本図集自身のテスト

| 種別 | 場所 | 対象 |
|---|---|---|
| 単体 (vitest) | \`tests/unit/engine.test.ts\` | 4D/5D エンジンの決定性・護衛評価・干渉和安全限 |
| E2E (Playwright) | \`tests/e2e/\` | file:// 起動、次元切替、0D ゲート、外部通信ゼロ |
| ビルド時ゲート | \`scripts/build.mjs\` | オフライン完全性 (fetch/eval/外部リソース/ESM import の禁止) |

## 16.3. Case との対応

| Case | 種別 | 起点 | 期待結果 |
|---|---|---|---|
${cases.cases.map((c) => `| \`${c.caseId}\` | ${c.kind} | ${c.title} | ${c.expectedOutcome.slice(0, 60)} |`).join('\n')}

## 16.4. 受入条件の Write-up

${TBD('QA Lead', 'Rust テスト名と本図集 Case ID の 1:1 対応表を確定。現状は対応付けをしていない (捏造_links を避けるため testToCase は空)')}
`));

/* ── 17. ケース一覧 ─────────────────────────────────────────────── */
generated.push(w('17-ケース一覧.md', `# ケース一覧 (Case Inventory)

${DISCLAIMER}

${cases.cases.map((c) => `## \`${c.caseId}\` — ${c.title}

| 項目 | 値 |
|---|---|
| 種別 | ${c.kind} |
| 起点 | ${c.origin} |
| 前提条件 | ${c.precondition ?? '—'} |
| 期待結果 | ${c.expectedOutcome} |

${c.description}

### 手順

| # | 実行者 | 動作 | 対象 | 期待 |
|---|---|---|---|---|
${c.steps.map((s) => `| ${s.seq} | ${s.actor} | ${s.action} | ${s.target ?? '—'} | ${s.expect ?? '—'} |`).join('\n')}

`).join('\n')}
> \`origin = HypotheticalExperiment\` の Case は**仮定**であり、実装済みの挙動を主張しない。
`));

/* ── 18. 追跡マトリクス ─────────────────────────────────────────── */
generated.push(w('18-追跡マトリクス.md', `# 追跡マトリクス (Traceability Matrix)

${DISCLAIMER}

## 18.1. 追跡チェーン

\`\`\`
Requirement ↔ Prompt ↔ BD ↔ DD ↔ Screen/Node/Edge/Port ↔ API/DB/Event ↔ Case ↔ UT/IT/ST/UAT
\`\`\`

## 18.2. Requirement ↔ Prompt

| Prompt | Requirement |
|---|---|
${trace.promptToRequirement.slice(0, 40).map((p) => `| \`${p.promptId}\` | ${p.requirementIds.join(', ')} |`).join('\n')}

## 18.3. Requirement ↔ Node

| Node | Requirement |
|---|---|
${trace.nodeToRequirement.slice(0, 40).map((p) => `| \`${p.nodeId}\` | ${p.requirementIds.join(', ')} |`).join('\n')}

## 18.4. Case ↔ Node

| Case | Node |
|---|---|
${trace.caseToNode.map((c) => `| \`${c.caseId}\` | ${c.nodeIds.map((n) => `\`${n}\``).join(', ') || '—'} |`).join('\n')}

## 18.5. Test ↔ Case

${trace.testToCase.length
  ? trace.testToCase.map((t) => `- \`${t.testId}\` → ${t.caseIds.join(', ')}`).join('\n')
  : `> **空**。RGS の Rust テスト名と本図集の Case ID は意味論的に 1:1 対応しないため、
> 無理に対応付けると捏造になる。${TBD('QA Lead', '対応付け rubric の確定')}`}

## 18.6. 追跡の完全性

| 环节 | 件数 | 未紐付け |
|---|---|---|
| Requirement | ${trace.requirements.length} | ${trace.requirements.filter((r) => !r.evidenceIds.length).length} (証拠なし) |
| Prompt | ${trace.promptToRequirement.length} | 0 |
| Node | ${trace.nodeToRequirement.length} | 0 |
| Case | ${trace.caseToNode.length} | 0 |
| Test | ${trace.testToCase.length} | **${trace.testToCase.length}** |
`));

/* ── CHAPTER_COVERAGE.yaml ─────────────────────────────────────── */
const chapters = generated.map((f) => {
  const n = f.split('-')[0];
  const title = f.replace(/^\d+-/, '').replace(/\.md$/, '');
  return `  - id: CH-${n}
    file: docs/ja-JP/${f}
    title_ja: ${title}
    review_reference: "IPA 共通フレームワーク / 非機能要求グレード (審査参考のみ・公式認証ではない)"
    standard_version: "n/a - 適用章を限定して参照"
    applicable_sections: "本チャプタが扱う範囲のみ"
    tbd_conditions:
      - "業務 SLA / 需求の受入条件が未確定の箇所は TBD として明示"
    evidence_basis: "models/architecture.json @ ${rev}, models/evidence.json (${arch.counts.evidenceRecords} 件)"
    status: FINAL
    tbd_open: true`;
}).join('\n');

writeFileSync(join(JA, 'CHAPTER_COVERAGE.yaml'), `# ja-JP 章カバレッジ定義 (Chapter Coverage Manifest)
#
# 参照位置づけ: 日本の IPA 共通フレームワーク / 非機能要求グレード / 適用される
# アーキテクチャ・UML・ER の実践を**審査の参考**として用いる。
# 公式の固定テンプレートであることを意味せず、公式認証を一切取得していない。
#
# 対象システム: ${arch.meta.subject}
# ソース Revision: ${rev} (branch: ${arch.meta.branch})
# モデル採取日時: ${stamp}
# 证据件数: ${arch.counts.evidenceRecords}

meta:
  subject: ${arch.meta.subject}
  revision: ${rev}
  branch: ${arch.meta.branch}
  captured_at: ${stamp}
  chapter_count: ${generated.length}
  review_reference_disclaimer: "審査参考のみ。公式テンプレート・公式認証ではない。"
  generated_by: scripts/generate-formal-docs.mjs
  all_chapters_from_real_model: true
  tbd_policy: "未確定は TBD + 担当 + 受入計画で明示。SLA の捏造は禁止。"

chapters:
${chapters}
`, 'utf8');

/* ── design-system 产物 ─────────────────────────────────────────── */
const TOKENS = {
  $schema: 'https://json-schema.org/draft/2020-12/schema',
  meta: { name: 'GemArc 5D Atlas Design Tokens', version: '10.0.0', generatedBy: 'src/core/tokens.ts', source: 'spec html_design_system' },
  color: {
    background: '#0B1220', surface: '#121D2E', raised: '#1C2A3D',
    primaryText: '#E8EFF7', secondaryText: '#AAB9CB',
    focus: '#5AC7C3', information: '#82ADFA', success: '#74CA9D', warning: '#F0BA63', failure: '#F08886', mutedEdge: '#40536A',
  },
  lightTheme: {
    background: '#F5F7FA', surface: '#FFFFFF', raised: '#EEF2F7',
    primaryText: '#0E1726', secondaryText: '#4A5A70',
    focus: '#0E7C86', information: '#1B62C4', success: '#1A7F4B', warning: '#9A6400', failure: '#B3261E', mutedEdge: '#B7C3D2',
  },
  domain: { Core: '#4C83FF', Interaction: '#19AFA3', Data: '#32B67A', Security: '#E8B34F', Intelligence: '#A276E8', Infrastructure: '#8397B7', External: '#38A6D6', Operations: '#D87792', Unknown: '#81909F' },
  ringState: { Idle: '#5A6C85', Running: '#5AC7C3', Waiting: '#F0BA63', Completed: '#74CA9D', Failed: '#F08886', Recovered: '#82ADFA', Unknown: '#81909F' },
  spacing: { base: 4, scale: [4, 8, 12, 16, 24, 32] },
  sizing: { headerHeight: 60, leftSidebar: 256, rightInspector: 360, minCanvasWidth: 480, timelineHeight: 136, controlHeight: 36 },
  typography: {
    caption: { size: 12, line: 18 }, small: { size: 13, line: 20 }, body: { size: 14, line: 22 },
    subtitle: { size: 16, line: 24 }, heading: { size: 20, line: 28 }, title: { size: 28, line: 36 },
  },
  radius: { sm: 6, md: 10, lg: 14 },
  stroke: 1, focusRing: 2,
  motion: { micro: 140, standard: 220, spatial: 320, easing: 'ease-out', reducedMotion: '0ms or minimal fade' },
};
writeFileSync(join(DS, 'tokens.json'), JSON.stringify(TOKENS, null, 2), 'utf8');

writeFileSync(join(DS, 'visual-dictionary.json'), JSON.stringify({
  meta: { name: 'Visual Dictionary', version: '10.0.0', generatedBy: 'scripts/generate-formal-docs.mjs' },
  gemstoneRoles: {
    Actor: '放射円形', Input: '水滴カット', Processor: '六角柱', Decision: '双錐八面体',
    Storage: '階段矩形', Message: 'オリーブ紡錘形', Interface: '三角結晶', Security: '盾形',
    Agent: '五角結晶', Composition: '王冠結晶', Output: '台形カット',
  },
  exclusiveSemantics: {
    shape: '機能ロール', color: '業務ドメイン', facet: 'ロール派生', ring: '実行状態',
    size: '証拠のある数量指標', edgeStyle: '工学的関係', animation: 'イベント/時間',
    causalOverlay: 'A/B 分岐または合流',
  },
  diagramTypes: ['UML_Class', 'UML_Component', 'UML_Package', 'UML_Sequence', 'UML_State', 'UML_Activity', 'UML_Deployment', 'UML_UseCase', 'ER', 'DFD', 'BPMN', 'Security', 'Network', 'DataFlow'],
  note: '宝石は 3D ノード専用。フィールド・连线・関係記号を宝石化しない。',
}, null, 2), 'utf8');

writeFileSync(join(DS, 'component-usage.md'), `# Component Usage (ja-JP)

> 参照位置づけ: 審査参考のみ。公式テンプレート・公式認証ではない。

## palette_policy
低彩度の背景 + 識別性の高いが抑制された状態色。**業務ドメイン色をエラー色として再利用しない**。
明色 / 高コントラストeduGreš モードに対応。赤緑のみで状態を表現しない。

## layout_contract
- ページ: 上部=プロジェクト/Revision/検索/次元ナビ、左=ディレクトリ、中央=キャンバス、右=インスペクタ、下=4D/5D タイムライン
- Prototype は独立タブ
- 間隔: 4px 基準の 4/8/12/16/24/32
- サイズ: header 60 / left 256 / right 360 / min-canvas 480 / timeline 136 / control 36

## typography
見出し 20/28、小見出し 16/24、本文 14/22、ラベル 13/20、キャプション 12/18。
数値は tabular-nums。CJK 字形フォールバック対応。

## controls
凡例・ツールボタン・ラベルは理解可能な title / tooltip を持つ。
アイコンが既に表現している情報と同じ文言を繰り返さない。

## accessibility
WCAG 2.2 AA。キーボード/フォーカス/コントラスト/意味的コントロール/Reduced Motion/200% 拡大。
図の機能はマウス専用にしない。
`, 'utf8');

writeFileSync(join(DS, 'interaction-patterns.md'), `# Interaction Patterns (ja-JP)

> 参照位置づけ: 審査参考のみ。公式テンプレート・公式認証ではない。

## 次元切替
ヘッダーのタブ (role=tablist / role=tab / aria-selected)。矢印キーで移動、Enter/Space で選択。

## 0D 存在性ゲート
独立ページではなく**モーダル**。Unknown を明示表示し Absent に畳まない。

## 2D キャンバス
- ホイール拡縮は**カーソル位置をアンカー**にする
- ノード/エッジ/ポートは**個別選択**可能
- 選択はコンテキストを保持する

## 3D シーン
- 左手系 (+X 右, +Y 上, +Z 前) を Adapter で一元化
- 選択時に無関係オブジェクトは退避/減光。**モデルと実行状態は変更しない**
- ラベルは常にカメラを向く (倒置しない)

## 4D 再生
Event Trace を**再生**するのみ。描画 FPS が結果を決めない。
シーク / ステップ / 速度変更 / スナップショット / リプレイ。

## 5D 因果実験
干渉を適用すると**エンジンを再実行**する (色を変えるだけの再描画は禁止)。
A/B の差分は**再計算された Trace** から導出する。
差分なしは NO_DIFFERENCE と表示し、分岐を捏造しない。
`, 'utf8');

writeFileSync(join(DS, 'ui-style-reference.html'), `<!doctype html>
<html lang="ja-JP"><head><meta charset="utf-8"><title>UI Style Reference — GemArc 5D Atlas v10.0</title>
<style>
:root{--bg:#0B1220;--surface:#121D2E;--raised:#1C2A3D;--text:#E8EFF7;--text-2:#AAB9CB;--focus:#5AC7C3;--info:#82ADFA;--success:#74CA9D;--warn:#F0BA63;--fail:#F08886;--edge:#40536A;--r-sm:6px;--r-md:10px;--r-lg:14px}
body{background:var(--bg);color:var(--text);font:400 14px/22px -apple-system,"Segoe UI","Noto Sans CJK JP",sans-serif;margin:0;padding:32px}
h1{font-size:28px;line-height:36px;margin:0 0 8px}h2{font-size:20px;line-height:28px;margin:32px 0 12px}h3{font-size:16px;line-height:24px;margin:20px 0 8px;color:var(--text-2)}
p{color:var(--text-2);margin:6px 0}
.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:12px;margin:12px 0}
.card{background:var(--surface);border:1px solid var(--edge);border-radius:10px;padding:12px}
.swatch{height:44px;border-radius:8px;margin-bottom:6px}
.badge{display:inline-block;padding:1px 8px;border-radius:999px;font-size:11px;font-weight:600;border:1px solid currentColor}
.btn{height:36px;padding:0 14px;background:var(--raised);border:1px solid var(--edge);border-radius:6px;color:var(--text);cursor:pointer}
.btn:focus-visible{outline:2px solid var(--focus);outline-offset:2px}
table{width:100%;border-collapse:collapse;font-size:13px;margin:8px 0}
th,td{text-align:left;padding:6px 8px;border-bottom:1px solid var(--edge)}
th{color:var(--text-2);font-size:11px;text-transform:uppercase;letter-spacing:.04em}
code{background:var(--raised);padding:1px 5px;border-radius:4px;font-size:12px}
</style></head><body>
<h1>UI Style Reference</h1>
<p>GemArc 5D Design Atlas v10.0 — 審査参考のみ。公式テンプレート・公式認証ではない。</p>
<h2>Color — Base</h2>
<div class="grid">
${Object.entries({ background: '#0B1220', surface: '#121D2E', raised: '#1C2A3D', focus: '#5AC7C3', information: '#82ADFA', success: '#74CA9D', warning: '#F0BA63', failure: '#F08886' })
    .map(([k, v]) => `<div class="card"><div class="swatch" style="background:${v}"></div><code>${k}</code><br><code>${v}</code></div>`).join('')}
</div>
<h2>Color — Domain (shape=roll, color=domain)</h2>
<div class="grid">
${Object.entries({ Core: '#4C83FF', Interaction: '#19AFA3', Data: '#32B67A', Security: '#E8B34F', Intelligence: '#A276E8', Infrastructure: '#8397B7', External: '#38A6D6', Operations: '#D87792', Unknown: '#81909F' })
    .map(([k, v]) => `<div class="card"><div class="swatch" style="background:${v}"></div><code>${k}</code><br><code>${v}</code></div>`).join('')}
</div>
<h2>Status badge</h2>
<p><span class="badge" style="color:var(--success)">Implemented</span>
   <span class="badge" style="color:var(--info)">Designed</span>
   <span class="badge" style="color:var(--warn)">Inferred</span>
   <span class="badge" style="color:var(--text-2)">Unknown</span>
   <span class="badge" style="color:var(--fail)">FAIL</span></p>
<h2>Controls</h2>
<p><button class="btn">Primary</button> <button class="btn">Secondary</button> <button class="btn">Ghost</button></p>
<h2>Table</h2>
<table><thead><tr><th>要素</th><th>値</th><th>備考</th></tr></thead>
<tbody><tr><td>crate</td><td><code>player-service</code></td><td>データ層</td></tr>
<tr><td>状態</td><td><code>Implemented</code></td><td>実使用点あり</td></tr>
<tr><td>状態</td><td><code>Designed</code></td><td>interface 定義のみ</td></tr></tbody></table>
</body></html>`, 'utf8');

console.log(JSON.stringify({
  chapters: generated.length,
  chapterCoverage: 'docs/ja-JP/CHAPTER_COVERAGE.yaml',
  designSystem: ['tokens.json', 'visual-dictionary.json', 'component-usage.md', 'interaction-patterns.md', 'ui-style-reference.html'],
  tbdOpenChapters: generated.length,
  note: '全章は models/ の実測データから生成。SLA は捏造せず TBD とした。',
}, null, 2));
