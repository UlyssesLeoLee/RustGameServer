# Universal GemArc 5D Design Atlas v10.0

> RustGameServer (RGS) の **0D–5D 工事設計アトラス** + Konva マルチタブプロトタイプ・ワークスペース。
> **単一 HTML** — `file://` で直接ダブルクリックして動く。オフライン完結。ランタイム通信 0。

| 項目 | 値 |
|---|---|
| 対象 | RustGameServer (RGS) |
| ソース Revision | `b6a998174582` (branch `dev`) |
| 成果物 | `index.html` (単一ファイル, 約 1.8 MB) + `dist/index.html` |
| 一次データ | `models/*.json` — RGS リポジトリから**自動採集** (`scripts/capture-evidence.mjs`) |
| オフライン | 実行時ネットワーク要求 0 / fetch・eval・動的 import・CDN なし |
| 構成 | <!--STATS:START-->31 crates / 710 RPC / 61 表 / 33 k8s Workload / 885 証拠レコード<!--STATS:END--> |
| ガバナンス | `git_governance.allow_local_commit: false` → **本ディレクトリは未コミット** |

---

## 1. -gradients ではなく「一次データから生成」

このアトラスの図・文章・プロンプト・因果ルールは、**すべて** RGS リポジトリを走査した
スクリプトが生成した。手で書いた業務事実は 1 件も無い。

```
crates/**/proto/**/*.proto   ──┐
crates/**/migrations/*.sql    ──┼──> scripts/capture-evidence.mjs ──> models/architecture.json
crates/**/Cargo.toml          ──┤                                  models/evidence.json
docs/deploy/01-k8s-manifests  ──┘
                                        │
                                        ├──> scripts/derive-models.mjs      -> requirements/prompts/cases/causal-rules/spatial/traceability/prototype
                                        ├──> scripts/generate-meta-models.mjs -> schema/diagram-definitions/information-placement/scenario-deltas
                                        ├──> scripts/generate-formal-docs.mjs -> docs/ja-JP/ (18 章) + design-system/
                                        └──> scripts/generate-handoff.mjs    -> docs/AI_IMPLEMENTATION_HANDOFF.md (51 タスク)
```

**事実性の門 (spec `implementation_gate`)**: `Implemented` 判定には**実使用点**が要る。
Rust コメントを完全に剥離してから、型注釈 / `::new` 構築 / パス参照のみを採用する。
`/// - 封装 ...CardServiceClient` のような**コメント内の言及は実装と判定しない**
（`Inferred` に降格）。結果: 実装済み跨サービス呼び出し **20** 箇所 / 設計意図のみ **1** 対。

---

## 2. 次元構成 — 階段状 (1D→2D→3D→4D→5D)

各次元は独立した世界ではなく、**1 つのバス**と**1 つの 3D ステージ**の上に並ぶ。
詳細は [`docs/DIMENSION_CHAIN.md`](docs/DIMENSION_CHAIN.md)。

```
 1D Prompt  ──promptScope──►  2D 無限キャンバス
 2D 選択    ──selection2d──►  3D 空間ステージ
 3D 焦点    ──focus3d─────►  4D Case 実行
 4D Trace   ──trace────────►  5D A/B 因果実験
 5D 分岐    ──branch───────►  3D ステージ (overlay / split)
```

| 次元 | 内容 | 実装 |
|---|---|---|
| **0D** | 存在性ゲート (Present / Absent / **Unknown**) | 独立ページではない — ヘッダーのゲート指示器 → モーダル |
| **1D** | Prompt 生成 / 編集 / 検証 / コピー / エクスポート / 版差分 / 証拠リンク | `src/dims/d1.ts` — 選択中 Prompt のスコープを `bus` へ公開 |
| **2D** | 無限キャンバス + **14 種の専門図** (UML 8 / ER / DFD / BPMN / Security / Network / DataFlow) | `src/dims/d2.ts` + `src/diagrams/` — 1D 範囲を受信、選択を公開、**デモツアー** (`demo2d.ts`) |
| **3D** | 左手系 (+X右 +Y上 +Z前) 空間ステージ、12 View、12 ナビゲーション、宝石ロール、動的退避 | `src/core/scene-host.ts` + `src/dims/d3.ts` — **共有ステージのビュー** |
| **4D** | 決定論的離散イベントエンジン + Case 9 種 + 10 コントロール | `src/sim/engine.ts` + `src/dims/d4.ts` — **同じステージ上で** Trace を再生 |
| **5D** | 干渉 10 種 + A/B 因果分岐 + 13 要素の因果説明 | `src/sim/causal.ts` + `src/dims/d5.ts` — **同じエンジン・同じステージ**。Overlay / Split は実描画 |
| **Prototype** | 6 サブタブ / 複数 Screen / Konva 編集 / **実 DOM プレビュー** / Action DSL | `src/proto/` + `src/dims/prototype.ts` |

**4D/5D の設計原則** (spec):
- エンジンは **Three.js から独立**。アニメーションは Event Trace の**再生**であり、FPS は結果を決めない。
- 干渉は**色を変えるのではなく再実行**する。差分は**再計算された Trace** から導出。
- 差分なしは `NO_DIFFERENCE` と表示し、**分岐を捏造しない**。
- モデル内因果は「モデル内因果」であって、**経験的検証ではない**ことを結論に明記する。
- 干渉は実験コピーに対する ScenarioGraphDelta であり、**正式事実グラフは不変**。

### 2.1 2D のデモツアー (認知負荷低減)

160 ノード / 14 図種 / 6 レイヤーが一度に出るため、`▶ 演示` ボタンから
**Coach Mark** で 6 ステップの導線を提供 (対象を実 DOM で矩形状にハイライトし、
実際にノードをフォーカスしたり図を開いたりする)。Esc で即解除。

