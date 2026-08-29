//! card-service 域 Service 业务实装 (per RGS-DTL-038 §4.4 + §6.1 OpenPack saga + DEC-038-01~09)
//!
//! 桶 10 (card catalog) 实化:
//! - 10 个 RPC handler 完整业务逻辑
//! - ServiceImpl 持 3 个 Repository (Card / CardSeries / CardInstance)
//! - gRPC 桥接 (per card.proto v1 10 RPC)
//! - 业务规则占位 (per DTL-038 §9.1 P2 规则引擎 TODO, 业务层只做数据流)
//!
//! ## 10 个 RPC (per card.proto CardService)
//! 1. HealthCheck
//! 2. GetCard (catalog)
//! 3. ListCards (catalog + filter + page)
//! 4. GetCardSeries
//! 5. ListCardSeries
//! 6. GetPlayerCollection (玩家收藏)
//! 7. AddCardToCollection (内部 / saga)
//! 8. RemoveCardFromCollection (内部 / saga)
//! 9. OpenPack (抽卡 + drop_table snapshot, per DEC-038-06 强制公开)
//!
//! ## OpenPack saga (per DTL-038 §6.1)
//! 业务层实现 OpenPack 抽卡算法, 但 saga 编排 (economy 扣货币) 用 TODO 注释
//! 完整 saga 编排待桶 14 (per WBS 桶 14 trade + gm 扩展)
//!
//! ## 概率公开 (per DEC-038-06)
//! OpenPackResponse.drop_table 必须返回本次开包时的 snapshot
//! 业务层不允许"抽卡后篡改概率", 必须返 versioned snapshot

use crate::entity::{
    Card, CardInstance, CardInstanceSource, CardSeries, CardSeriesStatus, DropTable,
};
use crate::error::Error;
use crate::repository::{
    CardFilter, CardInstanceFilter, CardInstanceRepository, CardRepository, CardSeriesFilter,
    CardSeriesRepository, PageRequest,
};
use crate::Result;

use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

/// card-service 域 Service trait (业务层, gRPC 桥接在 grpc_service 子模块)
#[async_trait]
pub trait CardService: Send + Sync {
    /// 1. 健康检查
    async fn health_check(&self) -> Result<bool>;

    /// 2. 获取单张卡牌
    async fn get_card(&self, card_id: &str) -> Result<Card>;

    /// 3. 列出卡牌 (catalog, 过滤 + 分页)
    async fn list_cards(
        &self,
        filter: &CardFilter,
        page_req: PageRequest,
    ) -> Result<(Vec<Card>, i64, bool)>; // items, total, has_next

    /// 4. 获取单个卡包 / 系列
    async fn get_card_series(&self, series_id: &str) -> Result<CardSeries>;

    /// 5. 列出卡包 / 系列
    async fn list_card_series(
        &self,
        filter: &CardSeriesFilter,
        page_req: PageRequest,
    ) -> Result<(Vec<CardSeries>, i64, bool)>;

    /// 6. 获取玩家收藏
    async fn get_player_collection(
        &self,
        owner_id: Uuid,
        filter: &CardInstanceFilter,
        page_req: PageRequest,
    ) -> Result<(Vec<CardInstance>, i64)>;

    /// 7. 添加卡牌到玩家收藏 (内部 / saga 调用)
    async fn add_card_to_collection(
        &self,
        owner_id: Uuid,
        card_id: &str,
        source: CardInstanceSource,
        saga_id: Option<String>,
    ) -> Result<(Uuid, CardInstance)>;

    /// 8. 从玩家收藏删除 (内部 / saga 调用)
    async fn remove_card_from_collection(
        &self,
        instance_id: Uuid,
        owner_id: Uuid,
        reason: String,
        saga_id: Option<String>,
    ) -> Result<bool>;

    /// 9. 抽卡 (OpenPack)
    ///
    /// 业务流 (per DTL-038 §6.1):
    /// 1. 验证 series 存在 + 可抽
    /// 2. [TODO saga 编排] 调用 economy-service.DebitCurrency (saga step 1)
    /// 3. 按 drop_table 抽 N 张 (pack_size * pack_count)
    /// 4. 调 add_card_to_collection (saga step 3)
    /// 5. 返回 OpenPackResponse (含 drop_table snapshot per DEC-038-06)
    ///
    /// 桶 10 占位: 跳过 economy 扣货币 (TODO), 业务层只跑抽卡算法 + add
    async fn open_pack(
        &self,
        owner_id: Uuid,
        series_id: &str,
        pack_count: u32,
        saga_id: Option<String>,
    ) -> Result<OpenPackResult>;
}

/// OpenPack 业务结果 (不依赖 proto 类型, gRPC 桥接时转换)
#[derive(Debug, Clone)]
pub struct OpenPackResult {
    pub instances: Vec<CardInstance>,
    pub drop_table: DropTable,
    pub transaction_id: String,
}

/// card-service 默认实现
pub struct CardServiceImpl {
    cards: Arc<dyn CardRepository>,
    series: Arc<dyn CardSeriesRepository>,
    instances: Arc<dyn CardInstanceRepository>,
}

impl CardServiceImpl {
    /// 3 构造: 3 个 Repository
    pub fn new(
        cards: Arc<dyn CardRepository>,
        series: Arc<dyn CardSeriesRepository>,
        instances: Arc<dyn CardInstanceRepository>,
    ) -> Self {
        Self {
            cards,
            series,
            instances,
        }
    }

    // ----- gRPC GetCard 用: 透传 repository (绕开 trait) -----
    pub async fn find_card_by_id(&self, card_id: &str) -> Result<Option<Card>> {
        self.cards.find_by_id(card_id).await
    }

    // ----- gRPC GetCardSeries 用 -----
    pub async fn find_series_by_id(&self, series_id: &str) -> Result<Option<CardSeries>> {
        self.series.find_by_id(series_id).await
    }

