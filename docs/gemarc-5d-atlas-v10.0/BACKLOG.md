# BACKLOG.md — Universal GemArc 5D Design Atlas v10.0

> **対象**: RustGameServer (RGS)
> **ソース Revision**: `b6a998174582` (branch `dev`)
> **更新**: 2026-10-21
> **原則**: 未解決の項目は**捏造せず**、状態と責任者と受入計画を書く。
> spec `gap_handling`: GAP_SOURCE_DOCUMENT / GAP_BUSINESS_DECISION / ATLAS_FAIL / NOT_APPLICABLE の 4 種で分類する。

---

## 1. ギャップ分類サマリ

| 分類 | 件数 | 意味 |
|---|---|---|
| **GAP_SOURCE_DOCUMENT** | 6 | 源要件 / 設計 / テスト資料が未整備。図集の実装失敗**ではない** |
| **GAP_BUSINESS_DECISION** | 4 | 製品側の判断待ち。**デフォルトの事実を捏造しない** |
| **ATLAS_FAIL** | 0 | 図集側の漏落・関連誤り・挙動失敗 |
| **NOT_APPLICABLE** | 3 | 該当なし (理由を明記) |

---

## 2. GAP_SOURCE_DOCUMENT — 源資料の欠落

| # | 項目 | 影響 | 担当 | 受入計画 | 状態 |
|---|---|---|---|---|---|
| SD-1 | per-service 業務要件の不存在 | 業務ルール詳細が TBD のまま | 各ドメイン Lead (6 ドメイン) | 要件定義書を各 Lead が作成し、差分反映 | ⏳ |
| SD-2 | 業務 SLA / NFR 目標値の未確定 | 非機能要求に実測値を置けない | SRE Lead | 2 週間の実測 (p95/p99/RPS) → 値を反映 | ⏳ |
| SD-3 | Rust テスト ↔ Case の 1:1 対応なし | `traceability.testToCase` が空 | QA Lead | 対応付け rubric を確定し自動生成 | ⏳ |
| SD-4 | 対象製品の Web フロントエンド実装の不存在 | 画面設計・画面遷移が TBD (gm-console は envoy 静的配信のみ) | gm-backend 所有者 | 管理コンソール画面仕様を定義 | ⏳ |
| SD-5 | 認証 / 認可方式の未確定 | セキュリティ章の TBD | セキュリティ Lead | mTLS 適用範囲 + 認可粒度を明文化 | ⏳ |
| SD-6 | クロスト Storage 境界の未確定 | データモデルの章で TBD | DBA / 各ドメイン Lead | マルチテナント分離方式を確定 | ⏳ |

---

## 3. GAP_BUSINESS_DECISION — 製品判断待ち

| # | 項目 | 影響 | 担当 | 受入計画 | 状態 |
|---|---|---|---|---|---|
| BD-1 | ja-JP 正式版の作成 | 正式設計文書が ja-JP で読めない | 設計 (Mavis 代签) | 人工翻訳 + 校閲 (機翻は不可) | ⏳ |
| BD-2 | en-US 版の制作 | 英語話者の review を支撑できない | 設計 (Mavis 代签) | 機翻 + 人工校閲 | ⏳ |
| BD-3 | 認知受容性実験 (3 タスク) の実施 | 設計の理解度が未検証 | UX | 5 人被験者での課題実験 (成功率/操作数/所要時間) | ⏳ |
| BD-4 | `dist/` ビルド成果物の導入 | 静的 HTTP 配下の確認手順が無い | SRE Lead | 静的配信手順を定義してから生成 | ⏳ |

---

## 4. ATLAS_FAIL — 図集側の欠陥

**2026-10-21 対抗審査 (3 名の独立審査員) で 34 件 (High 6 / Medium 16 / Low 12) が出され、全件処置済み。**

内訳・根本原因・検証手段は `ADVERSARIAL_AUDIT.md` を参照。