## 3. 使い方

### 開く
```
docs/gemarc-5d-atlas-v10.0/index.html  をダブルクリック
```

### 再生成 (ソース → 成果物)
```powershell
cd D:\RustGameServer\docs\gemarc-5d-atlas-v10.0

node scripts/capture-evidence.mjs --repo D:\RustGameServer --out models   # 事実の再採集
node scripts/derive-models.mjs                                            # 派生
node scripts/generate-meta-models.mjs                                     # 仕様由来のメタモデル
node scripts/generate-formal-docs.mjs                                     # ja-JP 18 章 + design-system
node scripts/generate-handoff.mjs                                         # AI 交接契約
node scripts/build.mjs                                                    # 単一 HTML 化 (オフライン門禁込み)
```

### 検証
```powershell
npx vitest run --reporter=json --outputFile=qa/test-results/vitest.json   # 単体 17 項目
npx playwright test --project=file-desktop --reporter=json                # E2E 5 項目 (file://)
node tests/verify-visual.mjs qa/visual                                    # 6 次元 × 2 ビューポート 撮影
node scripts/verify.mjs                                                   # 構造チェック 51 項目
node scripts/generate-qa-report.mjs                                       # QA_FINAL.md を再生成
```

> **QA 報告の数字はすべて機械計測値**。手で記入した数字は無い (スクリプトの `results.json` / `qa/test-results/*.json` / `qa/visual/visual-report.json` から生成)。

---

## 4. ディレクトリ

```
gemarc-5d-atlas-v10.0/
├── index.html                    ← エントリ (単一ファイル, 約 1.8 MB)
├── dist/index.html               ← 静的 HTTP 用 (同一内容)
├── dist/bundle-manifest.json     ← SHA256 + 第三方 license 一覧
├── src/
│   ├── main.ts / styles.css / index.template.html
│   ├── core/     types / graph (graphology) / tokens / app / bus
│   ├── dims/     d0 d1 d2 d3 d4 d5 prototype
│   ├── diagrams/ 14 種の専門図レンダラ + registry/layout/svg/data
│   ├── three/    adapter (左手系) / gems / edges / labels / model / views
│   ├── proto/    model / runtime / expr / cases / demo / ui
│   └── sim/      engine (決定論的イベント) / causal (A/B 因果)
├── models/        architecture / evidence / requirements / prompts / cases /
│                   causal-rules / spatial / traceability / prototype /
│                   schema / diagram-definitions / information-placement /
│                   scenario-deltas
├── design-system/ tokens.json / visual-dictionary.json / component-usage.md /
│                   interaction-patterns.md / ui-style-reference.html
├── docs/
│   ├── ja-JP/     18 章の正式設計 + CHAPTER_COVERAGE.yaml
│   └── AI_IMPLEMENTATION_HANDOFF.md   51 タスクの AI 実装交接契約
├── scripts/       capture-evidence / derive-models / generate-* / build / verify
├── tests/         unit/ (vitest) + e2e/ (playwright) + tools/ (診断・監査プローブ)
├── qa/            visual/ (12 枚) + test-results/ (JSON 構造化結果)
├── QA_FINAL.md    ← 機械生成の QA 報告
├── ADVERSARIAL_AUDIT.md  ← 対抗審査の統合報告
└── BACKLOG.md     ← 未解決ギャップ台帳
```

---

## 5. 依存とライセンス

| ライブラリ | 版 | ライセンス | 用途 |
|---|---|---|---|
| three | 0.160.1 | MIT | 3D シーン (実ジオメトリ / カメラ / レイキャスト) |
| konva | 9.3.16 | MIT | Prototype の WYSIWYG ステージ (**Polotno の有料版ではない**) |
| graphology | 0.25.4 | MIT | 図モデル基盤 (Node/Edge/隣接) |
| dagre | 0.8.5 | MIT | 2D 有向グラフの階層レイアウト |

> **採用しないもの**: `elkjs` (EPL-2.0 — spec「不默认引入」に従い不採用)、
> `mermaid` (UML/ER/BPMN の記号セマンティクスが本 spec の要求を満たさない + 単ファイル量が過大)、
> `dexie` (file:// で IndexedDB が不安定 — メモリ + JSON インポート/エクスポートで対応)、
> `polotno` (有料。Konva 開源コアのみを使用)。

---

## 6. ガバナンス

- 本アトラスは**読み取り専用の分析成果物**。対象製品の開発・配備・本番変更・Push・Merge を行わない。
- **コミットしていない** (spec `git_governance.allow_local_commit: false`)。
- 資格情報は平文 0 件 (ビルド時・QA 時に機械スキャンで確認)。
- `dist/` と `index.html` は**ビルド生成物**。`src/` と `models/` が保守対象。

### 代签
| 役割 | 本名 | 暫代签 |
|---|---|---|
| 改訂 | Ulysses | Mavis (AGENTS.md §0 + 守门 #14 v3+v4) |
| 承認 | 架構師 (一人公司 12 角色 per DEC-008) | Mavis (8/27 19:39/20:56/21:59 JST 三次强化) |

真人 (Ulysses / 架構師) 上岗後は、改訂履歴 +1 行で遡及署名を差し替える
(守门 #14 v2 拍板 D + 禁回溯叙事)。

---

## 7. 正式設計文書 (ja-JP)

`docs/ja-JP/` に 18 章。すべて `models/` の実測データから生成し、
業務 SLA など未確定箇所は **TBD + 担当 + 受入計画** と明記 (捏造しない)。

章カバレッジ定義は `docs/ja-JP/CHAPTER_COVERAGE.yaml`。
参照位置づけは「審査参考のみ。公式テンプレート・公式認証ではない」。
