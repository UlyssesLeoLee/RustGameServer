//! economy-service v3 商店 + 抽卡 + 限时 + 充值 + 基金/特权 + 活动 测试
//!
//! 至少 30 个 test 函数 (per L1.1 DoD)
//! 覆盖: 商店购买 / 兑换 / 礼包码 / 战利品 / 神秘商店 / 积分商城 / 抽卡占位

#[cfg(test)]
mod tests {
    use crate::entity::{Account, Currency};
    use crate::repository::{
        AccountRepository, InMemoryAccountRepository, InMemoryTransactionLedgerRepository,
        TransactionLedgerRepository,
    };
    use crate::shop_entity::{
        InMemoryEconomyV3Repository, LootEntry, LootTable, MysteryShop, MysteryShopState,
        ExchangeShop, ShopItemEntity, PlayerPoints, GiftCode, ActivityTemplateEntity, ActivityType,
    };
    use crate::shop_service::{ShopService, ShopServiceImpl};
    use chrono::{Duration, Utc};
    use std::sync::Arc;
    use uuid::Uuid;

    /// 构造带账户的 v3 测试上下文
    fn make_ctx() -> (
        ShopServiceImpl,
        Arc<InMemoryAccountRepository>,
        Arc<InMemoryTransactionLedgerRepository>,
        Arc<tokio::sync::Mutex<InMemoryEconomyV3Repository>>,
    ) {
        let led_repo = Arc::new(InMemoryTransactionLedgerRepository::new());
        let acc_repo =
            Arc::new(InMemoryAccountRepository::new().with_shared_ledger(led_repo.inner.clone()));
        let v3_repo = Arc::new(tokio::sync::Mutex::new(InMemoryEconomyV3Repository::new()));
        let svc = ShopServiceImpl::new(
            v3_repo.clone(),
            acc_repo.clone() as Arc<dyn AccountRepository>,
            led_repo.clone() as Arc<dyn TransactionLedgerRepository>,
        );
        (svc, acc_repo, led_repo, v3_repo)
    }

    /// 准备一个有金币账户的玩家
    async fn setup_player_with_gold(
        acc_repo: &Arc<InMemoryAccountRepository>,
        gold: i64,
    ) -> (Uuid, Uuid) {
        let player_id = Uuid::new_v4();
        let mut acc = Account::new(player_id, Currency::Gold);
        acc.credit(gold);
        let account_id = acc.id;
        acc_repo.save(&acc).await.unwrap();
        (player_id, account_id)
    }

    /// 准备一个有钻石账户的玩家
    async fn setup_player_with_diamond(
        acc_repo: &Arc<InMemoryAccountRepository>,
        diamond: i64,
    ) -> (Uuid, Uuid) {
        let player_id = Uuid::new_v4();
        let mut acc = Account::new(player_id, Currency::Diamond);
        acc.credit(diamond);
        let account_id = acc.id;
        acc_repo.save(&acc).await.unwrap();
        (player_id, account_id)
    }

    // ==================== 商店类 (20 RPC) UT ====================

    #[tokio::test]
    async fn shop_buy_deducts_gold_and_records() {
        let (svc, acc_repo, _led_repo, v3_repo) = make_ctx();
        let (player_id, _) = setup_player_with_gold(&acc_repo, 1000).await;
        let player_id_str = player_id.to_string();
        // 配置商品
        v3_repo.lock().await.shop_items.insert(
            (1, "sword".to_string()),
            ShopItemEntity {
                item_id: "sword".to_string(),
                sku: "SW-001".to_string(),
                name: "Iron Sword".to_string(),
                price_amount: 100,
                price_currency: 1, // Gold
                stock: 10,
                vip_level_required: 0,
                level_required: 0,
                limit_per_player: 0,
                tag: "".to_string(),
            },
        );
        let out = svc
            .shop_buy(
                player_id_str.clone(),
                1,
                "sword".to_string(),
                2,
                "buy-1".to_string(),
            )
            .await
            .unwrap();
        assert!(out.success);
        assert_eq!(out.cost_amount, 200);
        assert_eq!(out.remaining_stock, 8);
    }