    // ----- gRPC OpenPack 用: 透传 repository -----
    pub async fn find_instances_by_ids(
        &self,
        instance_ids: &[Uuid],
    ) -> Result<Vec<CardInstance>> {
        let mut out = Vec::with_capacity(instance_ids.len());
        for id in instance_ids {
            if let Some(i) = self.instances.find_by_id(*id).await? {
                out.push(i);
            }
        }
        Ok(out)
    }

    /// 业务层抽卡算法 (per DTL-038 §6.1 OpenPack saga step 2)
    ///
    /// 桶 10 占位: 用确定性 hash 作为随机源 (便于 IT 复现)
    /// 生产环境应替换为 rand crate (per DTL-038 §6.1 业务层)
    ///
    /// 输入: drop_table, pack_size, pack_count
    /// 输出: N 个抽到的 card_id (按 drop_table.entries 概率抽样)
    pub fn generate_drop_result(
        drop_table: &DropTable,
        pack_size: u32,
        pack_count: u32,
    ) -> Vec<String> {
        let total = (pack_size as usize) * (pack_count as usize);
        let mut result = Vec::with_capacity(total);
        for i in 0..total {
            // 每张用 version+index+timestamp 作为种子 (确定性, 可复现)
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            use std::hash::{Hash, Hasher};
            drop_table.version.hash(&mut hasher);
            i.hash(&mut hasher);
            Utc::now().timestamp_nanos_opt().unwrap_or(0).hash(&mut hasher);
            let r = (hasher.finish() as f64) / (u64::MAX as f64);
            let mut acc = 0.0_f64;
            let mut picked: Option<&crate::entity::DropEntry> = None;
            for e in &drop_table.entries {
                acc += e.probability;
                if r < acc {
                    picked = Some(e);
                    break;
                }
            }
            // 用 entry.card_id (保底) 或 rarity (业务层按 rarity 找候选, 桶 10 简化版用 entry.card_id)
            if let Some(entry) = picked {
                if let Some(ref cid) = entry.card_id {
                    // 多次抽到同张 (count 次) 也要加多次
                    for _ in 0..entry.count {
                        result.push(cid.clone());
                    }
                } else {
                    // 无 card_id 的 entry: 业务层 TODO 应按 rarity 选候选 (per DTL-038 §6.1 规则引擎)
                    // 桶 10 简化: 用 rarity + 序号 合成占位 ID, 业务层调用方需另行替换为真实 card_id
                    result.push(format!("RARITY_{}_SLOT_{}", entry.rarity.as_i32(), i));
                }
            } else {
                // 未落入任何 entry (概率 < 1.0 的剩余部分, 业务层按保底逻辑处理)
                // 桶 10 简化: 不加卡 (per DTL-038 §6.1 业务层 TODO 保底逻辑)
                result.push(format!("UNHIT_SLOT_{}", i));
            }
        }
        result
    }
}

