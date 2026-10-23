# API 設計 (API Design)

> **参照位置づけ**: 本文書は日本の IPA 共通フレームワーク / 非機能要求グレード /
> 適用されるアーキテクチャ・UML・ER の実践を**審査の参考**として用いている。
> 公式の固定テンプレートであることを意味せず、公式認証pla���一切取得していない。

## 9.1. 契約

- gRPC (proto3)。パッケージ命名規則: `<domain>.v1`
- 共通型: `common.v1` (`crates/shared-platform/proto/common/v1/common.proto`)
- 認証/認可の方式: - **TBD**: 未確定。担当: セキュリティ Lead。 受入計画: mTLS + 認可方式の確定

## 9.2. エラーコード分類 (実装済みのもの)

`crates/shared-platform/src/retry.rs` の `is_retryable` は以下を再試行可能と判定する:

| gRPC Code | 意味 |
|---|---|
| `Unavailable` | 一時的なサービス利用不可 |
| `DeadlineExceeded` | タイムアウト |
| `Aborted` | 競合により中断 |
| `ResourceExhausted` | リソース枯渇 |

上記以外のコードは再試行しない (fail-fast)。

## 9.3. サービス別メソッド数

- `activity.v1.ActivityService`: 30 メソッド
- `admin.v1.AdminService`: 6 メソッド
- `battle.v1.BattleEngineService`: 32 メソッド
- `battle.v1.PvPService`: 30 メソッド
- `battle.v1.BossService`: 15 メソッド
- `battle.v1.RoomService`: 46 メソッド
- `battle.v1.InstanceService`: 6 メソッド
- `battle.v1.EndlessTowerService`: 13 メソッド
- `battle.v1.EscortService`: 18 メソッド
- `battle.v1.HolyEquipService`: 24 メソッド
- `battle.v1.GuildWarService`: 17 メソッド
- `battle.v1.CrossServerService`: 19 メソッド
- `battle.v1.ExpeditionService`: 15 メソッド
- `battle.v1.HolidayActivityService`: 15 メソッド
- `card.v1.CardService`: 9 メソッド
- `cluster_ops.v1.ClusterOpsService`: 3 メソッド
- `economy.v1.EconomyService`: 92 メソッド
- `gm.v1.GmBackend`: 5 メソッド
- `gm_extra.v1.GmExtraService`: 10 メソッド
- `guild.v1.GuildService`: 30 メソッド
- `i18n.v1.I18nService`: 4 メソッド
- `leaderboard_extra.v1.LeaderboardExtraService`: 7 メソッド
- `leaderboard.v1.LeaderboardService`: 6 メソッド
- `match.v1.MatchService`: 10 メソッド
- `gateway.v1.GatewayAdmin`: 4 メソッド
- `operate.v1.OperateService`: 7 メソッド
- `player.v1.PlayerService`: 26 メソッド
- `pvp_full.v1.PvpFullService`: 30 メソッド
- `replay_extra.v1.ReplayExtraService`: 10 メソッド
- `replay.v1.ReplayService`: 9 メソッド
- `scene.v1.SceneService`: 145 メソッド
- `social_extra.v1.SocialExtraService`: 15 メソッド
- `social.v1.SocialService`: 2 メソッド
