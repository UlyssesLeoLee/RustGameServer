# ADVERSARIAL_AUDIT.md — Universal GemArc 5D Design Atlas v10.0

> **方式**: 3 名の**独立した読み取り専用**審査員を副代理として並列起動 (spec `adversarial_review`)。
> 各自が **13 項の対抗テスト** を分担し、ブラウザ実操作とソース監査で証拠を取った。
> 目標は「よく出来ていることの確認」ではなく、**嘘を探す**こと。
>
> | 審査員 | 担当 | 判定 |
> |---|---|---|
> | A | 事実と誠実性 (T1 参考リポジトリ汚染 / T2 構成のみ Implemented 誤り / T11 file:// 隠れた依存 / T13 未実行テストを PASS 表記) | **PASS_WITH_FINDINGS** |
> | B | 2D/3D 相互作用 (T3 リスナ累積 / T4 図種別誤用 / T5 左手系ずれ / T6 WebGL 偽装 / T12 狭幅検査器消失) | **FAIL** → 全件修正済 |
> | C | 4D/5D/Prototype 因果意味 (T7 偽分歧 / T8 節点のみ置物 / T9 旧 Edge 未改写 / T10 Konva Action 欠落) | **FAIL** → 全件修正済 |

---

## 1. 総括

| 区分 | 件数 | すべて处置済 |
|---|---|---|
| **High** | 6 | ✅ |
| **Medium** | 16 | ✅ |
| **Low** | 12 | ✅ (一部は backlog へ) |

**最終判定**: PASS (high 未残)。
ただし §5 のとおり、**残存する制約** (実 GPU 未検証 / 認知受容性実験未実施 / ja-JP 正式版未作成) は
NOT_TESTED として明示しており、PASS は「検証済みの範囲における PASS」である。

---

## 1.5 二次審査 — 次元チェーン (1D→2D→3D→4D→5D)

ユーザー要求「5D建立在4D基础上, 4D建立在3D基础上, 2D建立在1D基础上」を
受入条件に加えた上で、**別審査員 1 名**に**読み取り専用**で再審査させた。

### 1.5.1 審査で見つかった問題と処置

| # | 重大度 | 問題 | 処置 |
|---|---|---|---|
| CH-1 | High | 4D/5D が**各自の SVG 投影**を持ち、3D と別世界だった (「4D は 3D の上にある」が成立しない) | 3D シーンを `src/core/scene-host.ts` に**プロセス内 1 個**の共有ステージとして抽出。canvas 要素は保持し DOM 上の親だけ差し替え |
| CH-2 | High | 5D の `Overlay3D` / `SplitSynchronized3D` が**受動的な badge** のまま (3D が bus を購読していない) | ステージに `setCausalPresentation()` を実装。split は scissor テストで**左=ベースライン / 右=分岐**の 2 パス実描画 |
| CH-3 | High | 2D が 1D の Prompt と**無関係** (並列だった) | `bus.setPrompt()` で Prompt の `nodeEdgeIds` を公開。2D は範囲パネル + 絞り込みフィルタ responds |
| CH-4 | Medium | 2D → 3D の選択が渡っていなかった | `bus.setSelection2d()` を追加。3D は購読してカメラ寄せ + 隣接集合の強調 + 退避 |
| CH-5 | Medium | 3D → 4D の対象 Case 連動がなかった | `bus.setFocus3d()` を追加。4D は対象 Case を自動選定して再実行 |
| CH-6 | Medium | 4D → 5D で 5D の基準線が 4D と**別物**だった可能性 | 4D が `bus.publishTrace()`。5D は**それをそのまま Branch A** として使う (同一 Case・同一 seed・同一エンジン) |
| CH-7 | Medium | 2D に**認知負荷を下げる導線**がなかった | `src/dims/demo2d.ts` に 6 ステップの Coach Mark ツアーを実装。対象は実 DOM を矩形状にハイライトし、**実際に操作**する (図層を閉じる / ノードをフォーカス / 図を開く) |
| CH-8 | Low | 3D 書き直し時に軸ギズモを落とした | `SceneHost.toggleGizmo()` として復帰。E2E J04 が回帰検知 |

### 1.5.2 実装中に再度出たバグ (同種)

軸ギズモの落とし込みのあと、**innerHTML を書いてから querySelector する**型のバグが
3 回 (BUG-18 / BUG-19 / BUG-20) 出た。共通パッチで一括修正し、
E2E `j04-chain.spec.ts` の 7 項目を回帰検知として固定した。

### 1.5.3 判定

**PASS** — 7 項目の鎖テストすべて機械的に合格。
ただし §5 の残存制約 (4D の実行が明示ステップ列駆動でありグラフ自動走査ではない 等) は
「未検証」ではなく「仕様未確定」として明示している。

---


## 2. High 6 件

| # | 出所 | 問題 | 根本原因 | 処置 | 検証 |
|---|---|---|---|---|---|
| **H1** | A | SQL 解析がコメントを除去せず、**幻影テーブル 24 個 (名前 "IF")** を生成。表数 96 は虚偽 | `capture-evidence.mjs` の SQL パスに `stripRustComments` が無い。`--` コメント内の `CREATE TABLE IF NOT EXISTS` を拾っていた | `stripSqlComments()` を追加 (行コメント / ブロックコメント / 文字列リテラル除去)。防御的に `IF/NOT/EXISTS` 名称も拒否 | `tests/tools/audit-tables.mjs`: 幻影 **0** / 重複 **0** / 表 **96→61** / PK 54/54 一致 |
| **H2** | A | `README.md` / `QA_FINAL.md` の数値が**最新モデルと不一致** (20 辺 / 925 証拠 vs 実際 15 / 885) | 手書き数値を生成時に更新していなかった | 数値は生成スクリプト (`generate-qa-report.mjs` / `generate-formal-docs.mjs`) から取得。README もモデルから派生 | `scripts/verify.mjs` の BUND-SHA / 各種カウントと突合 |
| **H3** | B | **3D ラベル層が 0 個描画**。`LabelLayer.ensure()` が private で呼び出し口なし → `update()` が常に空 Map を走査。`3D.adaptive_labels` 項目全体が欠落。Smart/All/RelatedOnly 選択器は死んだ控件 | worker 実装の API 穴 | `LabelLayer.setTargets()` を公開 API として追加。d3.ts が全ターゲットを登録。優先度 Selected>DirectlyRelated>Context>Unrelated を実装 | `tests/tools/probe-3d-labels.mjs`: 登録 141 / All 可視 33 / Smart 可視 10 (修正前 0) |
| **H4** | B | **狭幅で検査器/目录ドロワーが開けない**。唯一のトグルが**ドロワー自身の中** (画面外) にある | `src/index.template.html` の `btn-collapse-right` が `#app-inspector` 内 | ヘッダーに常時表示のハンドル (`btn-toggle-left/right`) を追加。`aria-expanded` 同期。狭幅の 3 列上書きも `@media (min-width:1024px)` に隔離 | `tests/tools/probe-responsive.mjs`: 390/768×6 次元で横溢れ **0px**、検査器利用可能 |
| **H5** | C | **Prototype プレビューが完全にクラッシュ**。`props.tabs` / `props.items` はオブジェクト配列だが文字列と仮定して `appendChild` → TypeError → パネル 0 子 | `widgetDom` が `props` の型を仮定していた | `labelOf()` / `textOf()` ヘルパで文字列・オブジェクト両対応。`Loading/Empty/Error` の `props.message` も対応。描画例外は要素単位で捕捉 | `tests/tools/probe-causal-prototype.mjs`: 子 **33** / 実 DOM 入力 **5** / tabs 2 / nav 3 / **エラー 0** |
| **H6** | C | **旧 Edge が本当に削除されない**。`edges.delete(edge.id)` の直後に同一端点の A→B 直連辺を再作成 → 改写が topology として無効 | `engine.ts` の rewired 分岐が挿入要素と通常付け替えで分岐していなかった | `X-` 前缀時は `-in`/`-out` の 2 本**だけ**を作る。直連辺を作らない | GraphDelta パネルが `gm-backend → X-INT-1 → admin-service (旧 E-001)` を表示。旧辺 E-001 は移除 |

---

## 3. Medium 16 件

| # | 出所 | 問題 | 処置 |
|---|---|---|---|
| M1 | A | `use` インポート文と文字列リテラルを「実使用点」と誤判定 (20→15 に訂正) | `stripRustStrings()` を追加。`use` 文・文字列内出現は除外。`verify.mjs` の GATE-IMPL を**ソース行を再読する**検査に変更 (note 文字列の照合ではなく) |
| M2 | A | `verify.mjs` の `GIT-STATE` が `dirty.length===0?'PASS':'PASS'` という**恒真**だった | 実際の git 状態 (コミット有無 / push 有無) を読む検査に。ディレクトリ外の変更は**観察事項**として扱い FAIL 扱いしない (他 workstream の並列作業) |
| M3 | A | `BUND-LIC` が `thirdParty.length ? 'PASS':'FAIL'` で**恒真**。dagre を同梱済みと宣言，实は未 import | 各ライブラリを**産物の中で実測**して照合する検査に。dagre は manifest から除外 (宣言だけだった) |
| M4 | A | `ADVERSARIAL_AUDIT.md` が 17 バイトの stub なのに PASS | 報告類は「存在する」だけでなく「中身の有無」で判定。stub は NOT_TESTED |
| M5 | A | tests ディレクトリは「存在」だけ確認、0 テストでも PASS | 実テスト数を数えて照合する検査を追加 |
| M6 | A | `tests/migrations` と `migrations` の重複で qualifiedName が 14 件重複 | 本番用 `migrations/` を優先し重複を除外 (`duplicateTablesSkipped`) |
| M7 | A | `verify.mjs` の revision がハードコード | git `rev-parse HEAD` から取得 |
| M8 | A | `npm run serve` が `scripts/serve.mjs` を参照するが未作成 | `scripts/serve.mjs` を作成 (パス・トラバーサル対策付き) |
| M9 | A | `capture-evidence.mjs` のヘッダが 3 出力と記載、実際は 2 | ヘッダを実態に修正 |
| M10 | A | 使い捨て/デバッグ用スクリプトが成果物に含まれる | `tests/tools/` へ整理し、役割を明記 |
| M11 | B | 2D が**外殻常駐要素**(#canvas-toolbar / #sidebar-body) に委譲リスナを登録し、cleanup で解除せず累積 (13 回 mount → 1 クリックで 13 回描画) | `AbortController` で一括解除する `onShell()` ヘルパを導入し cleanup で `abort()`。**修正後は 13 回往復 → 1 クリックで 1 回だけ発火**を実測 |
| M12 | B | ER 図の既定表示が**常に空** (最早的 FK 表がモデル 33 位、scope 上限 30) | H1 の SQL 修正で**自動的に解消** (61  tabel 中 25 位 → scope 内)。FK 10 表を保持 |
| M13 | B | 軸ギズモが Adapter を経由せず「+Z」が意味座標 −Z を指す | `adapter.toThree()` を通すように |
| M14 | C | 4/5 件の因果ルールの護衛が**評価不能** → 実差分が UNKNOWN に降格 | `evaluateGuard` に `&&` と数値変数の比較を追加。`breaker_state` を第一級変数に。ルールの `when` を白名单形式に書換え。`RULE-RETRY-001` のスコープを全ノードへ (共通ヘルパのため) |
| M15 | C | Konva → DOM 位置同期が未実装 (`data-konva` チャネルがタブ切替で失われる) | `data-konva` を廃止。**モデル `c.x/c.y/c.w/c.h` から直接**プレビューへ絶対配置 |
| M16 | C | `RewireGraph` が `edgeId` のみで**削除のみ・新建なし**、診断も無い | 新接続先セレクトを追加し `from`/`to` を渡す。`rewired` 空なら診断を出す |

---

## 4. Low 12 件

| # | 出所 | 問題 | 処置 |
|---|---|---|---|
| L1 | A | `QA_FINAL.md` §6.2 が spec ファイル 1 つしか列挙しない | ディレクトリ走査で自動列挙 |
| L2 | A | `qa/` と `test-results/` の 2 系統の出力規約が併存 | 視覚成果物は `qa/visual/` に統一 (Playwright の `test-results/` は自動清掃対象) |
| L3 | B | ドロワーの初期値 `null` で 2 クリック必要 | `open`/`collapsed` の明確なトグルに変更 |
| L4 | B | 図注の `symbols` が静的宣言で、空図でも「基数 / PK 下線 / FK 破線」を宣言 | 実描画内容の反映へ (本バッチでは 14 種レンダラが独立に記号を出す状態を維持) |
| L5 | B | 図窓の既定位置が左サイドバーを塞ぎ、「次の図種」ボタンが押せない | 初期位置をサイドバー幅を差し引いた位置へ (広い画面 272px / 狭幅 16px)。以降開いた図窓は連番でずれる |
| L6 | B | 降級経路が `undefined` を返す (cleanup 登録の陷阱) | noop cleanup を明示的に返す |
| L7 | C | 5D の `Overlay3D` / `SplitSynchronized3D` が**受動的な badge だけ** | 3D は bus を購読しないため**未配線**。BACKLOG へ明示 (意図的に機能名を外さない) |
| L8 | C | `FirstDivergence` パネルが eventA/eventB (同一 ID) のみで差が読めない | 状態遷移 (`prev→next`) と比較キーを併記 (`firstDivergenceDetail`) |
| L9 | C | 注入故障が `outcome` に反映されない (A=SUCCESS B=SUCCESS) | 故障命中時 `outcome='FAILURE'` に |
| L10 | C | 5D UI が ScenarioGraphDelta を見せない | 「図増分」パネルを追加 (追加ノード / 除去辺 / 改写 / 無効化) |
| L11 | C | 終態差異、片側のみに存在するノードを「結果差異」として混入 | **両分岐共通ノードのみ**比較。構造差分は別項目 (`structuralOnlyNodeIds`) |
| L12 | C | 未使用 import (`replaceText`) が tree-shake され `ui.ts` の `fill` が死コード化 | import を削除 |

---

## 5. 残存制約 (NOT_TESTED — PASS の範囲外)

これらは**故意に未実施**であり、隠していない。QA_FINAL.md §8 と BACKLOG.md に同じ内容を記録。

| # | 制約 | 理由 | 解除条件 |
|---|---|---|---|
| NT-1 | 実 GPU での 3D 描画 | 検証環境は SwiftShader (ソフトウェアラスタライザ) | 実 GPU 環境で視覚回帰 |
| NT-2 | 3D ラベルの視認性 (遠景/近景/遮蔽) | 2 解像度のみ | 多解像度 + 多視角 |
| NT-3 | カバレッジ 90% 目標 | 計測 (c8/v8) 未導入 | `@vitest/coverage-v8` 導入 |
| NT-4 | axe-core による a11y 自動検査 | Playwright 統合が未完 | 統合してゲート化 |
| NT-5 | 2D 14 図種の全数目視レビュー | 各図種の記号を目視確認していない | 図種 × 状態の一括スクリーンショット |
| NT-6 | 5D 10 干渉種の実動確認 | 一部のみ実行 | 10 種 × 代表 Case の網羅実行 |
| NT-7 | 認知受容性実験 (3 タスク) | 被験者が居ない | 5 人で課題実験 |
| NT-8 | ja-JP / en-US 正式版 | 人工翻訳必要 | 翻訳 + 校閲 |
| NT-9 | 5D の 3D 重畳 (Overlay3D / SplitSynchronized3D) | **解消済** — 共有ステージに `setCausalPresentation()` を実装、split は 2 パス実描画 | 済 |
| NT-10 | 4D の実行が**明示ステップ列**駆動でありグラフ自動走査ではない | グラフ改写は構造を変えるが、Case のステップ列が参照しないと結果が変わらない | 仕様未確定として明示。Engine をグラフ走査型に拡張するか、Case をグラフ基準で定義するかを決める |
| NT-11 | Overlay/Split は**同一カメラ**の左右 2 パス | 真の 2 カメラ分割ではない | 独立カメラ (2 ビュー + 独立視点) の導入 |

---

## 6. 審査の再実行方法

審査員の再現スクリプトは `tests/tools/` に残っている。すべて読み取り専用。

```powershell
cd D:\RustGameServer\docs\gemarc-5d-atlas-v10.0
node tests/tools/audit-tables.mjs            # 幻影テーブル / 重複 / PK・FK
node tests/tools/audit-impl-edges.mjs        # Implemented 辺の分類妥当性
node tests/tools/probe-responsive.mjs        # 狭幅の横溢れ
node tests/tools/probe-listener-leak.mjs     # 2D リスナ累積
node tests/tools/probe-3d-labels.mjs        # 3D ラベル描画
node tests/tools/probe-causal-prototype.mjs  # T7/T8/T9/T10
node tests/verify-visual.mjs qa/visual       # 6 次元 × 2 ビューポート
```

> **審査 rigor の記録**: 審査員 A は「note に『真实使用点』と書いてあるだけでは不十分」という
> 論点で 4 件の偽陽性を検出した。これはまさに本図集が防ぐべき「報告と事実の乖離」そのものであり、
> 審査が**些細なレベルでも検出できなかった**。固定された検証 (stripRustComments + stripRustStrings + ソース行再読)
> を通して塞いだ。

---

**審査終了** — 発見 34 件 (High 6 / Medium 16 / Low 12)、**全件処置済**、残存制約は §5 に明示。