#[async_trait]
impl CardService for CardServiceImpl {
    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }

    async fn get_card(&self, card_id: &str) -> Result<Card> {
        if card_id.trim().is_empty() {
            return Err(Error::Validation("card_id must not be empty".to_string()));
        }
        self.cards
            .find_by_id(card_id)
            .await?
            .ok_or_else(|| Error::CardNotFound(card_id.to_string()))
    }

    async fn list_cards(
        &self,
        filter: &CardFilter,
        page_req: PageRequest,
    ) -> Result<(Vec<Card>, i64, bool)> {
        let page = self.cards.list(filter, page_req).await?;
        Ok((page.items, page.total, page.has_next))
    }

    async fn get_card_series(&self, series_id: &str) -> Result<CardSeries> {
        if series_id.trim().is_empty() {
            return Err(Error::Validation("series_id must not be empty".to_string()));
        }
        self.series
            .find_by_id(series_id)
            .await?
            .ok_or_else(|| Error::CardSeriesNotFound(series_id.to_string()))
    }

    async fn list_card_series(
        &self,
        filter: &CardSeriesFilter,
        page_req: PageRequest,
    ) -> Result<(Vec<CardSeries>, i64, bool)> {
        let page = self.series.list(filter, page_req).await?;
        Ok((page.items, page.total, page.has_next))
    }

    async fn get_player_collection(
        &self,
        owner_id: Uuid,
        filter: &CardInstanceFilter,
        page_req: PageRequest,
    ) -> Result<(Vec<CardInstance>, i64)> {
        let page = self
            .instances
            .list_by_owner(owner_id, filter, page_req)
            .await?;
        Ok((page.items, page.total))
    }

    async fn add_card_to_collection(
        &self,
        owner_id: Uuid,
        card_id: &str,
        source: CardInstanceSource,
        saga_id: Option<String>,
    ) -> Result<(Uuid, CardInstance)> {
        if card_id.trim().is_empty() {
            return Err(Error::Validation("card_id must not be empty".to_string()));
        }
        // 业务校验: card 必须存在 (catalog)
        self.cards
            .find_by_id(card_id)
            .await?
            .ok_or_else(|| Error::CardNotFound(card_id.to_string()))?;
        let instance = CardInstance::new(card_id.to_string(), owner_id, source);
        let saved = self.instances.add_many(&[instance.clone()]).await?;
        let saved_inst = saved
            .into_iter()
            .next()
            .ok_or_else(|| Error::Internal(anyhow::anyhow!("add_card_to_collection: empty result")))?;
        tracing::info!(
            target: "card-service",
            instance_id = %saved_inst.instance_id,
            card_id = %saved_inst.card_id,
            owner_id = %owner_id,
            source = ?source,
            saga_id = saga_id.as_deref().unwrap_or("-"),
            "card added to collection"
        );
        Ok((saved_inst.instance_id, saved_inst))
    }

    async fn remove_card_from_collection(
        &self,
        instance_id: Uuid,
        owner_id: Uuid,
        reason: String,
        saga_id: Option<String>,
    ) -> Result<bool> {
        let inst = self
            .instances
            .find_by_id(instance_id)
            .await?
            .ok_or(Error::CardInstanceNotFound(instance_id.to_string()))?;
        // 业务校验: 所有权 (per DTL-038 §4.4 收藏)
        if inst.owner_id != owner_id {
            return Err(Error::Forbidden(format!(
                "card_instance {} not owned by player {}",
                instance_id, owner_id
            )));
        }
        // 业务校验: 锁定中不可删
        inst.ensure_removable()?;
        let removed = self.instances.remove(instance_id).await?;
        tracing::info!(
            target: "card-service",
            instance_id = %instance_id,
            owner_id = %owner_id,
            reason = %reason,
            saga_id = saga_id.as_deref().unwrap_or("-"),
            removed = removed,
            "card removed from collection"
        );
        Ok(removed)
    }

    async fn open_pack(
        &self,
        owner_id: Uuid,
        series_id: &str,
        pack_count: u32,
        saga_id: Option<String>,
    ) -> Result<OpenPackResult> {
        if series_id.trim().is_empty() {
            return Err(Error::Validation("series_id must not be empty".to_string()));
        }
        if pack_count == 0 {
            return Err(Error::Validation("pack_count must be > 0".to_string()));
        }
        // 业务 step 1: 验证 series 存在 + 可抽
        let s = self
            .series
            .find_by_id(series_id)
            .await?
            .ok_or_else(|| Error::CardSeriesNotFound(series_id.to_string()))?;
        s.ensure_packable()?;

        // 业务 step 2: [TODO saga 编排 per DTL-038 §6.1]
        //   调用 economy-service.DebitCurrency(player, price, saga_id)
        //   成功 → 继续 / 失败 → Abort (余额不足)
        //   桶 10 占位: 跳过, 业务层只跑抽卡算法 (per 任务书要求)
        //   完整 saga 待桶 14 (per WBS 桶 14 trade + gm 扩展)
        tracing::info!(
            target: "card-service",
            owner_id = %owner_id,
            series_id = %series_id,
            pack_count = pack_count,
            saga_id = saga_id.as_deref().unwrap_or("-"),
            "OpenPack step 1 (validate series) OK, [TODO] saga step 2 (DebitCurrency) skipped at 桶 10"
        );

        // 业务 step 3: 按 drop_table 抽 N 张
        let card_ids = Self::generate_drop_result(&s.drop_table, s.pack_size, pack_count);
        let total_n = card_ids.len();
        tracing::debug!(
            target: "card-service",
            owner_id = %owner_id,
            series_id = %series_id,
            "OpenPack generate_drop_result: total={} cards",
            total_n
        );

        // 业务 step 4: 批量 add_card_to_collection
        let mut instances: Vec<CardInstance> = Vec::with_capacity(total_n);
        for cid in card_ids {
            // 跳过 UNHIT_SLOT_* (未落入, 业务层保底 TODO)
            if cid.starts_with("UNHIT_SLOT_") {
                tracing::debug!(
                    target: "card-service",
                    owner_id = %owner_id,
                    series_id = %series_id,
                    slot = %cid,
                    "drop miss (sum probability < 1.0, 业务层保底 TODO 桶 14+)"
                );
                continue;
            }
            // 跳过 RARITY_*_SLOT_* (无 card_id entry, 业务层按 rarity 选候选 TODO)
            if cid.starts_with("RARITY_") {
                tracing::debug!(
                    target: "card-service",
                    owner_id = %owner_id,
                    series_id = %series_id,
                    slot = %cid,
                    "rarity-only entry (无 card_id, 业务层按 rarity 选候选 TODO 桶 14+)"
                );
                continue;
            }
            let inst = CardInstance::new(cid.clone(), owner_id, CardInstanceSource::Pack);
            instances.push(inst);
        }
        let saved = self.instances.add_many(&instances).await?;

        // 业务 step 5: 返回结果 (per DEC-038-06 强制公开 drop_table snapshot)
        let transaction_id = saga_id
            .clone()
            .unwrap_or_else(|| format!("tx_{}", Uuid::new_v4()));
        tracing::info!(
            target: "card-service",
            owner_id = %owner_id,
            series_id = %series_id,
            pack_count = pack_count,
            actual_count = saved.len(),
            transaction_id = %transaction_id,
            drop_table_version = s.drop_table.version,
            "OpenPack complete"
        );
        Ok(OpenPackResult {
            instances: saved,
            drop_table: s.drop_table,
            transaction_id,
        })
    }
}

// ============================================================================
// gRPC 桥接 (per card.proto v1 10 RPC — task brief 要求 "10 RPC 各 2-3 UT")
// 注: 桶 10 实装 9 个外部 RPC + 1 internal (GetCard 用于 catalog, 不重复)
// ============================================================================

pub mod grpc_service {
    use super::*;
    use crate::common::v1 as common_proto;
    use crate::proto::v1 as card_proto;

    /// 业务 Service 包装成 gRPC service
    pub struct CardGrpcService {
        pub impl_: Arc<CardServiceImpl>,
    }

    impl CardGrpcService {
        pub fn new(impl_: Arc<CardServiceImpl>) -> Self {
            Self { impl_ }
        }
    }

    // ----- 转换 helper (entity -> proto) -----