| # | 重大度 | 項目 | 状態 |
|---|---|---|---|
| H1 | High | SQL 解析がコメントを除去せず**幻影テーブル 24 個 (名前 "IF")** を生成 | ✅ 修正済 (表 96→61) |
| H2 | High | README / QA_FINAL の数値が最新モデルと不一致 | ✅ 機械生成に統一 |
| H3 | High | **3D ラベル層が 0 個描画** (LabelLayer に公開 API が無い) | ✅ 修正済 (登録 141 / Smart 可視 10) |
| H4 | High | **狭幅で検査器/目录ドロワーが開けない** (トグルが画面外) | ✅ ヘッダーに常時ハンドル追加 |
| H5 | High | **Prototype プレビューが型エラーで完全クラッシュ** | ✅ 修正済 (子 33 / 実入力 5 / エラー 0) |
| H6 | High | **旧 Edge が削除直後に再作成され改写が無効** | ✅ 修正済 (A→X→B のみ生成) |
| M1-M16 | Medium | `use` インポート/文字列リテラル誤判定、恒真チェック (GIT-STATE / BUND-LIC)、stub 報告の PASS 化、2D リスナ累積、ER 空表示、gizmo 左手系ずれ、護衛評価不能、Konva→DOM 未同期、RewireGraph 削除のみ 他 | ✅ 全件修正済 |
| L1-L12 | Low |  Various | ✅ 全件修正済 (残るものは §6 へ) |

**ATLAS_FAIL 残存: 0 件。**

---

## 5. NOT_APPLICABLE — 該当なし (理由明記)

| # | 項目 | 理由 |
|---|---|---|
| NA-1 | ソースドメイン / 参照 GitHub リポジトリ | prompt の入力が空。`reference_github_repositories[0].url = ""` のため borrow / do_not_borrow は**適用されない**。実データ源はローカルの RGS リポジトリのみ |
| NA-2 | 対象製品そのものの実装 | `task.build_target_product: false` / `deploy_target_product: false`。本図集は**設計と計測**であり、対象製品の開発・配備は行わない |
| NA-3 | 本番 API 呼び出し | `prototype_workbench.data`: "無授権では対象本番 API を呼び出さない"。全モックは deterministic fixture |

---

## 6. 実装上の既知の限定 (NOT_TESTED として明示)

| # | 項目 | 理由 | 解除条件 |
|---|---|---|---|
| NT-1 | 実 GPU での 3D 描画 | 検証環境は SwiftShader (ソフトウェアラスタライザ) | 実 GPU 環境で視覚回帰を実行 |
| NT-2 | 3D ラベルの視認性 (遠景/近景/遮蔽) | スクリーンショットは 1440/390 の 2 解像度のみ | 多解像度 + 複数视角で検証 |
| NT-3 | カバレッジ 90% 目標 | カバレッジ計測 (c8/v8) 未導入 | `@vitest/coverage-v8` を導入し実分母を取得 |
| NT-4 | axe-core による a11y 自動検査 | axe-core は導入済みだが CI 統合が未完了 | Playwright に axe 統合しゲート化 |
| NT-5 | 2D の 14 図種の全数レビュー | 各図種の記号を 1 枚ずつ目視確認していない | 図種 × 状態マトリクスのスクリーンショット出一括レビュー |
| NT-6 | 5D の 10 干预種の実動確認 | 監査で一部のみ実行 | 10 種 × 代表 Case の網羅実行 |
| NT-7 | 5D の 3D 重畳 (Overlay3D / SplitSynchronized3D) | 3D が bus を購読していないため受動的 badge のまま | 3D 側 bus 購読を実装するか、モード名を外す |
| NT-8 | 認知受容性実験 (3 タスク) | 設計の理解度が未検証 | 5 人被験者での課題実験 |

---

## 7. 監査で判明した実バグ台帳 (修正済み)

本セッションで**実バグが見つかり修正済み**。再発防止の手がかりとして記録する。

