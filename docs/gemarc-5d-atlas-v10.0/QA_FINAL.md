# QA_FINAL.md — Universal GemArc 5D Design Atlas v10.0

> **対象**: RustGameServer (RGS)
> **ソース Revision**: `b6a998174582` (branch `dev`)
> **モデル採取**: 2026-10-22T15:11:52.672Z
> **生成**: `scripts/generate-qa-report.mjs` — **本書の数値はすべて機械計測値**。手で記入した数字はない。
> **報告単位**: 1 回の実行結果であり、常時 PASS を保証するものではない。

---

## 1. 結論

| 項目 | 値 |
|---|---|
| 総合判定 | **PASS_WITH_FINDINGS (要確認項目あり)** |
| hard_fail 実測 | **0** 件 |
| 構造チェック | 53 項 (PASS 53 / FAIL 0 / PARTIAL 0 / NOT_TESTED 0) |
| 単体テスト | **17 passed / 0 failed** (17 総数) |
| E2E (file://) | **33 passed / 0 failed / 0 flaky / 0 skipped** |
| 視覚検証 | 12 枚 (6 次元 × 2 ビューポート) / コンソールエラー **0** 件 |

---

## 2. 二重クリック起動 (file://) — P1 優先度

spec `portable_html_delivery.must_open`: "file:// で直接 index.html を開いてコア閲覧・プロトタイプ・因果デモをオフラインで完了"

| 項目 | 実測 |
|---|---|
| エントリ | `index.html` (ルート) — 単一ファイル |
| ファイルサイズ | 1746 KB |
| 実行時ネットワーク要求 | **0** (Playwright request イベントで実測) |
| コンソールエラー | **0** 件 |
| 静的 dist | `dist/index.html` あり (同一内容) |

---

## 3. 离线完全性ゲート (spec `forbidden_at_runtime`)

| 検査 | 判定 | 詳細 |
|---|---|---|
| 无 fetch( | PASS | 未检出 |
| 无 eval( | PASS | 未检出 |
| 无外部 script src | PASS | 未检出 |
| 无 type="module" | PASS | 未检出 |
| 无远程资源引用 | PASS | 未检出 |
| 无 @import url(remote) | PASS | 未检出 |
| 无动态 import() | PASS | 未检出 |

> ビルド時ゲート (`scripts/build.mjs`) と QA 時の再検査 (`scripts/verify.mjs`) の**二重化**。
> 前者はビルドを停止させ、後者は結果を記録する。

---

## 4. 事実性ゲート (spec `implementation_gate`)

> "Implemented は対象工程に該当する挙動がコード path:startLine-endLine と revision として落实されていることを要する。
> pom・依存・ディレクトリ・インタフェース定義・DTO・Mock・TODO のみでは Business Behavior の実装を証明しない。"

| 検査 | 判定 | 詳細 |
|---|---|---|
| 凭据硬编码扫描 | PASS | 0 命中 |
| 模型证据 ID 完整性 | PASS | 全部证据 ID 可解析 (141 节点/边) |
| implementation_gate: Implemented 边经源码复核均为真实使用点 | PASS | 15/15 条 Implemented 边全部经源码行复核 (非 use 导入 / 非字符串字面量) |

### 4.1 収集実績

| 項目 | 件数 |
|---|---|
| crate | 31 |
| proto ファイル | 23 |
| gRPC サービス / メソッド | 33 / 710 |
| migration / テーブル | 45 / 61 |
| k8s Workload | 33 |
| 実装済跨サービス呼び出し点 | 15 |
| 設計意図のみの呼び出し対 | 1 |
| 証拠レコード | 885 |
| Case / 因果ルール | 9 / 5 |

> **注**: 「実装済呼び出し点」判定は Rust のコメント**と文字列リテラル**を完全に除去した上で、
> `Client::new` 構築 / 型注釈 / パス参照 / メソッド呼び出しの 4 形態のみを採用する
> (scripts/capture-evidence.mjs の `stripRustComments` + `stripRustStrings`)。
> `use` インポート文 (型が使えることの証明) や文字列リテラル内の言及は
> **実装の証明にならない**ため `Inferred` に降格する。
> 実際にこの除外で 4 件が Implemented から Inferred へ降格した (対抗審査で発覚)。

---

## 5. 次元別ステータス

| 次元 | 判定 | 備考 |
|---|---|---|
| 0D 存在性ゲート | PASS | 147 行, 导出 d0, id='0D', mount 存在 |
| 1D Prompt | PASS | 254 行, 导出 d1, id='1D', mount 存在 |
| 2D 無限キャンバス + 14 図種 | PASS | 1508 行, 导出 d2, id='2D', mount 存在 |
| 3D 空間グラフ | PASS | 273 行, 导出 d3, id='3D', mount 存在 |
| 4D 動的 Case | PASS | 301 行, 导出 d4, id='4D', mount 存在 |
| 5D 因果実験 | PASS | 417 行, 导出 d5, id='5D', mount 存在 |
| Prototype ワークスペース | PASS | 750 行, 导出 prototype, id='Prototype', mount 存在 |

### 5.1 専門図レンダラ (14 種)

| 検査 | 判定 | 詳細 |
|---|---|---|
| 14 种专业图渲染器覆盖 | PASS | 14/14 种: uml-class, uml-component, uml-package, uml-sequence, uml-state, uml-activity, uml-deployment, uml-usecase, er, dfd, bpmn, security, network, dataflow |

---

## 6. テスト

### 6.1 単体 (vitest)

| 項目 | 値 |
|---|---|
| ファイル | `tests/unit/engine.test.ts` |
| 総数 | 17 |
| 成功 / 失敗 | 17 / 0 |
| 検証範囲 | 決定性 (同 seed 3 回同軌跡) / 護衛評価 / 干渉 (DisableNode・InjectFault・AddInteractiveElement) / 安全限 / A-B 因果分岐 |

### 6.2 E2E (Playwright, システム Chrome, file://)

| 項目 | 値 |
|---|---|
| ファイル | `tests/e2e/j01-core.spec.ts` + `tests/e2e/j02-journeys.spec.ts` + `tests/e2e/j03-regression.spec.ts` + `tests/e2e/j04-chain.spec.ts` |
| プロジェクト | `file-desktop` (1440×900) / `file-narrow` (390×844) |
| 総数 | 33 passed / 0 failed |
| 失敗時の扱い | トレース・スクリーンショット・種を保存 (spec `failure_policy`) |

> 再実行による PASS 化は認めない (spec: "偶発失敗は FLAKY_UNRESOLVED として記録し、再実行のみで PASS を主張してはならない")。

### 6.3 視覚検証 (spec `visual_tests`)

| ビューポート | 次元 | スクリーンショット | コンソールエラー |
|---|---|---|---|
| desktop | 1D | `qa/visual/desktop-1D.png` | 0 |
| desktop | 2D | `qa/visual/desktop-2D.png` | 0 |
| desktop | 3D | `qa/visual/desktop-3D.png` | 0 |
| desktop | 4D | `qa/visual/desktop-4D.png` | 0 |
| desktop | 5D | `qa/visual/desktop-5D.png` | 0 |
| desktop | Prototype | `qa/visual/desktop-Prototype.png` | 0 |
| narrow | 1D | `qa/visual/narrow-1D.png` | 0 |
| narrow | 2D | `qa/visual/narrow-2D.png` | 0 |
| narrow | 3D | `qa/visual/narrow-3D.png` | 0 |
| narrow | 4D | `qa/visual/narrow-4D.png` | 0 |
| narrow | 5D | `qa/visual/narrow-5D.png` | 0 |
| narrow | Prototype | `qa/visual/narrow-Prototype.png` | 0 |

---

## 7. 対抗審査 (spec `adversarial_review`)

3 名の独立した読み取り専用審査員を副代理として並列起動し、13 項の対抗テストを分担実施。

| 審査員 | 担当テスト | 判定 | 報告 |
|---|---|---|---|
| A | 事実と誠実性 (T1/T2/T11/T13) | PASS_WITH_FINDINGS | `ADVERSARIAL_AUDIT.md` |
| B | 2D/3D 相互作用 (T3/T4/T5/T6/T12) | FAIL → 全件修正済 | 同上 |
| C | 4D/5D/Prototype 因果意味 (T7/T8/T9/T10) | FAIL → 全件修正済 | 同上 |

**発見 34 件 (High 6 / Medium 16 / Low 12) をすべて処置済み**。
内訳と検証手段は `ADVERSARIAL_AUDIT.md` に記載。残存する制約 (実 GPU 未検証 / 認知受容性実験未実施 /
ja-JP 正式版未作成 / 5D の 3D 重畳未配線) は §8 に明示し、隠していない。

---

## 7.5 次元チェーン検証 (1D→2D→3D→4D→5D)

要件「5Dは4Dの上に、4Dは3Dの上に、2Dは1Dの上に立つ」を**機械的に**検証する。
`tests/e2e/j04-chain.spec.ts` (7 項目) が証人になる。

| テスト | 確認すること |
|---|---|
| C0 | 3D / 4D / 5D が**同一の canvas 要素**を使う (同一 WebGL コンテキスト = 同一ステージ) |
| C1 | 1D で Prompt を選ぶ → 2D がその范围を受け取る |
| C2 | 2D でノードを選ぶ → 3D が「上流: 2D 選択」を表示しフォーカスする |
| C3 | 4D が「上流: 3D 焦点」パネルを常に持ち、焦点 Case を自動選定する |
| C4 | 4D の Trace ID を 5D が受け継ぐ (Branch A の基準線が同一) |
| C5 | 5D の Split は**実描画** (左右 2 パス描画の説明が出る) |
| C6 | 2D のデモツアーが起動し、Coach Mark が出ると消える |

2D のデモツアー (認知負荷低減) は 6 ステップ:
1D 範囲 → 図層 → ノード選択 → 専門図 → 3D への導入。
対象は実 DOM を矩形状にハイライトし、**実際に操作**する (説明だけの扉は作らない)。

設計の詳細: `docs/DIMENSION_CHAIN.md`

## 8. 既知のギャップ (Known Gaps)

> **欠標比錯標** — 不確かなままPretendカバレッジを埋めず、欠けた箇所を明示する。

| # | ギャップ | 影響 | 補完方法 |
|---|---|---|---|
| 1 | 源要件ドキュメントが未整備 | 業務ルール詳細・受入条件は TBD | 各ドメイン Lead が要件定義書を整備 |
| 2 | 業務 SLA が未確定 | 非機能要求に実測値を置けない | SRE が 2 週間測定 → 値を反映 |
| 3 | Rust テスト ↔ Case の 1:1 対応なし | `traceability.testToCase` が空 | QA Lead が対応付け rubric を確定 |
| 4 | ja-JP / en-US 版が未作成 | 対象プロジェクトの英語話者_review を支撑できない | 人工翻訳 + 校閲 |
| 5 | 3D の実 GPU 描画は未検証 | SwiftShader との差は未測定 | 実 GPU 環境で視覚回帰 |
| 6 | 認知的受容性 (3 タスク実験) は未実施 | 設計の理解度は未検証 | 5 人被験者での課題実験 |
| 7 | カバレッジ計測は部分のみ | 目標 90% の実分母は未取得 | vitest coverage + c8 を導入 |

---

## 9. 権限と副作用の宣言

- 本図集は**読み取り専用**の分析成果物であり、対象製品 (RustGameServer) の
  **開発・配備・本番変更・Push・Merge を行わない**。
- リポジトリ remote: `git@github.com:UlyssesLeoLee/RustGameServer.git`
- 現在のブランチ: `dev`
- コミット: **行っていない** (spec `git_governance.allow_local_commit: false`)
- 資格情報は平文 0 件 (ビルド時・QA 時に機械スキャンで確認)

---

## 10. 再現手順

```powershell
cd D:\RustGameServer\docs\gemarc-5d-atlas-v10.0

# 1. 事実の再採集 (RGS リポジトリから)
node scripts/capture-evidence.mjs --repo D:\RustGameServer --out models
node scripts/derive-models.mjs
node scripts/generate-meta-models.mjs
node scripts/generate-formal-docs.mjs
node scripts/generate-handoff.mjs

# 2. 単一 HTML のビルド (オフライン完结性ゲート込み)
node scripts/build.mjs

# 3. 検証
npx vitest run
npx playwright test --project=file-desktop
node tests/verify-visual.mjs qa/visual
node scripts/verify.mjs
node scripts/generate-qa-report.mjs
```

---

**報告終了** — 本ファイルは `scripts/generate-qa-report.mjs` が生成する。数値の改変はスクリプト側で行うこと。