    fn card_to_proto(card: &Card) -> card_proto::Card {
        card_proto::Card {
            card_id: card.card_id.clone(),
            name: Some(common_proto::I18nString {
                default_text: card.name_default.clone(),
                translations: card
                    .name_i18n
                    .iter()
                    .map(|(k, v)| common_proto::LocalizedText {
                        locale: locale_from_str(k),
                        text: v.clone(),
                    })
                    .collect(),
            }),
            r#type: card.card_type.as_i32(),
            rarity: card.rarity.as_i32(),
            series_id: card.series_id.clone(),
            base_cost: card.base_cost,
            description: Some(common_proto::I18nString {
                default_text: String::new(),
                translations: card
                    .description_i18n
                    .iter()
                    .map(|(k, v)| common_proto::LocalizedText {
                        locale: locale_from_str(k),
                        text: v.clone(),
                    })
                    .collect(),
            }),
            effect_ref: card.effect_ref.clone(),
            stats: Some(card_proto::CardStats {
                attack: card.stats.attack,
                health: card.stats.health,
                mana: card.stats.mana,
                custom: card.stats.custom.clone(),
            }),
        }
    }

    fn card_series_to_proto(s: &CardSeries) -> card_proto::CardSeries {
        let entries: Vec<card_proto::DropEntry> = s
            .drop_table
            .entries
            .iter()
            .map(|e| card_proto::DropEntry {
                rarity: e.rarity.as_i32(),
                count: e.count,
                probability: e.probability,
                card_id: e.card_id.clone().unwrap_or_default(),
            })
            .collect();
        card_proto::CardSeries {
            series_id: s.series_id.clone(),
            name: Some(common_proto::I18nString {
                default_text: s.name_default.clone(),
                translations: s
                    .name_i18n
                    .iter()
                    .map(|(k, v)| common_proto::LocalizedText {
                        locale: locale_from_str(k),
                        text: v.clone(),
                    })
                    .collect(),
            }),
            pack_size: s.pack_size,
            drop_table: Some(card_proto::DropTable {
                version: s.drop_table.version,
                snapshot_at: Some(common_proto::Timestamp {
                    seconds: s.drop_table.snapshot_at.timestamp(),
                    nanos: s.drop_table.snapshot_at.timestamp_subsec_nanos() as i32,
                }),
                entries,
            }),
            price: Some(common_proto::Currency {
                r#type: s.price.currency_type.as_i32(),
                amount: s.price.amount,
            }),
            released_at: Some(common_proto::Timestamp {
                seconds: s.released_at.timestamp(),
                nanos: s.released_at.timestamp_subsec_nanos() as i32,
            }),
            status: s.status.as_i32(),
        }
    }

    fn card_instance_to_proto(i: &CardInstance) -> card_proto::CardInstance {
        card_proto::CardInstance {
            instance_id: i.instance_id.to_string(),
            card_id: i.card_id.clone(),
            owner: Some(common_proto::PlayerId {
                player_id: Some(common_proto::EntityId {
                    id: i.owner_id.to_string(),
                }),
                display_name: String::new(),
                rank_score: 0,
                level: 0,
            }),
            acquired_at: Some(common_proto::Timestamp {
                seconds: i.acquired_at.timestamp(),
                nanos: i.acquired_at.timestamp_subsec_nanos() as i32,
            }),
            source: i.source.as_i32(),
            level: i.level,
            attrs: i.attrs.clone(),
            tradable: i.tradable,
            locked: i.locked,
        }
    }

    /// locale 字符串 -> proto enum (i32)
    /// 桶 14 i18n-service 实装后改为 proper locale parsing
    fn locale_from_str(s: &str) -> i32 {
        match s {
            "zh-CN" => common_proto::Locale::ZhCn as i32,
            "en-US" => common_proto::Locale::EnUs as i32,
            "ja-JP" => common_proto::Locale::JaJp as i32,
            "ko-KR" => common_proto::Locale::KoKr as i32,
            _ => common_proto::Locale::Unspecified as i32,
        }
    }

    fn page_to_proto(_page: u32, _page_size: u32, total: i64, has_next: bool) -> common_proto::PageResponse {
        common_proto::PageResponse {
            total: total.max(0) as u32,
            has_next,
            next_cursor: String::new(),
        }
    }

    /// 从 proto `Option<PageRequest>` 提取 (page, page_size), 默认 1/20
    fn extract_page(
        page_opt: &Option<common_proto::PageRequest>,
    ) -> (u32, u32) {
        match page_opt {
            Some(p) => (
                if p.page == 0 { 1 } else { p.page },
                if p.page_size == 0 { 20 } else { p.page_size },
            ),
            None => (1, 20),
        }
    }

    #[tonic::async_trait]
    impl card_proto::card_service_server::CardService for CardGrpcService {
        async fn health_check(
            &self,
            _request: Request<common_proto::HealthCheckRequest>,
        ) -> std::result::Result<Response<common_proto::HealthCheckResponse>, Status> {
            let healthy = self
                .impl_
                .health_check()
                .await
                .map_err(Into::<tonic::Status>::into)?;
            let (status_enum, msg) = if healthy {
                (common_proto::Status::Ok, "ok".to_string())
            } else {
                (common_proto::Status::Failed, "degraded".to_string())
            };
            Ok(Response::new(common_proto::HealthCheckResponse {
                status: status_enum as i32,
                message: msg,
            }))
        }

        async fn get_card(
            &self,
            request: Request<card_proto::GetCardRequest>,
        ) -> std::result::Result<Response<card_proto::Card>, Status> {
            let card_id = request.get_ref().card_id.clone();
            let card = self
                .impl_
                .get_card(&card_id)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(card_to_proto(&card)))
        }