| # | 場所 | 内容 | なぜ起きるか | 再発防止 |
|---|---|---|---|---|
| BUG-1 | `src/styles.css` | `.modal-root { display: grid }` が `hidden` 属性を上書き → 全画面遮罩が常にクリックを吸収 | クラスセレクタは UA スタイル表の `[hidden]` より優先順位が高い | `[hidden] { display: none !important }` を追加 |
| BUG-2 | `src/main.ts` | `App.status()` 未定義 → boot が中断し、revision が "—" のまま | `status` は DimensionContext にしか無く、App クラスには無い | `App.status()` / `App.renderSourceInfo()` を追加し、起動時に必ず呼ぶ |
| BUG-3 | `src/dims/d3.ts` | イベント配線を `renderToolbar()` の**前**に書いて `null.addEventListener` | ツールバーが未生成で querySelector が null | 呼び出し順序を修正し、コメントで明記 |
| BUG-4 | `src/dims/d3.ts` | `CameraPreset` のフィールドを `{yaw,pitch,distance}` と誤読 → `undefined` でカメラ位置が NaN → **画面が真っ黒** | 実際のフィールドは `{az, el, dist}` | 実測値を検証済み。スクリーンショットでの目視確認を工程に必須化 |
| BUG-5 | `src/three/model.ts` | `portIndex` に `PortInfo` (position あり) を入れず生 `GraphPort` を格納 → 3D 構築時に `reading 'x'` | `PortInfo` 型を定義したが実装が未接続 | ポート位置を所属 crate のサービスノードまわりに決定論的に配置 |
| BUG-6 | `src/dims/prototype.ts` | `ui.ts` の `fill` は「消去して置換」だが**積み上げ**に使っていた → Screen タブ行とツールバーボタンが消滅 | helper の semantics を誤解 | ローカル `fill` を追記版に再定義、`replaceText` を明示的に導入 |
| BUG-7 | `scripts/capture-evidence.mjs` | 列型解析が `UUID PRIMARY KEY` の PRIMARY KEY を型に吸い込み、**96 表すべての pkColumns が空** | 型が `[A-Z][A-Z0-9 ]*` の貪欲マッチ | 列名の後ろ全体を制約区として解析し、PK/FK を正規表現で抽出 |
| BUG-8 | `scripts/capture-evidence.mjs` | コメント内の client 名を**実装済み**と誤判定 | 生の正規表現がコメントにも一致 | Rust コメントを完全剥离 (`stripRustComments`) してから実使用点のみ採用 |
| BUG-9 | `vitest.config.ts` | 既定 include が Playwright の E2E spec を拾い `success:false` | 既定 include が全 .spec を対象にする | `include: ['tests/unit/**/*.test.ts']` に限定 |
| BUG-10 | `scripts/generate-formal-docs.mjs` | 単一引用符文字列が改行をまたいで JS 構文エラー | 複数行テキストに `'` 文字列を使った | テンプレートリテラルへ置換 |
| BUG-11 | `src/styles.css` | 狭幅で app 外殻が **713px** まで MLE し横スクロール (390px ビューポート) | grid の auto 列がヘッダーの max-content に引き寄せられた | `.app { grid-template-columns: minmax(0,1fr) }` + `.dim-nav { flex:1 1 auto; min-width:0 }` + 狭幅で chip/検索を隠す |
| BUG-12 | `src/dims/d3.ts` | `renderToolbar()` を 2 度呼び、**イベントリスナが失われた** (軸ギズモの aria-pressed が切り替わらない) | 修正ために先に呼んだ分、初期化側にも残っていた | 初期化側の重複呼び出しを削除。リセットは select の値を原地更新 |
| BUG-13 | `src/three/labels.ts` | `LabelLayer.ensure()` が private で呼び出し口なし → 3D の**ラベルが 1 つも描画されない** | API の設計漏れ | `setTargets()` を公開 API として追加 |
| BUG-14 | `src/dims/prototype.ts` | `props.tabs` / `props.items` (オブジェクト配列) を文字列と仮定 → `appendChild` が TypeError → **プレビュー全体が白紙** | props の型を仮定していた | `labelOf()` / `textOf()` ヘルパで文字列・オブジェクト両対応。描画例外は要素単位で捕捉 |
| BUG-15 | `src/sim/engine.ts` | 旧辺を削除した直後に**同一端点の直連辺を再作成** → 改写が無効 | rewired 分岐で挿入要素と通常付け替えを分岐していなかった | `X-` 前置時は `-in`/`-out` のみ生成 |
| BUG-16 | `scripts/capture-evidence.mjs` | SQL の**コメント内の `CREATE TABLE`** を拾い **幻影テーブル 24 個 (名前 "IF")** を生成 | SQL パスにコメント除去が無く、Rust パスにはあった | `stripSqlComments()` を追加。`IF/NOT/EXISTS` 名称も拒否 |
| BUG-17 | `scripts/verify.mjs` | `GIT-STATE` が `dirty.length===0?'PASS':'PASS'` で**恒真**。`BUND-LIC` も配列非空判定のみ | 数値 truthiness だけで判定していた | 実際の git 状態と**産物中のライブラリ実測**で判定 |