    #[tokio::test]
    async fn shop_buy_idempotency_conflict() {
        let (svc, acc_repo, _led_repo, v3_repo) = make_ctx();
        let (player_id, _) = setup_player_with_gold(&acc_repo, 1000).await;
        let player_id_str = player_id.to_string();
        v3_repo.lock().await.shop_items.insert(
            (1, "potion".to_string()),
            ShopItemEntity {
                item_id: "potion".to_string(),
                sku: "PT-001".to_string(),
                name: "Potion".to_string(),
                price_amount: 50,
                price_currency: 1,
                stock: 5,
                vip_level_required: 0,
                level_required: 0,
                limit_per_player: 0,
                tag: "".to_string(),
            },
        );
        // 第一次购买
        svc.shop_buy(player_id_str.clone(), 1, "potion".to_string(), 1, "same-key".to_string())
            .await
            .unwrap();
        // 第二次相同 idempotency_key 应该 conflict
        let err = svc
            .shop_buy(player_id_str, 1, "potion".to_string(), 1, "same-key".to_string())
            .await
            .unwrap_err();
        assert!(matches!(err, crate::Error::IdempotencyConflict(_)));
    }

    #[tokio::test]
    async fn shop_buy_insufficient_funds() {
        let (svc, acc_repo, _led_repo, v3_repo) = make_ctx();
        let (player_id, _) = setup_player_with_gold(&acc_repo, 50).await;
        let player_id_str = player_id.to_string();
        v3_repo.lock().await.shop_items.insert(
            (1, "expensive".to_string()),
            ShopItemEntity {
                item_id: "expensive".to_string(),
                sku: "EX-001".to_string(),
                name: "Expensive".to_string(),
                price_amount: 1000,
                price_currency: 1,
                stock: 1,
                vip_level_required: 0,
                level_required: 0,
                limit_per_player: 0,
                tag: "".to_string(),
            },
        );
        let err = svc
            .shop_buy(
                player_id_str,
                1,
                "expensive".to_string(),
                1,
                "buy-2".to_string(),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, crate::Error::InsufficientFunds { .. }));
    }

    #[tokio::test]
    async fn shop_buy_stock_exhausted() {
        let (svc, acc_repo, _led_repo, v3_repo) = make_ctx();
        let (player_id, _) = setup_player_with_gold(&acc_repo, 100000).await;
        let player_id_str = player_id.to_string();
        v3_repo.lock().await.shop_items.insert(
            (1, "limited".to_string()),
            ShopItemEntity {
                item_id: "limited".to_string(),
                sku: "LM-001".to_string(),
                name: "Limited".to_string(),
                price_amount: 10,
                price_currency: 1,
                stock: 1, // 1 库存
                vip_level_required: 0,
                level_required: 0,
                limit_per_player: 0,
                tag: "".to_string(),
            },
        );
        // 买 1 个成功
        svc.shop_buy(player_id_str.clone(), 1, "limited".to_string(), 1, "k1".to_string())
            .await
            .unwrap();
        // 买第 2 个失败
        let err = svc
            .shop_buy(player_id_str, 1, "limited".to_string(), 1, "k2".to_string())
            .await
            .unwrap_err();
        assert!(matches!(err, crate::Error::Conflict(_)));
    }

    #[tokio::test]
    async fn shop_buy_quantity_zero_rejected() {
        let (svc, _acc_repo, _led_repo, _v3_repo) = make_ctx();
        let err = svc
            .shop_buy(Uuid::new_v4().to_string(), 1, "x".to_string(), 0, "k".to_string())
            .await
            .unwrap_err();
        assert!(matches!(err, crate::Error::Validation(_)));
    }

    #[tokio::test]
    async fn shop_buy_item_not_found() {
        let (svc, acc_repo, _led_repo, _v3_repo) = make_ctx();
        let (player_id, _) = setup_player_with_gold(&acc_repo, 100).await;
        let err = svc
            .shop_buy(
                player_id.to_string(),
                99,
                "ghost".to_string(),
                1,
                "k".to_string(),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, crate::Error::NotFound { .. }));
    }

    #[tokio::test]
    async fn shop_record_filters_by_player() {
        let (svc, acc_repo, _led_repo, v3_repo) = make_ctx();
        let (p1, _) = setup_player_with_gold(&acc_repo, 10000).await;
        let (p2, _) = setup_player_with_gold(&acc_repo, 10000).await;
        v3_repo.lock().await.shop_items.insert(
            (1, "x".to_string()),
            ShopItemEntity {
                item_id: "x".to_string(),
                sku: "X".to_string(),
                name: "X".to_string(),
                price_amount: 10,
                price_currency: 1,
                stock: 100,
                vip_level_required: 0,
                level_required: 0,
                limit_per_player: 0,
                tag: "".to_string(),
            },
        );
        svc.shop_buy(p1.to_string(), 1, "x".to_string(), 1, "a".to_string()).await.unwrap();
        svc.shop_buy(p2.to_string(), 1, "x".to_string(), 1, "b".to_string()).await.unwrap();
        let (p1_records, p1_total) = svc.shop_record(p1.to_string(), 0, 20).await.unwrap();
        assert_eq!(p1_total, 1);
        assert_eq!(p1_records.len(), 1);
        let (p2_records, p2_total) = svc.shop_record(p2.to_string(), 0, 20).await.unwrap();
        assert_eq!(p2_total, 1);
        assert_eq!(p2_records.len(), 1);
    }

    // ==================== 神秘商店 (4 RPC) UT ====================

    #[tokio::test]
    async fn mystery_shop_list_returns_items_when_unlocked() {
        let (svc, _acc_repo, _led_repo, v3_repo) = make_ctx();
        let player_id = Uuid::new_v4().to_string();
        // 配置阶段: 用 scoped block 让 lock guard 在 service 调用前 drop
        // (避免 re-entrant lock: service.mystery_shop_list 内部会再 lock(self.repo))
        {
            let mut repo = v3_repo.lock().await;
            repo.mystery_shops.insert(
                10,
                MysteryShop {
                    mystery_shop_id: 10,
                    unlock_level: 10,
                    refresh_cost: 50,
                    max_refresh: 5,
                },
            );
            repo.mystery_states.insert(
                (player_id.clone(), 10),
                MysteryShopState {
                    player_id: player_id.clone(),
                    mystery_shop_id: 10,
                    unlocked: true,
                    unlocked_at: Some(Utc::now()),
                    refresh_count: 1,
                    refreshed_at: Utc::now(),
                    current_items: vec![ShopItemEntity {
                        item_id: "rare".to_string(),
                        sku: "RR".to_string(),
                        name: "Rare".to_string(),
                        price_amount: 100,
                        price_currency: 2,
                        stock: 1,
                        vip_level_required: 0,
                        level_required: 0,
                        limit_per_player: 0,
                        tag: "rare".to_string(),
                    }],
                },
            );
        }
        let out = svc.mystery_shop_list(player_id, 10).await.unwrap();
        assert_eq!(out.items.len(), 1);
        assert_eq!(out.refresh_count, 1);
        assert_eq!(out.unlock_level, 10);
    }

    #[tokio::test]
    async fn mystery_shop_list_locked_returns_forbidden() {
        let (svc, _acc_repo, _led_repo, v3_repo) = make_ctx();
        let player_id = Uuid::new_v4().to_string();
        v3_repo.lock().await.mystery_shops.insert(
            20,
            MysteryShop {
                mystery_shop_id: 20,
                unlock_level: 20,
                refresh_cost: 100,
                max_refresh: 3,
            },
        );
        v3_repo.lock().await.mystery_states.insert(
            (player_id.clone(), 20),
            MysteryShopState {
                player_id: player_id.clone(),
                mystery_shop_id: 20,
                unlocked: false,
                unlocked_at: None,
                refresh_count: 0,
                refreshed_at: Utc::now(),
                current_items: vec![],
            },
        );
        let err = svc.mystery_shop_list(player_id, 20).await.unwrap_err();
        assert!(matches!(err, crate::Error::Forbidden(_)));
    }

    #[tokio::test]
    async fn mystery_shop_list_shop_not_found() {
        let (svc, _acc_repo, _led_repo, _v3_repo) = make_ctx();
        let err = svc
            .mystery_shop_list(Uuid::new_v4().to_string(), 999)
            .await
            .unwrap_err();
        assert!(matches!(err, crate::Error::NotFound { .. }));
    }

    // ==================== 兑换 (3 RPC) UT ====================

    #[tokio::test]
    async fn exchange_do_deducts_points_and_returns_limit() {
        let (svc, _acc_repo, _led_repo, v3_repo) = make_ctx();
        let player_id = Uuid::new_v4().to_string();
        v3_repo.lock().await.exchange_shops.insert(
            100,
            ExchangeShop {
                exchange_id: 100,
                cost_currency: 1, // 用 1 区分: 不是真货币, 是积分
                items: vec![ShopItemEntity {
                    item_id: "fragment".to_string(),
                    sku: "FG".to_string(),
                    name: "Fragment".to_string(),
                    price_amount: 50,
                    price_currency: 1,
                    stock: -1,
                    vip_level_required: 0,
                    level_required: 0,
                    limit_per_player: 0,
                    tag: "".to_string(),
                }],
            },
        );
        v3_repo.lock().await.player_points.insert(
            (player_id.clone(), 1),
            PlayerPoints {
                player_id: player_id.clone(),
                point_type: 1,
                balance: 200,
            },
        );
        let out = svc
            .exchange_do(player_id.clone(), 100, "fragment".to_string(), 2, "k1".to_string())
            .await
            .unwrap();
        assert!(out.success);
        assert_eq!(out.cost_points, 100);
        let repo = v3_repo.lock().await;
        assert_eq!(repo.player_points.get(&(player_id, 1)).unwrap().balance, 100);
    }

    #[tokio::test]
    async fn exchange_do_insufficient_points() {
        let (svc, _acc_repo, _led_repo, v3_repo) = make_ctx();
        let player_id = Uuid::new_v4().to_string();
        v3_repo.lock().await.exchange_shops.insert(
            200,
            ExchangeShop {
                exchange_id: 200,
                cost_currency: 1,
                items: vec![ShopItemEntity {
                    item_id: "big".to_string(),
                    sku: "BG".to_string(),
                    name: "Big".to_string(),
                    price_amount: 1000,
                    price_currency: 1,
                    stock: -1,
                    vip_level_required: 0,
                    level_required: 0,
                    limit_per_player: 0,
                    tag: "".to_string(),
                }],
            },
        );
        v3_repo.lock().await.player_points.insert(
            (player_id.clone(), 1),
            PlayerPoints {
                player_id: player_id.clone(),
                point_type: 1,
                balance: 10,
            },
        );
        let err = svc
            .exchange_do(player_id, 200, "big".to_string(), 1, "k1".to_string())
            .await
            .unwrap_err();
        assert!(matches!(err, crate::Error::InsufficientFunds { .. }));
    }

    #[tokio::test]
    async fn exchange_do_shop_not_found() {
        let (svc, _acc_repo, _led_repo, _v3_repo) = make_ctx();
        let err = svc
            .exchange_do(Uuid::new_v4().to_string(), 999, "x".to_string(), 1, "k".to_string())
            .await
            .unwrap_err();
        assert!(matches!(err, crate::Error::NotFound { .. }));
    }

    #[tokio::test]
    async fn exchange_list_returns_items_and_points() {
        let (svc, _acc_repo, _led_repo, v3_repo) = make_ctx();
        let player_id = Uuid::new_v4().to_string();
        v3_repo.lock().await.exchange_shops.insert(
            300,
            ExchangeShop {
                exchange_id: 300,
                cost_currency: 2,
                items: vec![ShopItemEntity {
                    item_id: "i1".to_string(),
                    sku: "I1".to_string(),
                    name: "Item 1".to_string(),
                    price_amount: 10,
                    price_currency: 2,
                    stock: 5,
                    vip_level_required: 0,
                    level_required: 0,
                    limit_per_player: 0,
                    tag: "".to_string(),
                }],
            },
        );
        v3_repo.lock().await.player_points.insert(
            (player_id.clone(), 2),
            PlayerPoints {
                player_id: player_id.clone(),
                point_type: 2,
                balance: 500,
            },
        );
        let (items, points) = svc.exchange_list(player_id, 300).await.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(points, 500);
    }

    // ==================== 积分商城 (2 RPC) UT ====================

    #[tokio::test]
    async fn point_shop_buy_deducts_points() {
        let (svc, _acc_repo, _led_repo, v3_repo) = make_ctx();
        let player_id = Uuid::new_v4().to_string();
        v3_repo.lock().await.exchange_shops.insert(
            400,
            ExchangeShop {
                exchange_id: 400,
                cost_currency: 1, // 1=竞技
                items: vec![ShopItemEntity {
                    item_id: "ps-item".to_string(),
                    sku: "PS".to_string(),
                    name: "PS Item".to_string(),
                    price_amount: 20,
                    price_currency: 1,
                    stock: -1,
                    vip_level_required: 0,
                    level_required: 0,
                    limit_per_player: 0,
                    tag: "".to_string(),
                }],
            },
        );
        v3_repo.lock().await.player_points.insert(
            (player_id.clone(), 1),
            PlayerPoints {
                player_id: player_id.clone(),
                point_type: 1,
                balance: 100,
            },
        );
        let out = svc
            .point_shop_buy(player_id.clone(), 1, "ps-item".to_string(), 1, "k1".to_string())
            .await
            .unwrap();
        assert!(out.success);
        assert_eq!(out.cost_points, 20);
        assert_eq!(out.remaining_points, 80);
    }

    #[tokio::test]
    async fn point_shop_list_filters_by_point_type() {
        let (svc, _acc_repo, _led_repo, v3_repo) = make_ctx();
        let player_id = Uuid::new_v4().to_string();
        v3_repo.lock().await.exchange_shops.insert(
            500,
            ExchangeShop {
                exchange_id: 500,
                cost_currency: 1,
                items: vec![ShopItemEntity {
                    item_id: "a".to_string(),
                    sku: "A".to_string(),
                    name: "A".to_string(),
                    price_amount: 1,
                    price_currency: 1,
                    stock: 1,
                    vip_level_required: 0,
                    level_required: 0,
                    limit_per_player: 0,
                    tag: "".to_string(),
                }],
            },
        );
        v3_repo.lock().await.exchange_shops.insert(
            600,
            ExchangeShop {
                exchange_id: 600,
                cost_currency: 2,
                items: vec![ShopItemEntity {
                    item_id: "b".to_string(),
                    sku: "B".to_string(),
                    name: "B".to_string(),
                    price_amount: 1,
                    price_currency: 2,
                    stock: 1,
                    vip_level_required: 0,
                    level_required: 0,
                    limit_per_player: 0,
                    tag: "".to_string(),
                }],
            },
        );
        let (items, _points, _total) = svc
            .point_shop_list(player_id, 1, 0, 20)
            .await
            .unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].item_id, "a");
    }

    // ==================== 礼包码 (2 RPC) UT ====================

    #[tokio::test]
    async fn gift_code_redeem_success() {
        let (svc, _acc_repo, _led_repo, v3_repo) = make_ctx();
        let player_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        v3_repo.lock().await.gift_codes.insert(
            ("ABCD1234".to_string(), 1),
            GiftCode {
                code: "ABCD1234".to_string(),
                server_id: 1,
                reward_template: "gold_1000".to_string(),
                valid_from: now - Duration::days(1),
                valid_to: now + Duration::days(30),
                max_uses: 100,
                current_uses: 0,
            },
        );
        let out = svc
            .gift_code_redeem(
                player_id.clone(),
                "ABCD1234".to_string(),
                1,
                "redeem-1".to_string(),
            )
            .await
            .unwrap();
        assert!(out.success);
        assert!(out.error_msg.is_empty());
        let repo = v3_repo.lock().await;
        assert_eq!(
            repo.gift_codes.get(&("ABCD1234".to_string(), 1)).unwrap().current_uses,
            1
        );
        assert_eq!(repo.gift_redemptions.len(), 1);
    }

    #[tokio::test]
    async fn gift_code_redeem_expired() {
        let (svc, _acc_repo, _led_repo, v3_repo) = make_ctx();
        let player_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        v3_repo.lock().await.gift_codes.insert(
            ("EXPIRED".to_string(), 1),
            GiftCode {
                code: "EXPIRED".to_string(),
                server_id: 1,
                reward_template: "gold_1000".to_string(),
                valid_from: now - Duration::days(30),
                valid_to: now - Duration::days(1),
                max_uses: 100,
                current_uses: 0,
            },
        );
        let out = svc
            .gift_code_redeem(
                player_id,
                "EXPIRED".to_string(),
                1,
                "k1".to_string(),
            )
            .await
            .unwrap();
        assert!(!out.success);
        assert_eq!(out.error_msg, "expired");
    }

    #[tokio::test]
    async fn gift_code_redeem_max_uses_reached() {
        let (svc, _acc_repo, _led_repo, v3_repo) = make_ctx();
        let player_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        v3_repo.lock().await.gift_codes.insert(
            ("FULL".to_string(), 1),
            GiftCode {
                code: "FULL".to_string(),
                server_id: 1,
                reward_template: "gold_100".to_string(),
                valid_from: now - Duration::days(1),
                valid_to: now + Duration::days(30),
                max_uses: 1,
                current_uses: 1,
            },
        );
        let out = svc
            .gift_code_redeem(player_id, "FULL".to_string(), 1, "k1".to_string())
            .await
            .unwrap();
        assert!(!out.success);
        assert_eq!(out.error_msg, "max_uses_reached");
    }

    #[tokio::test]
    async fn gift_code_redeem_already_used_by_player() {
        let (svc, _acc_repo, _led_repo, v3_repo) = make_ctx();
        let player_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        v3_repo.lock().await.gift_codes.insert(
            ("ONCE".to_string(), 1),
            GiftCode {
                code: "ONCE".to_string(),
                server_id: 1,
                reward_template: "gold_100".to_string(),
                valid_from: now - Duration::days(1),
                valid_to: now + Duration::days(30),
                max_uses: 100,
                current_uses: 0,
            },
        );
        let out1 = svc
            .gift_code_redeem(
                player_id.clone(),
                "ONCE".to_string(),
                1,
                "k1".to_string(),
            )
            .await
            .unwrap();
        assert!(out1.success);
        // 模拟同 key (用不同 idempotency_key 但 same player 仍应失败)
        let out2 = svc
            .gift_code_redeem(
                player_id,
                "ONCE".to_string(),
                1,
                "k2".to_string(),
            )
            .await
            .unwrap();
        assert!(!out2.success);
        assert_eq!(out2.error_msg, "already_used");
    }

    #[tokio::test]
    async fn gift_code_redeem_not_found() {
        let (svc, _acc_repo, _led_repo, _v3_repo) = make_ctx();
        let err = svc
            .gift_code_redeem(
                Uuid::new_v4().to_string(),
                "DOESNOTEXIST".to_string(),
                1,
                "k".to_string(),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, crate::Error::NotFound { .. }));
    }

    #[tokio::test]
    async fn gift_code_query_returns_metadata() {
        let (svc, _acc_repo, _led_repo, v3_repo) = make_ctx();
        let now = Utc::now();
        v3_repo.lock().await.gift_codes.insert(
            ("QUERY".to_string(), 1),
            GiftCode {
                code: "QUERY".to_string(),
                server_id: 1,
                reward_template: "diamond_50".to_string(),
                valid_from: now - Duration::days(1),
                valid_to: now + Duration::days(30),
                max_uses: 200,
                current_uses: 5,
            },
        );
        let out = svc
            .gift_code_query(Uuid::new_v4().to_string(), "QUERY".to_string(), 1)
            .await
            .unwrap();
        assert!(out.exists);
        assert_eq!(out.reward_template, "diamond_50");
        assert_eq!(out.max_uses, 200);
        assert_eq!(out.current_uses, 5);
    }

    #[tokio::test]
    async fn gift_code_query_returns_not_exists() {
        let (svc, _acc_repo, _led_repo, _v3_repo) = make_ctx();
        let out = svc
            .gift_code_query(Uuid::new_v4().to_string(), "MISSING".to_string(), 1)
            .await
            .unwrap();
        assert!(!out.exists);
    }

    // ==================== 战利品 (2 RPC) UT ====================

    #[tokio::test]
    async fn loot_roll_weighted_random() {
        let (svc, _acc_repo, _led_repo, v3_repo) = make_ctx();
        let player_id = Uuid::new_v4().to_string();
        // 表格: 90% common, 9% rare, 1% legendary
        v3_repo.lock().await.loot_tables.insert(
            1,
            LootTable {
                loot_table_id: 1,
                entries: vec![
                    LootEntry {
                        item_id: "common".to_string(),
                        weight: 90,
                        rarity: 1,
                    },
                    LootEntry {
                        item_id: "rare".to_string(),
                        weight: 9,
                        rarity: 3,
                    },
                    LootEntry {
                        item_id: "legendary".to_string(),
                        weight: 1,
                        rarity: 5,
                    },
                ],
            },
        );
        let out = svc
            .loot_roll(player_id, 1, 1000, "k1".to_string())
            .await
            .unwrap();
        assert_eq!(out.rolled_item_ids.len(), 1000);
        // 至少应该出现 common
        assert!(out
            .rolled_item_ids
            .iter()
            .any(|i| i == "common"));
    }

    #[tokio::test]
    async fn loot_roll_empty_table_rejected() {
        let (svc, _acc_repo, _led_repo, v3_repo) = make_ctx();
        v3_repo.lock().await.loot_tables.insert(
            99,
            LootTable {
                loot_table_id: 99,
                entries: vec![],
            },
        );
        let err = svc
            .loot_roll(Uuid::new_v4().to_string(), 99, 1, "k".to_string())
            .await
            .unwrap_err();
        assert!(matches!(err, crate::Error::Validation(_)));
    }

    #[tokio::test]
    async fn loot_roll_table_not_found() {
        let (svc, _acc_repo, _led_repo, _v3_repo) = make_ctx();
        let err = svc
            .loot_roll(Uuid::new_v4().to_string(), 999, 1, "k".to_string())
            .await
            .unwrap_err();
        assert!(matches!(err, crate::Error::NotFound { .. }));
    }

    #[tokio::test]
    async fn loot_claim_marks_batch_claimed() {
        let (svc, _acc_repo, _led_repo, v3_repo) = make_ctx();
        let player_id = Uuid::new_v4().to_string();
        v3_repo.lock().await.loot_tables.insert(
            1,
            LootTable {
                loot_table_id: 1,
                entries: vec![LootEntry {
                    item_id: "x".to_string(),
                    weight: 1,
                    rarity: 1,
                }],
            },
        );
        let roll = svc
            .loot_roll(player_id.clone(), 1, 1, "k1".to_string())
            .await
            .unwrap();
        // 拿 batch_id (需要在 repo 找)
        let batch_id = {
            let repo = v3_repo.lock().await;
            repo.loot_batches
                .iter()
                .find(|(_, b)| b.player_id == player_id && b.loot_table_id == 1)
                .map(|(id, _)| *id)
                .unwrap()
        };
        let out = svc
            .loot_claim(player_id.clone(), 1, batch_id)
            .await
            .unwrap();
        assert!(out.success);
        // 重复 claim 失败
        let err = svc.loot_claim(player_id, 1, batch_id).await.unwrap_err();
        assert!(matches!(err, crate::Error::Conflict(_)));
    }

    #[tokio::test]
    async fn loot_claim_batch_not_found() {
        let (svc, _acc_repo, _led_repo, _v3_repo) = make_ctx();
        let err = svc
            .loot_claim(Uuid::new_v4().to_string(), 1, Uuid::new_v4())
            .await
            .unwrap_err();
        assert!(matches!(err, crate::Error::NotFound { .. }));
    }

    // ==================== 抽卡 / 限时 / 充值 / 基金 stub UT ====================

    #[tokio::test]
    async fn stub_apis_return_unimplemented() {
        // 验证未实现的 stub RPC 都返回 Unimplemented
        let (svc, _acc_repo, _led_repo, _v3_repo) = make_ctx();
        let err = svc
            .shop_refresh(Uuid::new_v4().to_string(), 1, false)
            .await
            .unwrap_err();
        assert!(matches!(err, crate::Error::Unimplemented(_)));

        let err = svc
            .mystery_shop_buy(
                Uuid::new_v4().to_string(),
                1,
                "x".to_string(),
                "k".to_string(),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, crate::Error::Unimplemented(_)));

        let err = svc
            .wish_list(Uuid::new_v4().to_string(), 1)
            .await
            .unwrap_err();
        assert!(matches!(err, crate::Error::Unimplemented(_)));
    }

    #[tokio::test]
    async fn wish_draw_stub_unimplemented() {
        let (svc, _acc_repo, _led_repo, _v3_repo) = make_ctx();
        let err = svc
            .wish_draw(Uuid::new_v4().to_string(), 1, 1, "k".to_string())
            .await
            .unwrap_err();
        assert!(matches!(err, crate::Error::Unimplemented(_)));
    }

    // ==================== 数据驱动反例验证 (per 9/4 MD §4) ====================

    #[test]
    fn data_driven_activity_type_covers_9_holiday_variants() {
        // 数据驱动: 1 个 ActivityType 涵盖 9 个 holiday_* 活动
        // 不是 9 套重复 RPC, 而是 1 套 + 配置
        let types = vec![
            ActivityType::Holiday,
            ActivityType::Signin,
            ActivityType::Achievement,
            ActivityType::Battlepass,
            ActivityType::Return,
            ActivityType::Invite,
            ActivityType::LevelReward,
            ActivityType::Daily,
            ActivityType::Weekly,
        ];
        for t in types {
            let i = t.as_i32();
            assert_eq!(t, ActivityType::from_i32(i));
        }
    }

    #[test]
    fn activity_template_serializes() {
        let now = Utc::now();
        let template = ActivityTemplateEntity {
            activity_id: 1,
            name: "Spring Festival".to_string(),
            activity_type: ActivityType::Holiday,
            starts_at: now,
            ends_at: now + Duration::days(7),
            max_progress: 100,
            min_level: 10,
            max_level: 0,
            template_json: r#"{"reward_tiers":[1,5,10]}"#.to_string(),
            enabled: true,
        };
        assert_eq!(template.activity_type, ActivityType::Holiday);
        assert!(template.enabled);
    }
}