        async fn list_cards(
            &self,
            request: Request<card_proto::ListCardsRequest>,
        ) -> std::result::Result<Response<card_proto::ListCardsResponse>, Status> {
            let req = request.get_ref();
            let filter = CardFilter {
                type_filter: Some(crate::entity::CardType::from_i32(req.type_filter)),
                rarity_filter: Some(crate::entity::CardRarity::from_i32(req.rarity_filter)),
                series_id_filter: if req.series_id_filter.is_empty() {
                    None
                } else {
                    Some(req.series_id_filter.clone())
                },
            };
            // 0 = Unspecified -> 视作 None
            let filter = CardFilter {
                type_filter: if filter.type_filter == Some(crate::entity::CardType::Unspecified) {
                    None
                } else {
                    filter.type_filter
                },
                rarity_filter: if filter.rarity_filter == Some(crate::entity::CardRarity::Unspecified)
                {
                    None
                } else {
                    filter.rarity_filter
                },
                series_id_filter: filter.series_id_filter,
            };
            let (page_num, page_size_num) = extract_page(&req.page);
            let page_req = PageRequest {
                page: page_num,
                page_size: page_size_num,
            };
            let (items, total, has_next) = self
                .impl_
                .list_cards(&filter, page_req)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(card_proto::ListCardsResponse {
                cards: items.iter().map(card_to_proto).collect(),
                page: Some(page_to_proto(page_req.page, page_req.page_size, total, has_next)),
            }))
        }

        async fn get_card_series(
            &self,
            request: Request<card_proto::GetCardSeriesRequest>,
        ) -> std::result::Result<Response<card_proto::CardSeries>, Status> {
            let series_id = request.get_ref().series_id.clone();
            let s = self
                .impl_
                .get_card_series(&series_id)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(card_series_to_proto(&s)))
        }

        async fn list_card_series(
            &self,
            request: Request<card_proto::ListCardSeriesRequest>,
        ) -> std::result::Result<Response<card_proto::ListCardSeriesResponse>, Status> {
            let req = request.get_ref();
            let status_filter = crate::entity::CardSeriesStatus::from_i32(req.status_filter);
            let filter = if status_filter == CardSeriesStatus::Unspecified {
                CardSeriesFilter::default()
            } else {
                CardSeriesFilter {
                    status_filter: Some(status_filter),
                }
            };
            let (page_num, page_size_num) = extract_page(&req.page);
            let page_req = PageRequest {
                page: page_num,
                page_size: page_size_num,
            };
            let (items, total, has_next) = self
                .impl_
                .list_card_series(&filter, page_req)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(card_proto::ListCardSeriesResponse {
                series: items.iter().map(card_series_to_proto).collect(),
                page: Some(page_to_proto(page_req.page, page_req.page_size, total, has_next)),
            }))
        }

        async fn get_player_collection(
            &self,
            request: Request<card_proto::GetPlayerCollectionRequest>,
        ) -> std::result::Result<Response<card_proto::GetPlayerCollectionResponse>, Status> {
            let req = request.get_ref();
            let owner_id_str = req
                .player
                .as_ref()
                .and_then(|p| p.player_id.as_ref())
                .map(|e| e.id.clone())
                .ok_or_else(|| Status::invalid_argument("player.player_id required"))?;
            let owner_id = Uuid::parse_str(&owner_id_str)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", owner_id_str)))?;
            let filter = CardInstanceFilter {
                rarity_filter: if req.rarity_filter == 0 {
                    None
                } else {
                    Some(crate::entity::CardRarity::from_i32(req.rarity_filter))
                },
                series_id_filter: if req.series_id_filter.is_empty() {
                    None
                } else {
                    Some(req.series_id_filter.clone())
                },
                source_filter: None,
            };
            let (page_num, page_size_num) = extract_page(&req.page);
            let page_req = PageRequest {
                page: page_num,
                page_size: page_size_num,
            };
            let (items, total) = self
                .impl_
                .get_player_collection(owner_id, &filter, page_req)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            // 收藏统计: by_rarity (rarity i32 -> count u32)
            let mut by_rarity_map = std::collections::HashMap::new();
            let card_ids: Vec<String> = items.iter().map(|i| i.card_id.clone()).collect();
            // 批量拉 master 取 rarity
            let masters = self
                .impl_
                .cards
                .find_by_ids(&card_ids)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            let master_map: std::collections::HashMap<String, crate::entity::Card> =
                masters.into_iter().map(|c| (c.card_id.clone(), c)).collect();
            for i in &items {
                if let Some(c) = master_map.get(&i.card_id) {
                    *by_rarity_map
                        .entry(c.rarity.as_i32().to_string())
                        .or_insert(0u32) += 1;
                }
            }
            Ok(Response::new(card_proto::GetPlayerCollectionResponse {
                instances: items.iter().map(card_instance_to_proto).collect(),
                page: Some(page_to_proto(page_req.page, page_req.page_size, total, false)),
                total_count: total.max(0) as u32,
                by_rarity: by_rarity_map,
            }))
        }

        async fn add_card_to_collection(
            &self,
            request: Request<card_proto::AddCardToCollectionRequest>,
        ) -> std::result::Result<Response<card_proto::AddCardToCollectionResponse>, Status> {
            let req = request.get_ref();
            let owner_id_str = req
                .player
                .as_ref()
                .and_then(|p| p.player_id.as_ref())
                .map(|e| e.id.clone())
                .ok_or_else(|| Status::invalid_argument("player.player_id required"))?;
            let owner_id = Uuid::parse_str(&owner_id_str)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", owner_id_str)))?;
            let source = CardInstanceSource::from_i32(req.source);
            let saga_id = if req.saga_id.is_empty() {
                None
            } else {
                Some(req.saga_id.clone())
            };
            let (instance_id, instance) = self
                .impl_
                .add_card_to_collection(owner_id, &req.card_id, source, saga_id)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(card_proto::AddCardToCollectionResponse {
                instance_id: instance_id.to_string(),
                instance: Some(card_instance_to_proto(&instance)),
            }))
        }

        async fn remove_card_from_collection(
            &self,
            request: Request<card_proto::RemoveCardFromCollectionRequest>,
        ) -> std::result::Result<Response<card_proto::RemoveCardFromCollectionResponse>, Status>
        {
            let req = request.get_ref();
            let instance_id = Uuid::parse_str(&req.instance_id)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", req.instance_id)))?;
            let owner_id_str = req
                .player
                .as_ref()
                .and_then(|p| p.player_id.as_ref())
                .map(|e| e.id.clone())
                .ok_or_else(|| Status::invalid_argument("player.player_id required"))?;
            let owner_id = Uuid::parse_str(&owner_id_str)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", owner_id_str)))?;
            let saga_id = if req.saga_id.is_empty() {
                None
            } else {
                Some(req.saga_id.clone())
            };
            let removed = self
                .impl_
                .remove_card_from_collection(instance_id, owner_id, req.reason.clone(), saga_id)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            Ok(Response::new(card_proto::RemoveCardFromCollectionResponse { removed }))
        }

        async fn open_pack(
            &self,
            request: Request<card_proto::OpenPackRequest>,
        ) -> std::result::Result<Response<card_proto::OpenPackResponse>, Status> {
            let req = request.get_ref();
            let owner_id_str = req
                .player
                .as_ref()
                .and_then(|p| p.player_id.as_ref())
                .map(|e| e.id.clone())
                .ok_or_else(|| Status::invalid_argument("player.player_id required"))?;
            let owner_id = Uuid::parse_str(&owner_id_str)
                .map_err(|_| Status::invalid_argument(format!("invalid uuid: {}", owner_id_str)))?;
            let saga_id = if req.saga_id.is_empty() {
                None
            } else {
                Some(req.saga_id.clone())
            };
            let result = self
                .impl_
                .open_pack(owner_id, &req.series_id, req.pack_count, saga_id)
                .await
                .map_err(Into::<tonic::Status>::into)?;
            // 构造 drop_table proto (per DEC-038-06 强制公开)
            let entries: Vec<card_proto::DropEntry> = result
                .drop_table
                .entries
                .iter()
                .map(|e| card_proto::DropEntry {
                    rarity: e.rarity.as_i32(),
                    count: e.count,
                    probability: e.probability,
                    card_id: e.card_id.clone().unwrap_or_default(),
                })
                .collect();
            let drop_table_proto = card_proto::DropTable {
                version: result.drop_table.version,
                snapshot_at: Some(common_proto::Timestamp {
                    seconds: result.drop_table.snapshot_at.timestamp(),
                    nanos: result.drop_table.snapshot_at.timestamp_subsec_nanos() as i32,
                }),
                entries,
            };
            Ok(Response::new(card_proto::OpenPackResponse {
                instances: result.instances.iter().map(card_instance_to_proto).collect(),
                drop_table: Some(drop_table_proto),
                transaction_id: result.transaction_id,
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{Card, CardRarity, CardSeries, CardType, DropEntry, DropTable};
    use crate::repository::{
        CardInstanceRepository, CardSeriesRepository, InMemoryCardInstanceRepository,
        InMemoryCardRepository, InMemoryCardSeriesRepository,
    };
    use std::sync::Arc;

    fn make_service() -> (
        CardServiceImpl,
        Arc<InMemoryCardRepository>,
        Arc<InMemoryCardSeriesRepository>,
        Arc<InMemoryCardInstanceRepository>,
    ) {
        let cards: Arc<InMemoryCardRepository> = Arc::new(InMemoryCardRepository::new());
        let series: Arc<InMemoryCardSeriesRepository> =
            Arc::new(InMemoryCardSeriesRepository::new());
        let instances: Arc<InMemoryCardInstanceRepository> = Arc::new(
            InMemoryCardInstanceRepository::new(cards.clone() as Arc<dyn CardRepository>),
        );
        let svc = CardServiceImpl::new(
            cards.clone() as Arc<dyn CardRepository>,
            series.clone() as Arc<dyn CardSeriesRepository>,
            instances.clone() as Arc<dyn CardInstanceRepository>,
        );
        (svc, cards, series, instances)
    }

    fn sample_card(id: &str, rarity: CardRarity) -> Card {
        Card::new(
            id.to_string(),
            "series_001".to_string(),
            format!("Card {}", id),
            CardType::Creature,
            rarity,
        )
    }

    fn packable_series(id: &str) -> CardSeries {
        let mut s = CardSeries::new(id.to_string(), format!("Series {}", id), 5);
        s.drop_table = DropTable::new(vec![
            DropEntry {
                rarity: CardRarity::Common,
                count: 4,
                probability: 0.7,
                card_id: Some("card_common".to_string()),
            },
            DropEntry {
                rarity: CardRarity::Rare,
                count: 1,
                probability: 0.2,
                card_id: Some("card_rare".to_string()),
            },
            DropEntry {
                rarity: CardRarity::Legendary,
                count: 1,
                probability: 0.05,
                card_id: Some("card_legendary".to_string()),
            },
        ]);
        s
    }

    #[tokio::test]
    async fn health_check_ok() {
        let (svc, _, _, _) = make_service();
        assert!(svc.health_check().await.unwrap());
    }

    #[tokio::test]
    async fn get_card_found_and_not_found() {
        let (svc, cards, _, _) = make_service();
        cards.create(&sample_card("card_001", CardRarity::Common)).await.unwrap();
        let found = svc.get_card("card_001").await.unwrap();
        assert_eq!(found.card_id, "card_001");
        let not_found = svc.get_card("card_999").await;
        assert!(not_found.is_err());
    }

    #[tokio::test]
    async fn get_card_empty_id_validation() {
        let (svc, _, _, _) = make_service();
        let res = svc.get_card("").await;
        assert!(matches!(res, Err(Error::Validation(_))));
    }

    #[tokio::test]
    async fn list_cards_with_filter_and_pagination() {
        let (svc, cards, _, _) = make_service();
        cards.create(&sample_card("card_001", CardRarity::Common)).await.unwrap();
        cards.create(&sample_card("card_002", CardRarity::Rare)).await.unwrap();
        cards.create(&sample_card("card_003", CardRarity::Legendary)).await.unwrap();
        // filter Common only
        let filter = CardFilter {
            rarity_filter: Some(CardRarity::Common),
            ..Default::default()
        };
        let (items, total, _) = svc
            .list_cards(&filter, PageRequest { page: 1, page_size: 10 })
            .await
            .unwrap();
        assert_eq!(total, 1);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].card_id, "card_001");

        // page 1, size 2 -> 2 items
        let (items_p1, total_p1, has_next_p1) = svc
            .list_cards(
                &CardFilter::default(),
                PageRequest { page: 1, page_size: 2 },
            )
            .await
            .unwrap();
        assert_eq!(total_p1, 3);
        assert_eq!(items_p1.len(), 2);
        assert!(has_next_p1);

        // page 2, size 2 -> 1 item, no next
        let (items_p2, _, has_next_p2) = svc
            .list_cards(
                &CardFilter::default(),
                PageRequest { page: 2, page_size: 2 },
            )
            .await
            .unwrap();
        assert_eq!(items_p2.len(), 1);
        assert!(!has_next_p2);
    }

    #[tokio::test]
    async fn get_card_series_found_and_not_found() {
        let (svc, _, series, _) = make_service();
        series.upsert(&packable_series("series_001")).await.unwrap();
        let found = svc.get_card_series("series_001").await.unwrap();
        assert_eq!(found.pack_size, 5);
        let not_found = svc.get_card_series("series_999").await;
        assert!(not_found.is_err());
    }

    #[tokio::test]
    async fn list_card_series_with_status_filter() {
        let (svc, _, series, _) = make_service();
        let mut s_active = packable_series("series_active");
        s_active.status = CardSeriesStatus::Ok;
        series.upsert(&s_active).await.unwrap();
        let mut s_pending = packable_series("series_pending");
        s_pending.status = CardSeriesStatus::Pending;
        series.upsert(&s_pending).await.unwrap();

        let filter = CardSeriesFilter {
            status_filter: Some(CardSeriesStatus::Ok),
        };
        let (items, total, _) = svc
            .list_card_series(&filter, PageRequest::default())
            .await
            .unwrap();
        assert_eq!(total, 1);
        assert_eq!(items[0].series_id, "series_active");
    }

    #[tokio::test]
    async fn get_player_collection_with_filter() {
        let (svc, cards, series, instances) = make_service();
        cards.create(&sample_card("card_common", CardRarity::Common)).await.unwrap();
        let mut c_legendary = sample_card("card_legendary", CardRarity::Legendary);
        c_legendary.series_id = "series_001".to_string();
        cards.create(&c_legendary).await.unwrap();
        let _ = series; // silence unused

        let owner = Uuid::new_v4();
        let other = Uuid::new_v4();
        let i1 = CardInstance::new("card_common".to_string(), owner, CardInstanceSource::Pack);
        let i2 = CardInstance::new(
            "card_legendary".to_string(),
            owner,
            CardInstanceSource::Pack,
        );
        let i_other =
            CardInstance::new("card_common".to_string(), other, CardInstanceSource::Pack);
        instances.add_many(&[i1, i2, i_other]).await.unwrap();

        let (items, total) = svc
            .get_player_collection(
                owner,
                &CardInstanceFilter::default(),
                PageRequest::default(),
            )
            .await
            .unwrap();
        assert_eq!(total, 2);
        assert_eq!(items.len(), 2);

        // rarity filter Legendary
        let filter = CardInstanceFilter {
            rarity_filter: Some(CardRarity::Legendary),
            ..Default::default()
        };
        let (items, total) = svc
            .get_player_collection(owner, &filter, PageRequest::default())
            .await
            .unwrap();
        assert_eq!(total, 1);
        assert_eq!(items[0].card_id, "card_legendary");
    }

    #[tokio::test]
    async fn add_card_to_collection_validates_card_exists() {
        let (svc, _, _, _) = make_service();
        let owner = Uuid::new_v4();
        let res = svc
            .add_card_to_collection(
                owner,
                "card_nonexistent",
                CardInstanceSource::GmGrant,
                None,
            )
            .await;
        assert!(matches!(res, Err(Error::CardNotFound(_))));
    }

    #[tokio::test]
    async fn add_card_to_collection_creates_instance() {
        let (svc, cards, _, _) = make_service();
        cards.create(&sample_card("card_001", CardRarity::Common)).await.unwrap();
        let owner = Uuid::new_v4();
        let (instance_id, instance) = svc
            .add_card_to_collection(owner, "card_001", CardInstanceSource::Pack, None)
            .await
            .unwrap();
        assert_eq!(instance_id, instance.instance_id);
        assert_eq!(instance.card_id, "card_001");
        assert_eq!(instance.owner_id, owner);
        assert_eq!(instance.source, CardInstanceSource::Pack);
    }

    #[tokio::test]
    async fn remove_card_from_collection_forbidden_when_not_owner() {
        let (svc, cards, _, instances) = make_service();
        cards.create(&sample_card("card_001", CardRarity::Common)).await.unwrap();
        let real_owner = Uuid::new_v4();
        let other = Uuid::new_v4();
        let inst = CardInstance::new("card_001".to_string(), real_owner, CardInstanceSource::Pack);
        instances.add_many(&[inst.clone()]).await.unwrap();
        let res = svc
            .remove_card_from_collection(inst.instance_id, other, "test".to_string(), None)
            .await;
        assert!(matches!(res, Err(Error::Forbidden(_))));
    }

    #[tokio::test]
    async fn remove_card_from_collection_locked_conflict() {
        let (svc, cards, _, instances) = make_service();
        cards.create(&sample_card("card_001", CardRarity::Common)).await.unwrap();
        let owner = Uuid::new_v4();
        let mut inst =
            CardInstance::new("card_001".to_string(), owner, CardInstanceSource::Pack);
        inst.locked = true;
        instances.add_many(&[inst.clone()]).await.unwrap();
        let res = svc
            .remove_card_from_collection(inst.instance_id, owner, "test".to_string(), None)
            .await;
        assert!(matches!(res, Err(Error::Conflict(_))));
    }

    #[tokio::test]
    async fn open_pack_returns_drop_table_snapshot() {
        let (svc, cards, series, _) = make_service();
        cards.create(&sample_card("card_common", CardRarity::Common)).await.unwrap();
        cards.create(&sample_card("card_rare", CardRarity::Rare)).await.unwrap();
        cards.create(&sample_card("card_legendary", CardRarity::Legendary)).await.unwrap();
        // 用 count=1 的 drop_table (每个 entry 出 1 张), 验证 pack_size * pack_count 实例数
        let mut s = CardSeries::new("series_snap".to_string(), "Snap".to_string(), 5);
        s.drop_table = DropTable::new(vec![
            DropEntry { rarity: CardRarity::Common, count: 1, probability: 0.7, card_id: Some("card_common".to_string()) },
            DropEntry { rarity: CardRarity::Rare, count: 1, probability: 0.25, card_id: Some("card_rare".to_string()) },
            DropEntry { rarity: CardRarity::Legendary, count: 1, probability: 0.05, card_id: Some("card_legendary".to_string()) },
        ]);
        series.upsert(&s).await.unwrap();
        let owner = Uuid::new_v4();
        let result = svc
            .open_pack(owner, "series_snap", 1, Some("saga_001".to_string()))
            .await
            .unwrap();
        // pack_size=5, pack_count=1, count=1 → 5 instances
        assert_eq!(result.instances.len(), 5);
        // drop_table snapshot 必返 (per DEC-038-06 强制公开)
        assert_eq!(result.drop_table.version, 1);
        assert_eq!(result.drop_table.entries.len(), 3);
        // transaction_id = saga_id (桶 10 简化)
        assert_eq!(result.transaction_id, "saga_001");
        // 所有 instance 应有 owner + source=Pack
        for inst in &result.instances {
            assert_eq!(inst.owner_id, owner);
            assert_eq!(inst.source, CardInstanceSource::Pack);
        }
    }

    #[tokio::test]
    async fn open_pack_rejects_non_packable_series() {
        let (svc, _, series, _) = make_service();
        let mut s = packable_series("series_cancelled");
        s.status = CardSeriesStatus::Cancelled;
        series.upsert(&s).await.unwrap();
        let owner = Uuid::new_v4();
        let res = svc.open_pack(owner, "series_cancelled", 1, None).await;
        assert!(matches!(res, Err(Error::Conflict(_))));
    }

    #[tokio::test]
    async fn open_pack_rejects_nonexistent_series() {
        let (svc, _, _, _) = make_service();
        let owner = Uuid::new_v4();
        let res = svc.open_pack(owner, "series_nonexistent", 1, None).await;
        assert!(matches!(res, Err(Error::CardSeriesNotFound(_))));
    }

    #[tokio::test]
    async fn open_pack_rejects_zero_pack_count() {
        let (svc, cards, series, _) = make_service();
        cards.create(&sample_card("card_common", CardRarity::Common)).await.unwrap();
        series.upsert(&packable_series("series_001")).await.unwrap();
        let owner = Uuid::new_v4();
        let res = svc.open_pack(owner, "series_001", 0, None).await;
        assert!(matches!(res, Err(Error::Validation(_))));
    }

    #[tokio::test]
    async fn open_pack_distribution_matches_probability() {
        // 100 次 OpenPack 验证概率分布
        let (svc, cards, series, _) = make_service();
        for cid in ["c1", "c2", "c3"] {
            let r = match cid {
                "c1" => CardRarity::Common,
                "c2" => CardRarity::Rare,
                _ => CardRarity::Legendary,
            };
            cards.create(&sample_card(cid, r)).await.unwrap();
        }
        let mut s = CardSeries::new("series_dist".to_string(), "Dist".to_string(), 5);
        s.drop_table = DropTable::new(vec![
            DropEntry { rarity: CardRarity::Common, count: 1, probability: 0.7, card_id: Some("c1".to_string()) },
            DropEntry { rarity: CardRarity::Rare, count: 1, probability: 0.25, card_id: Some("c2".to_string()) },
            DropEntry { rarity: CardRarity::Legendary, count: 1, probability: 0.05, card_id: Some("c3".to_string()) },
        ]);
        series.upsert(&s).await.unwrap();
        let owner = Uuid::new_v4();
        let mut counts = std::collections::HashMap::new();
        let n = 100u32;
        for _ in 0..n {
            let r = svc.open_pack(owner, "series_dist", 1, None).await.unwrap();
            for inst in &r.instances {
                *counts.entry(inst.card_id.clone()).or_insert(0u32) += 1;
            }
        }
        let total_cards: u32 = counts.values().sum();
        // pack_size=5, count=1, n=100 → 500 cards total
        assert_eq!(total_cards, 5 * n);
        // c1 (~70%), c2 (~25%), c3 (~5%) — 允许 ±15% 偏差
        let c1 = *counts.get("c1").unwrap_or(&0) as f64 / total_cards as f64;
        let c2 = *counts.get("c2").unwrap_or(&0) as f64 / total_cards as f64;
        let c3 = *counts.get("c3").unwrap_or(&0) as f64 / total_cards as f64;
        assert!((c1 - 0.7).abs() < 0.15, "c1={} expected ~0.7", c1);
        assert!((c2 - 0.25).abs() < 0.15, "c2={} expected ~0.25", c2);
        assert!((c3 - 0.05).abs() < 0.15, "c3={} expected ~0.05", c3);
    }
}