> **教訓**: BUG-4 / BUG-11 / BUG-13 / BUG-14 は、**静的な検査では検出できず**、
> 実ブラウザのスクリーンショットあるいは実 DOM 上での操作でしか現れなかった。図集の品質検証は
> 「コンソールエラーなし」だけでは不十分で、**描画結果の目視確認**と
> **実際の操作**が必須である。独立審査員に出させるのも有効だった
> (発見 High 6 件は自検では出なかった)。

### 7.2 次元チェーン実装 (1D→2D→3D→4D→5D) で判明したもの

次元を「階段」に「並列」ではなく作り直す過程で、**同種のバグが 3 回**出た。
いずれも「innerHTML を書いてから querySelector する」順序と、
「renderToolbar を 2 回呼んでイベントリスナを失う」型。

| # | 場所 | 内容 | 再発防止 |
|---|---|---|---|
| BUG-18 | `src/dims/d3.ts` `d4.ts` `d5.ts` | イベント配線を `renderToolbar()` の**前**に書いて `null.addEventListener` | 順序を修正し、コメントで明記。共通パッチで 3 ファイルを一括修正 |
| BUG-19 | `src/dims/d3.ts` `d4.ts` `d5.ts` | `renderToolbar()` を **2 回**呼び、2 回目で innerHTML が差し替わり**イベントリスナが消失** (軸ギズモの aria-pressed が切り替わらない等) | 重複呼び出しを除去。`renderToolbar` は mount 時に 1 度だけ |
| BUG-20 | `src/dims/d4.ts` `d5.ts` | `renderHost()` が情報パネルを**積み上げ** (前回分が残る) | `d4-info` / `d5-info` クラスで前回分を除去してから差し替え |
| BUG-21 | `src/dims/d3.ts` | 3D 書き直し時に**軸ギズモのボタンと実体を落とした** | ギズモを SceneHost 側へ移し `toggleGizmo()` として公開。E2E J04 が回帰を検知 |

### 7.3 対抗審査で見つかったもの

上記 BUG-1〜BUG-21 に加え、対抗審査で 34 件 (High 6 / Medium 16 / Low 12) が発見され全件修正。
詳細は `ADVERSARIAL_AUDIT.md`。


---

## 8. 次のアクション (優先順)

| 優先 | アクション | 担当 | 依存 |
|---|---|---|---|
| P0 | 対抗審査の残 HIGH を修正 | 設計 (Mavis 代签) | 審査結果 |
| P0 | `qa/adversarial/` の結果を反映して QA_FINAL.md を再生成 | 設計 | 審査結果 |
| P1 | 2D 14 図種の全数目視レビュー + スクリーンショット出一括 | 設計 | — |
| P1 | 5D 10 干预種 × 代表 Case の網羅実行 | 設計 | — |
| P1 | `@vitest/coverage-v8` 導入してカバレッジ実分母を取得 | 設計 | — |
| P2 | axe-core を Playwright に統合 | 設計 | — |
| P2 | ja-JP / en-US 版 | 設計 | BD-1 / BD-2 |
| P3 | `dist/` 静的配下の検証手順 | SRE Lead | BD-4 |

---

**本ファイルは欠落を隠さないための台帳である。項目を消さずに、状態だけを更新すること。**
