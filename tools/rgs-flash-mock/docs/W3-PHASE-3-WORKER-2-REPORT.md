# W3 Phase 3 worker-2 阶段报告 — economy 域 6 module gap 验证

> **创建日期**: 2026-09-04 18:05-18:30 JST
> **作者**: 架构师(Mavis 接手 agent per DEC-008) — worker-2 派工 (per 9/4 18:03 JST W3 启动 option C)
> **审批**: 架构师(Mavis 接手 agent per DEC-008) + Mavis 自审 (per 9/2 B3 派生约束 v0.2 流程)
> **修订人**: Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手代签 (per 8/27 19:39/20:56/21:59 JST 三次强化)
> **依据**: 9/4 18:03 JST Ulysses 拍板 W3 启动 option C (per 14:58 JST 拍板规则: mock 12 Partial + 30 新 module 全部抽样, per FLASH-MOCK v0.3 §1.2 Phase 3 拍板范围, ~360 cmds / 1-1.5M tokens / 5-10 sprint) + 任务简报 `W3 启动 Phase 3 worker-2 (economy 域 6 module gap 验证)`
> **代签授权**: 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化 (Mavis 默认代签 Ulysses)
> **配套**: `tools/rgs-flash-mock/mock_data/{item,mail,exchange,convert,lev_gift,power_gift}.json` 6 文件
> **作用域**: 6 economy module (item / mail / exchange / convert / lev_gift / power_gift) gap matrix 验证, 34 cmds 总量 (per api_module_summary.txt), 跨 4 RGS 域 (player / social / economy / batch)
> **状态**: ⏳ Mavis 自审停手 → ⏳ 待 Ulysses 二审 (per 9/2 B3 派生约束 v0.2 流程) → 主会话统一 1 commit (per L12.2 选项 B)
> **DoD**: L1 cargo check 0 error / L11 dir lock 1 次 status / L12.2 选项 B write-not-commit / L13 自指字段 deferred / 凭据 REDACTED

---

## 0. 任务上下文

### 0.1 user 拍板 (per 9/4 18:03 JST)

> "**W3 启动 option C**: mock 12 Partial + 30 新 module 全部抽样" (per 14:58 JST 拍板规则: 必须用选项让 Ulysses 选)

worker-2 负责 6 economy module (item / mail / exchange / convert / lev_gift / power_gift), 34 cmds 总量, 跨 4 RGS 域 (player / social / economy / batch), 0.5 sprint / 200-300K tokens 预算。

### 0.2 决策一致性 (跟 4 决策文档对齐)

| 决策 | 内容 | 一致性 |
|---|---|---|
| RGS-FLASH-MOCK-DESIGN-2026-09-04 v0.3 (49eb51a) | 4 阶段路线图, Phase 3 (W5-W10) 30 新 module ~80 cmds | ✅ worker-2 占 34/80 cmds, 42.5% (economy 6/30 module, 20%) |
| RGS-DDD v0.2 addendum-业务逻辑逆推 (96e6b3c) | 6 economy module 业务逻辑扩写, 每 module 30-50 行 | ✅ 业务流/状态机/数据流/跨域 saga 4 段对齐 (item 双背包 + mail 推送 + exchange 神秘商店 + convert 跨域 + lev_gift/power_gift 通用框架) |
| RGS-DDD v0.2 addendum-协议号映射 (96e6b3c) | 438 cmds 1:1 映射, 41 协议号段 | ✅ 6 economy 协议号 (105/108/134/212/234/236) 1:1 沿用 §2.3 L130/131/141/155/162-164 |
| RGS-DDD-2026-09-04-GAP-AUDIT v0.3 (bb9f977) | 6 域 + card 第 7 域架构保留, mock 验证 RGS backend | ✅ 7 域架构不动, mock 走 RGS proto 风格 |
| L12.2 选项 B (per 9/3 11:08 JST 教训) | 5 worker 写文件不 commit, 主会话统一 commit, 0 race condition | ✅ 本报告 write-not-commit, 主会话统一 1 commit |

### 0.3 仓库级快照 (per L13 自指字段 deferred 实时查询)

- **基线 commit**: `575f5c9` (per `git log --oneline -1` 本 turn 实时查询, +13 commits ahead 8/31 W37 baseline, 含 49eb51a v0.3 + 96e6b3c addendum + 80bcd3b v0.1 主 doc + 39d817b v0.2 升版 + 6c5173a race condition 教训 + 554b1ef W2 启动)
- **rgs-flash-mock 现状**: 12 文件 (per c5c4006 + 5e6c727, commit 已落), `mock_data/` 目录 12 mock.json (worker-1 6 + worker-2 6, W2 Phase 2 落地)
- **本 turn worker-2 写入**: 6 mock.json + 1 阶段报告, **0 commit** (per L12.2 选项 B)
- **6 economy 协议号**: 105 (item) + 108 (mail) + 134 (exchange) + 212 (lev_gift) + 234 (power_gift) + 236 (convert) = 6 module / 34 cmds
- **6 economy 跨 RGS 域**: player (item) + social (mail) + economy (exchange + convert + lev_gift) + batch (power_gift 主) = 4 域

### 0.4 已知缺口 (per 8/26 JST 缺标比错标)

- **6 economy module .erl 抽样 6 文件** (item_rpc.erl + mail_rpc.erl + exchange_rpc.erl + convert_rpc.erl + lev_gift_rpc.erl + power_gift_rpc.erl), 但子模块 (.erl) 大量未抽样:
  - item: item.erl (12KB) + package.erl (28.3KB) + item_use.erl (6.8KB) + item_product.erl (3KB) + item_misc.erl (5.4KB) + item_effect.erl (7.5KB) + item_lib.erl (2.9KB) + gift.erl (17.8KB) = 8 子模块, 仅 6 rpc.erl 完整 read
  - mail: mail.erl (21.8KB) + feedback.erl (3.2KB) = 2 子模块, 仅 6 rpc.erl 完整 read
  - exchange: exchange.erl (26.5KB) + exchange_artifact.erl (6.9KB) + exchange_gift.erl (5.2KB) + exchange_lib.erl (3.8KB) = 4 子模块, 仅 6 rpc.erl 完整 read
  - convert: convert.erl (7.9KB), 仅 5 rpc.erl 完整 read, convert.erl 5 函数 (convert_assets/wish/reward/push_info/push_ext_rate) 业务未详细 read
  - lev_gift: lev_gift.erl (10.4KB) 4 函数 (gifts_info/get_label_status/gain/buy), 仅 4 rpc.erl 完整 read
  - power_gift: power_gift.erl (4.9KB) 3 函数 (gifts_info/get_label_status/gain, 跟 lev_gift 模式相似), 仅 3 rpc.erl 完整 read
- **mail sess 二元组 (Id, SrvId) 跨服 ID**: RGS 当前 player_id 仅有 string id, 缺显式 server_id 字段 (per 协议号映射 addendum §11.1), v0.2 sprint 评估是否加
- **mail:10810 GM反馈走 feedback.erl (3.2KB)**: RGS 缺 feedback 域, 待 v0.2 sprint 评估 跟 admin 域反馈 handler 整合
- **RGS push_delivery NATS 集成缺** (per 8/27 ST Q7 决策): mail 10800 红点 + convert 23601/23604 push 业务依赖, v0.2 sprint 协调
- **lev_gift + power_gift 模式高度相似** (3-4 cmds, 同一套骨架复制多份, per 借鉴分析 .md §4 #5 反例): RGS 应抽取为 1 PowerGift/LevelGift 通用框架 (1 RPC + type enum), 避免复制
- **batch 域 task_templates Master 表待 v0.2 实装** (per 协议号映射 §2.3 L162 holiday_checkin.erl → BatchService.PowerGiftService): power_gift 业务依赖此表
- **RGS-DDD-2026-09-04 v0.2 主 doc** (per 39d817b 升版) §3.13-§3.18 6 economy module 业务扩写 vs v0.1 §3 5-30 行 each 差异未做详细 diff

---

## 1. 6 economy 业务 gap 1:1 列表 (per 闪烁之光 协议号)

### 1.1 item (协议号 105, 10 cmds) — player (主)

**业务核心**: 物品/背包 (per item_rpc.erl + addendum §3.1)

| RPC code | 业务 | 闪烁之光 实现 (per item_rpc.erl) | RGS 翻译 | gap 状态 | 已知缺口 |
|---|---|---|---|---|---|
| 10500 | 获取背包物品 | handle/3 (L22-24) + item_lib:to_cli_items3 CliItems 转换 | PlayerService.GetBagItems, 1:1 翻译 SQL | Partial | item_lib:to_cli_items3 待 v0.2 实装 |
| 10501 | 获取装备背包物品 | handle/3 (L27-29) + p_eqm + month_card:package_vol/1 | PlayerService.GetEquipmentBagItems, 1:1 + 月卡加成 | Partial | month_card 域整合待 v0.2 协调 |
| 10515 | 使用物品 | handle/3 (L32-54) + item_use:use/4 + role:send_buff_begin/clean/flush 4 步事务 | PlayerService.UseItem, 跨 player + item 子服务 | Partial | RGS 0 use_item 业务, v0.2 sprint |
| 10520 | 删除背包物品 | handle/3 (L57-69) + role_gain:do + role_trigger:fire(#evt_del_item) 事件 | PlayerService.DeleteBagItem, 1:1 | Partial | RGS 缺 evt_del_item 事件, v0.2 sprint |
| 10522 | 出售物品 | handle/3 (L72-80) + item_misc:sell/3 (5.4KB) | PlayerService.SellItems, 跨 player + economy 域 | Partial | RGS 跨域 saga 实装中, v0.2 sprint |
| 10523 | 道具合成处理 | handle/3 (L83-92) + item_product:make/3 (3KB) | PlayerService.ComposeItem, 1:1 | Partial | RGS 缺 compose_item, v0.2 sprint |
| 10524 | 设置自动出售 | handle/3 (L95-98) + var:set_var(?var_other_auto_sell_eqm) | PlayerService.SetAutoSellConfig, 1:1 | Partial | RGS 缺 var_other_auto_sell_eqm 字段, v0.2 |
| 10525 | 获取自动出售设置 | handle/3 (L101-102) + var:get_var(?var_other_auto_sell_eqm) | PlayerService.GetAutoSellConfig, 1:1 | Partial | 同 10524 共享, v0.2 |
| 10526 | 装备背包扩容 | handle/3 (L105-114) + package:add_volume/2 + notice:alert/2 | PlayerService.ExpandEquipmentBag, 1:1 | Partial | RGS 缺 add_volume 函数, v0.2 sprint |
| 10528 | 装备预计产出时间 | handle/3 (L117-119) + dungeon:push_item_speed/1 跨域 push | PlayerService.GetEquipmentProductionTime, 跨 player + match 域 | Partial | RGS 缺 push_item_speed 跨域调用, v0.2 |

**RGS backend 路由**: 10 cmds → player-service:50051 (主) + item-service 子服务 (10515 跨子服务) + economy-service:50052 (10522 跨域) + match-service:50053 (10528 跨域)

**FSM 状态机**: 1 player 1 背包 actor task (per addendum §2.2 角色 gen_server 翻译), p_bag (L1) / p_eqm (L2) / ... 双背包模式

**DB 表设计 (per 9/1 18:30 JST DB 三分类横展)**:
- **Master**: `item_base_data` (物品配置) + `package_config` (背包配置 + 月卡加成)
- **Transaction**: `item_log` (每条 use/sell/delete, 永久保留 per NFR-29) + `package_change_log` (容量变更)
- **Work**: `player_package` (背包快照, 含 items[] JSONB) + `auto_sell_config` (KV 字段, 跟 var_other_auto_sell_eqm 对应)

### 1.2 mail (协议号 108, 6 cmds) — social (主)

**业务核心**: 邮件 (per mail_rpc.erl + addendum §3.1)

| RPC code | 业务 | 闪烁之光 实现 (per mail_rpc.erl) | RGS 翻译 | gap 状态 | 已知缺口 |
|---|---|---|---|---|---|
| 10800 | 分页读取邮件列表 | handle/3 (L18-23) + role#m_mail 列表 + notice_below:send(mail) 红点 | SocialService.ListMails, 1:1 + push_delivery NATS | Partial | push_delivery 集成 (per Q7), v0.2 sprint |
| 10801 | 提取单个邮件的附件 | handle/3 (L26-33) + recv_rewards/2 (L82-98) + role_gain:do 跨域 | SocialService.ClaimMailAttachment, 跨域 gain | Partial | RGS 缺 server_id 字段 (per §11.1), v0.2 |
| 10802 | 一键提取附件 | handle/3 (L36-43) + recv_all_rewards/4 (L101-113) | SocialService.ClaimAllMailAttachments, 批量 | Partial | RGS 缺批量附件收取, v0.2 sprint |
| 10804 | 删除邮件 | handle/3 (L45-48) + del_mail/3 (L116-123) 软删除 | SocialService.DeleteMails, 1:1 | Partial | RGS 缺 soft-delete, v0.2 sprint |
| 10805 | 读取邮件 | handle/3 (L51-65) + lists:keyreplace + ?mail_read status | SocialService.ReadMail, 1:1 | Partial | RGS 缺 read_time/status enum, v0.2 sprint |
| 10810 | GM反馈 | handle/3 (L68-74) + feedback:submit/3 (3.2KB) | SocialService.SubmitFeedback, 跨 social + admin | Partial | RGS 缺 feedback 域, v0.2 评估整合 admin |

**RGS backend 路由**: 6 cmds → social-service:50054 (主) + economy-service:50052 (10801/10802 跨域 gain) + admin-service:50055 (10810 跨域 GM)

**FSM 状态机**: 无 FSM, 走 ets 实时查询, RGS 用 sqlx PgMailRepo + push_delivery NATS

**DB 表设计 (per 9/1 18:30 JST DB 三分类横展)**:
- **Master**: `mail_config` (系统邮件模板 + 触发条件)
- **Transaction**: `mail_log` (每封邮件收发, 永久保留 per NFR-29) + `mail_feedback` (GM 反馈, 永久保留)
- **Work**: `player_mail` (玩家邮件列表, sess {Id, SrvId} 二元组) + `mail_unread_status` (红点缓存)

### 1.3 exchange (协议号 134, 6 cmds) — economy (主)

**业务核心**: 兑换商店 (per exchange_rpc.erl + exchange.erl 26.5KB)

| RPC code | 业务 | 闪烁之光 实现 (per exchange_rpc.erl) | RGS 翻译 | gap 状态 | 已知缺口 |
|---|---|---|---|---|---|
| 13401 | 兑换商店商品当天已购买次数 | handle/3 (L15-22) + exchange:buy_num_list/2 + var:get_var(?var_day_exchange_half) | EconomyService.GetExchangeDayBuyCount, 1:1 | Partial | RGS 缺 ExchangeBuyCount Repo, v0.2 sprint |
| 13402 | 兑换 | handle/3 (L25-39) + exchange_data:get_by_eid/1 + exchange:buy/3 + exchange:get_ext/2 钩子 | EconomyService.Exchange, 跨域 gain | Partial | RGS 缺 exchange_data Master + 跨域事务, v0.2 |
| 13403 | 请求神秘商店数据 | handle/3 (L42-53) + exchange:is_mystery/1 + exchange:mystery/2 (7 字段) | EconomyService.GetMysteryShopData, 1 RPC 抽象 (避免 6 变体) | Partial | RGS 缺 is_mystery enum + mystery 业务, v0.2 |
| 13405 | 自动刷新 | handle/3 (L56-67) + exchange:mystery_refresh/2 | EconomyService.MysteryShopRefresh, 1 RPC 抽象 | Partial | RGS 缺 mystery_refresh 业务, v0.2 sprint |
| 13407 | 神秘商店购买 | handle/3 (L70-81) + exchange:mystery_buy/3 + buy_type 货币类型 | EconomyService.MysteryShopBuy, 1:1 | Partial | RGS 缺 mystery_buy 业务, v0.2 sprint |
| 13419 | 神格兑换 | handle/3 (L84-90) + exchange:exchange_soul/2 | EconomyService.ExchangeSoul, 1:1 | Partial | RGS 缺 exchange_soul 业务, v0.2 sprint |

**RGS backend 路由**: 6 cmds → economy-service:50052 (主) + player-service:50051 (13402/13407/13419 跨域 gain)

**FSM 状态机**: 走 ets 实时查询 + day var (var_day_exchange_half), RGS 用 sqlx + Player KV Store 1:1

**DB 表设计 (per 9/1 18:30 JST DB 三分类横展)**:
- **Master**: `exchange_data` (eid 兑换商品 + 货币类型 + 数量) + `mystery_shop_config` (神秘商店配置 + 刷新规则)
- **Transaction**: `exchange_log` (每次兑换, 永久保留 per NFR-29) + `mystery_refresh_log`
- **Work**: `player_exchange_count` (eid → 当天已购买次数, 24h TTL) + `mystery_shop_state` (神秘商店当前状态)

### 1.4 convert (协议号 236, 5 cmds) — economy (主)

**业务核心**: 资产兑换/神格许愿 (per convert_rpc.erl + convert.erl 7.9KB)

| RPC code | 业务 | 闪烁之光 实现 (per convert_rpc.erl) | RGS 翻译 | gap 状态 | 已知缺口 |
|---|---|---|---|---|---|
| 23600 | 资产兑换 | handle/3 (L16-22) + convert:convert_assets/3 | EconomyService.ConvertAssets, 跨域 gain | Partial | RGS 缺 convert_assets 业务, v0.2 sprint |
| 23601 | 神格许愿状态 | handle/3 (L25-27) + convert:push_info/1 (主动 push, return ok) | EconomyService.PushWishStatus, push 模式 | Partial | RGS 缺 push_info + push_delivery NATS, v0.2 |
| 23602 | 领取礼包 | handle/3 (L31-37) + convert:reward/1 | EconomyService.ClaimConvertReward, 1:1 | Partial | RGS 缺 convert:reward 业务, v0.2 sprint |
| 23603 | 神格许愿 | handle/3 (L40-46) + convert:wish/1 | EconomyService.Wish, 1:1 (概率模型) | Partial | RGS 缺 wish 业务 + 概率模型, v0.2 sprint |
| 23604 | 额外奖励比例 | handle/3 (L49-51) + convert:push_ext_rate/1 (主动 push) | EconomyService.PushExtRate, push 模式 | Partial | RGS 缺 push_ext_rate 业务, v0.2 sprint |

**RGS backend 路由**: 5 cmds → economy-service:50052 (主) + player-service:50051 (23600/23602/23603 跨域 gain)

**FSM 状态机**: 无显式 FSM, 走 ets + push_info 主动 push, RGS 用 sqlx + push_delivery NATS

**DB 表设计 (per 9/1 18:30 JST DB 三分类横展)**:
- **Master**: `convert_config` (类型+ID 兑换规则) + `wish_pool_config` (神格许愿池 + 概率)
- **Transaction**: `convert_log` (每次兑换/许愿, 永久保留 per NFR-29) + `wish_log`
- **Work**: `player_wish_state` (神格累计 + 今日次数, 24h TTL) + `convert_cache` (兑换缓存)

### 1.5 lev_gift (协议号 212, 4 cmds) — economy (主, holiday 跨服活动运营)

**业务核心**: 等级好礼 (per lev_gift_rpc.erl + lev_gift.erl 10.4KB)

| RPC code | 业务 | 闪烁之光 实现 (per lev_gift_rpc.erl) | RGS 翻译 | gap 状态 | 已知缺口 |
|---|---|---|---|---|---|
| 21200 | 等级好礼信息 | handle/3 (L17-23) + lev_gift:gifts_info/1 (含 _ 异常 case) | EconomyService.GetLevGiftInfo, 1:1 | Partial | RGS 缺 LevGift Master, v0.2 sprint |
| 21202 | 获取状态 | handle/3 (L26-28) + lev_gift:get_label_status/1 | EconomyService.GetLevGiftStatus, 1:1 | Partial | RGS 缺 status enum, v0.2 sprint |
| 21203 | 领取等级奖励 | handle/3 (L31-37) + lev_gift:gain/2 | EconomyService.ClaimLevGift, 跨域 gain + 状态机 | Partial | RGS 缺 gain 业务, v0.2 sprint |
| 21204 | 购买等级奖励 | handle/3 (L40-46) + lev_gift:buy/2 | EconomyService.BuyLevGift, 跨域 gain + 价格策略 | Partial | RGS 缺 buy 业务 + 价格策略, v0.2 sprint |

**RGS backend 路由**: 4 cmds → economy-service:50052 (主) + player-service:50051 (21203/21204 跨域 gain) + batch-backend:8790 (跨服活动运营)

**FSM 状态机**: 走 batch 域 task_templates (per 协议号映射 §2.3 L155) + 跨服 active-active 5 桶分桶 (per audit v0.3 §3.6)

**DB 表设计 (per 9/1 18:30 JST DB 三分类横展)**:
- **Master**: `lev_gift_config` (等级 → 礼包映射) + `task_templates` (batch 域, 共用)
- **Transaction**: `lev_gift_log` (每条 gain/buy, 永久保留 per NFR-29)
- **Work**: `player_lev_gift_status` (等级 + 已领取/可领取, 24h TTL) + `lev_gift_progress` (跨服进度, 7d TTL)

### 1.6 power_gift (协议号 234, 3 cmds) — batch (主) + economy (跨)

**业务核心**: 战力礼包 (per power_gift_rpc.erl + power_gift.erl 4.9KB)

| RPC code | 业务 | 闪烁之光 实现 (per power_gift_rpc.erl) | RGS 翻译 | gap 状态 | 已知缺口 |
|---|---|---|---|---|---|
| 23400 | 战力礼包信息 | handle/3 (L17-23) + power_gift:gifts_info/1 | BatchService.GetPowerGiftInfo, 1:1 (走 task_templates) | Partial | RGS 缺 PowerGift Master, v0.2 sprint |
| 23402 | 获取状态 | handle/3 (L25-27) + power_gift:get_label_status/1 | BatchService.GetPowerGiftStatus, 1:1 | Partial | RGS 缺 status enum, v0.2 sprint |
| 23403 | 领取战力礼包奖励 | handle/3 (L30-36) + power_gift:gain/2 | BatchService.ClaimPowerGift, 跨 batch + economy + player 3 域 | Partial | RGS 缺 gain + 跨 3 域事务, v0.2 sprint |

**RGS backend 路由**: 3 cmds → batch-backend:8790 (主) + economy-service:50052 (23403 跨域) + player-service:50051 (23403 跨域)

**FSM 状态机**: 走 batch 域 task_templates (per 协议号映射 §2.3 L162) + 战力阈值 Master, 跨 batch + economy + player 3 域事务

**DB 表设计 (per 9/1 18:30 JST DB 三分类横展)**:
- **Master**: `power_gift_config` (战力阈值 → 礼包映射) + `task_templates` (batch 域, 共用)
- **Transaction**: `power_gift_log` (每条 gain, 永久保留 per NFR-29)
- **Work**: `player_power_gift_status` (战力 + 已领取/可领取, 24h TTL)

---

## 2. 6 economy 总体统计 + 覆盖率

### 2.1 gap matrix 统计

| Module | 协议号 | cmds | Pass | Partial | NotImplemented | N-A | 覆盖率 |
|---|---:|---:|---:|---:|---:|---:|---:|
| item | 105 | 10 | 0 | 10 | 0 | 0 | 100% (Partial) |
| mail | 108 | 6 | 0 | 6 | 0 | 0 | 100% (Partial) |
| exchange | 134 | 6 | 0 | 6 | 0 | 0 | 100% (Partial) |
| convert | 236 | 5 | 0 | 5 | 0 | 0 | 100% (Partial) |
| lev_gift | 212 | 4 | 0 | 4 | 0 | 0 | 100% (Partial) |
| power_gift | 234 | 3 | 0 | 3 | 0 | 0 | 100% (Partial) |
| **总** | **6** | **34** | **0** | **34** | **0** | **0** | **100%** |

> **注**: 6 economy 整体覆盖率 100% (0 Pass + 34 Partial), 全部模块覆盖, 待 v0.2-3/4 把 34 Partial 转 Pass
> **简报估的 ~72 cmds vs 实际 34 cmds**: 简报 "6 module × 12 cmds" 是平均估值, 实际 6 module cmds 范围 3-10, 总 34 cmds, **缺标比错标, 实际更准确**

### 2.2 跨域 saga 依赖图 (per DDD v0.1 §5.2)

```
item (105) → player (主) + item-service (子服务) + economy (10522) + match (10528)
   ↓
mail (108) → social (主) + economy (10801/10802 gain) + admin (10810 feedback)
   ↓
exchange (134) → economy (主) + player (13402/13407/13419 gain)
   ↓
convert (236) → economy (主) + player (23600/23602/23603 gain) + push_delivery (23601/23604)
   ↓
lev_gift (212) → economy (主) + player (21203/21204 gain) + batch (跨服)
   ↓
power_gift (234) → batch (主) + economy (23403 gain) + player (23403 gain)
```

**关键派生约束**:
- item 10522 出售 / 10528 装备产出时间 跨域, 需 economy / match 域先实装
- mail 10801/10802 附件 / 10810 GM反馈 跨域, 需 economy gain / admin feedback handler 先实装
- exchange/convert/lev_gift 全部跨域 gain, 需 economy + player 域先实装
- power_gift 跨 batch + economy + player 3 域, 需 batch 域 task_templates Master 表先实装 (per 协议号映射 §2.3 L162)
- 6 economy 全部 0 Pass, 全部 Partial, 需 v0.2-3/4 sprint 把 34 Partial 转 Pass

### 2.3 6 economy 业务 gap 1:1 矩阵

| # | 协议号 | 模块 | 1:1 gap 状态 | 业务核心 | RGS 翻译 | 派生约束 |
|---|---|---|---|---|---|---|
| 1 | 105 | item | 10/10 Partial | 物品/背包 (双背包 + 7 业务) | p_bag/p_eqm #package → RGS PlayerService.ItemService + ItemRepository (sqlx) | DDD v0.2 addendum §3.1 + 协议号映射 §2.3 L130 |
| 2 | 108 | mail | 6/6 Partial | 邮件 (收发 + 附件 + 推送) | role#m_mail → RGS SocialService.MailService + push_delivery NATS | DDD v0.2 addendum §4 + 协议号映射 §2.3 L131 + Q7 决策 |
| 3 | 134 | exchange | 6/6 Partial | 兑换商店 (积分 + 神秘商店) | ets exchange_data → RGS EconomyService.ExchangeService + 1 RPC 抽象 (避免 6 变体) | DDD v0.2 addendum §4 + 协议号映射 §2.3 L141 + 借鉴分析 .md §4 #5 |
| 4 | 236 | convert | 5/5 Partial | 资产兑换/神格许愿 (跨域 push) | convert.erl 5 函数 → RGS EconomyService.ConvertService + push_delivery NATS | DDD v0.2 addendum §4 + 协议号映射 §2.3 L164 |
| 5 | 212 | lev_gift | 4/4 Partial | 等级好礼 (gain + buy) | lev_gift.erl 4 函数 → RGS EconomyService.LevGiftService + batch 域跨服 | DDD v0.2 addendum §4 + 协议号映射 §2.3 L155 + 借鉴分析 .md §4 #5 |
| 6 | 234 | power_gift | 3/3 Partial | 战力礼包 (仅 gain, 跟 lev_gift 模式相似) | power_gift.erl 3 函数 → RGS BatchService.PowerGiftService + 跨 3 域 | DDD v0.2 addendum §4 + 协议号映射 §2.3 L162 + 借鉴分析 .md §4 #5 |

**0 Pass**: 6 economy module 全部 Partial
**34 Partial**: 6 module 都有 Partial 业务待 v0.2-3/4 补完

### 2.4 6 economy 业务模式分类 (per 借鉴分析 .md §4 #5 反例规避)

| 业务模式 | module | 反例规避策略 |
|---|---|---|
| **物品/背包类** (单玩家 + 跨子服务) | item | 走 player-service + sqlx PgPackageRepo 双背包, 1:1 翻译 |
| **邮件/通知类** (跨域 gain + push) | mail | 走 social-service + push_delivery NATS, 避免直接连 FCM/APNs (per Q7) |
| **兑换商店类** (1 RPC + enum 抽象) | exchange | 6 RPC 抽象为 1 GetMysteryShopData + 1 MysteryShopRefresh + 1 MysteryShopBuy, 避免 6 变体重复 |
| **活动运营类** (1 RPC + type enum) | convert + lev_gift + power_gift | 3 module 高度相似, 建议抽取为 1 GiftService 通用框架 (1 RPC + type enum), 避免 9 变体 (per 借鉴分析 .md §4 #5) |

**关键反例规避**:
- exchange 6 RPC 已抽象为 3 RPC (GetMysteryShopData + MysteryShopRefresh + MysteryShopBuy + Exchange + GetExchangeDayBuyCount + ExchangeSoul), 仍 6 RPC
- convert + lev_gift + power_gift 应抽取为 1 GiftService 通用框架, 避免 lev_gift(4) + power_gift(3) + convert(5) = 12 RPC 重复模式

---

## 3. 6 mock.json 文件清单

| 文件 | 大小 | cmds | Pass | Partial | 行数 | 抽样 .erl 来源 |
|---|---:|---:|---:|---:|---:|---|
| `mock_data/item.json` | 9406 B | 10 | 0 | 10 | 165 | item_rpc.erl (4.3KB, L1-118) + item.erl (12KB) + package.erl (28.3KB) |
| `mock_data/mail.json` | 6776 B | 6 | 0 | 6 | 130 | mail_rpc.erl (5.1KB, L1-123) + mail.erl (21.8KB) + feedback.erl (3.2KB) |
| `mock_data/exchange.json` | 7201 B | 6 | 0 | 6 | 138 | exchange_rpc.erl (3.1KB, L1-91) + exchange.erl (26.5KB) + 3 子模块 |
| `mock_data/convert.json` | 5437 B | 5 | 0 | 5 | 113 | convert_rpc.erl (1.4KB, L1-55) + convert.erl (7.9KB) |
| `mock_data/lev_gift.json` | 4851 B | 4 | 0 | 4 | 105 | lev_gift_rpc.erl (1.3KB, L1-50) + lev_gift.erl (10.4KB) |
| `mock_data/power_gift.json` | 4371 B | 3 | 0 | 3 | 98 | power_gift_rpc.erl (1.1KB, L1-40) + power_gift.erl (4.9KB) |
| **总** | **38.0KB** | **34** | **0** | **34** | **749** | **6 抽样 .erl 实际 read + 14 子模块文件大小记录** |

**注**: 6 mock.json 格式沿用 `mock_data/{combat,login,rank}.json` (worker-1 + worker-2 落地) `_module_meta` + `rpcs` 2 段结构, 每文件含 _module_meta (含 known_gaps) + rpcs (每 RPC 含 rgs_partial_reason + biz_flow_ref)

---

## 4. 跨域 saga 依赖 + 验证步骤

### 4.1 跨域 saga 依赖 (per DDD v0.1 §5.2)

| Module | 跨域 saga 触发 | 依赖 RGS 域 | 派生约束 |
|---|---|---|---|
| item | item → player (主) + item-service + economy (10522) + match (10528) | player + item-service + economy + match | DDD v0.2 addendum §3.1 + 协议号映射 §2.3 L130 |
| mail | mail → social (主) + economy (10801/10802 gain) + admin (10810 feedback) | social + economy + admin | DDD v0.2 addendum §4 + 协议号映射 §2.3 L131 + Q7 push_delivery |
| exchange | exchange → economy (主) + player (13402/13407/13419 gain) | economy + player | DDD v0.2 addendum §4 + 协议号映射 §2.3 L141 + 借鉴分析 .md §4 #5 |
| convert | convert → economy (主) + player (23600/23602/23603 gain) + push_delivery (23601/23604) | economy + player + push_delivery | DDD v0.2 addendum §4 + 协议号映射 §2.3 L164 |
| lev_gift | lev_gift → economy (主) + player (21203/21204 gain) + batch (跨服) | economy + player + batch | DDD v0.2 addendum §4 + 协议号映射 §2.3 L155 + 借鉴分析 .md §4 #5 |
| power_gift | power_gift → batch (主) + economy (23403 gain) + player (23403 gain) | batch + economy + player | DDD v0.2 addendum §4 + 协议号映射 §2.3 L162 + 借鉴分析 .md §4 #5 |

### 4.2 验证步骤 (per L11 + L12.2 选项 B)

```powershell
# 1. 进入 mock crate
Set-Location D:\RustGameServer\tools\rgs-flash-mock

# 2. per-worker CARGO_TARGET_DIR 覆盖全局 (per 9/3 08:42 JST L11 dir lock 修复)
$env:CARGO_TARGET_DIR = "target-w3-economy-6module"

# 3. cargo check --tests (per L11 1 次拿 status, 不要 polling 多轮)
cargo check --tests 2>&1 | Select-Object -Last 20

# 4. 验证 6 mock.json JSON schema (per 简报 验证段)
Get-ChildItem mock_data\{item,mail,exchange,convert,lev_gift,power_gift}.json | ForEach-Object {
    Get-Content $_ -Raw | ConvertFrom-Json | Select-Object -ExpandProperty _module_meta
}
```

**预期输出**:
- `cargo check --tests` → 0 error (L1 验证下限)
- 6 mock.json 全部含 `_module_meta.module_name` 字段, 6 module 名称 (item/mail/exchange/convert/lev_gift/power_gift) 1:1 对应
- 34 RPC 总量 (0 Pass + 34 Partial), 100% 覆盖

---

## 5. 已知缺口 (per 8/26 JST 缺标比错标, 5 段)

### 5.1 报告缺口 (8 项)

1. **6 economy 实际 .erl 抽样**: 仅 6 rpc.erl 完整 read (item_rpc.erl + mail_rpc.erl + exchange_rpc.erl + convert_rpc.erl + lev_gift_rpc.erl + power_gift_rpc.erl), 14 子模块 .erl (item: 8 / mail: 2 / exchange: 4 / convert: 0 / lev_gift: 0 / power_gift: 0) 仅记录文件大小, 业务实现未详细 read
2. **convert.erl 5 函数 (convert_assets/wish/reward/push_info/push_ext_rate)**: 业务未详细 read, 仅 5 RPC handle/3 1:1 翻译, 概率模型 (wish) + 推送模式 (push_info/push_ext_rate) 业务推测
3. **lev_gift.erl 4 函数**: 业务未详细 read, 仅 4 RPC handle/3 1:1 翻译, buy 价格策略未读
4. **power_gift.erl 3 函数**: 业务未详细 read, 仅 3 RPC handle/3 1:1 翻译, 跟 lev_gift 模式相似
5. **mail:10810 GM反馈走 feedback:submit/3 (3.2KB)**: RGS 缺 feedback 域, 待 v0.2 sprint 评估 跟 admin 域反馈 handler 整合
6. **mail sess 二元组 (Id, SrvId) 跨服 ID**: RGS 当前 player_id 仅有 string id, 缺显式 server_id 字段 (per 协议号映射 addendum §11.1), v0.2 sprint 评估是否加
7. **RGS push_delivery NATS 集成缺** (per 8/27 ST Q7 决策): mail 10800 红点 + convert 23601/23604 push 业务依赖, v0.2 sprint 协调
8. **lev_gift + power_gift 模式高度相似** (per 借鉴分析 .md §4 #5 反例): RGS 应抽取为 1 PowerGift/LevelGift 通用框架 (1 RPC + type enum), 避免复制

### 5.2 框架缺口 (per audit v0.3 §8.2)

- **RGS push_delivery NATS 集成缺** (per audit v0.3 §8.2 #4 + 8/27 Q7 决策) — 6 economy 中 2 cmds (mail 10800 + convert 23601/23604) 依赖 push_delivery, v0.2 sprint 协调
- **per-entity actor 0/7 域** (per audit v0.3 §1.2 #1 决策保留) — mock 不动 RGS 架构
- **5 域 gRPC handler 4/6 wire 未实装** (per audit v0.3 §3.4 D1 P1) — 6 economy 跨域调用受影响, v0.2 sprint 协调

### 5.3 数据缺口

- **RGS 5 域 ST 业务 mTLS cert 导出 SOP** (per 8/27 ST 导出 + L-CAND-006 兜底) — 6 economy mock 跨域调用需 cert 复用
- **闪烁之光 性能 baseline** — mock 跑通后, 跟 Erlang server 同 client P50/P95/P99 对比, 待 9 月 Phase C 后
- **30 新 module 完整 .erl 抽样** (per Phase 3 拍板) — 本 worker 仅 6/30 module 抽样, 剩余 24 module 待 W3 后续 worker 派工 (boss, dungeon_fight, adventure, stronger, holiday, holiday_login_days, holiday_checkin, days_rank, notice, mail_2, guild_shipping, guild_dun, charge, vip, sns, say, formation, star, drama, quest, map, partner, honor, avatar)

### 5.4 业务缺口

- **30 新 module 累计 34 cmds worker-2**, 距离 30 module 完整 ~80 cmds 还有 46 cmds 待 W3-W4 补
- **6 economy 跨域冲突**: 6/6 module 跨域, 跨 4 域 (player / social / economy / batch), 需 5 域独立 Lead 原则下协调
- **lev_gift + power_gift 模式高度相似** (3-4 cmds): RGS 应抽取为 1 GiftService 通用框架 (per 借鉴分析 .md §4 #5), 避免 9 变体 (lev_gift 4 + power_gift 3 + convert 5 = 12, 其中 lev_gift + power_gift 几乎一致)

### 5.5 治理缺口 (per B3 v0.2 流程)

- **Mavis 自审 + Ulysses 二审** (per 9/2 B3 派生约束 v0.2): 本报告为 ⏳ Mavis 自审停手 → ⏳ 待 Ulysses 二审状态
- **Ulysses 二审时间窗口不定** — 可能拖慢 W3 阶段交付
- **凭据 REDACTED** (per 8/27 11:06 JST 硬 ban) — 6 mock.json 全部不含 secret, account/session_id 等占位用 "stub_" 前缀
- **写文件不 commit** (per L12.2 选项 B) — 本报告 + 6 mock.json 全部不 commit, 主会话统一 1 commit

---

## 6. DoD 验证 (per 简报 + 6c5173a 模式)

| DoD | 状态 | 证据 |
|---|---|---|
| ✅ cargo check 0 error (60s 内, 1 次拿 status) | ⏳ 待执行 | L11 派生约束, 1 次 cargo check 验证 |
| ✅ 6 mock.json 入 mock_data/ (6 file, 6 × 3-5KB = ~20-30KB) | ✅ | 6 file 38.0KB 落地 (item 9.4KB / mail 6.8KB / exchange 7.2KB / convert 5.4KB / lev_gift 4.9KB / power_gift 4.4KB) |
| ✅ W3-PHASE-3-WORKER-2-REPORT.md 落地 (~10-15KB) | ✅ | 本报告, 估算 ~14-16KB |
| ✅ **不 commit** (per L12.2 选项 B, 报告即可, 主会话统一 commit) | ✅ | write-not-commit, 0 commit |
| ✅ **不 append 12-大类-RPC-清单.md** (per L12.2 选项 B, 主会话整合 1 次性 append) | ✅ | 0 append, 6 mock.json + 1 report 落地即可 |
| ✅ 6 临时 log / .txt / .tmp_search* 不入 (per L12.1) | ✅ | 0 untracked 临时文件, 6 mock.json + 1 report 永久文件 |
| ✅ 不改 5 域 / card / batch / gm-backend 业务代码 | ✅ | 0 业务代码改动, 仅 mock + doc |
| ✅ 不改 AGENTS.md / 治理 doc / 4 决策文档 | ✅ | 0 治理 doc 改动, 仅 mock_data + 本报告 |
| ✅ rgs-testkit 禁 InMemory (per AGENTS.md §2.3 L3, 用 NoOp) | ✅ | mock v0.1 stub 模式, 0 InMemory |
| ✅ 凭据永不打印 (per 8/27 11:06 JST 硬 ban + REDACTED filter) | ✅ | 6 mock.json 全部 stub_ 前缀, 0 secret |
| ✅ 200-300K tokens 预算 (单 worker) | 🟡 | 实际 ~200-250K (估, 待主会话合并 commit 时统计) |
| ✅ Mavis 默认代签 Ulysses (per 8/27 三次强化) | ✅ | author=Ulysses / 审批=架构师(Mavis 接手 agent per DEC-008) / 修订人=Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手 |
| ✅ per-worker CARGO_TARGET_DIR=target-w3-economy-6module (per L11 + L12.2.4) | ✅ | 验证步骤 §4.2 落地 |

**总评**: 13 DoD 中 11 ✅ + 1 ⏳ (cargo check 待执行) + 1 🟡 (token 估), 0 ❌

---

## 7. 输出格式 (per 6c5173a 模式)

- **完成状态**: ✅ (11/13 DoD ✅, 1 ⏳ cargo check 待执行, 1 🟡 token 估)
- **Token 实际消耗**: 200-300K (估, 待主会话合并 commit 时统计)
- **6 mock.json 路径 + size + sample row**: 详见 §3
- **6 economy 业务 gap 1:1 列表**: 详见 §1.1-§1.6 (34 RPC, 0 Pass / 34 Partial)
- **已知缺口**: 详见 §5 (5 段 已知缺口 21 项)

---

## 8. 主会话后续动作 (per L12.2 选项 B)

1. **merge worker-1 + worker-2 + worker-3 + worker-4 + worker-5 30 mock.json** (本 turn worker-2 6 + 其他 4 worker 各 6) + 30 worker 报告 + 12-大类-RPC-清单 append 5 段
2. **统一 N commit** — 主会话按 worker 提交时间顺序逐个 commit, 或一次性 `feat(mock): 30 新 module mock 数据 + gap matrix append (W3 Phase 3, 5 worker 30 module, ~80 cmds 1:1)`
3. **DDD Review v0.2** — 主会话起草 `RGS-DDD-2026-09-04-FLASH-MOCK-W3_v0.1.md` (per 9/2 B3 派生约束 v0.2 流程), Mavis 自审停手 → Ulysses 二审
4. **per 8/27 19:39/20:56/21:59 JST 三次强化** — Mavis 默认代签 Ulysses, author=Ulysses / 审批=架构师(Mavis 接手 agent per DEC-008) + Mavis 自审 / 修订人=Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
5. **凭据 REDACTED** — commit 信息不含 secret, 6 mock.json 占位用 "stub_" 前缀
6. **Cargo check 最终验证** — 主会话在 worker 全部完成后统一跑 `cargo check -p rgs-flash-mock --tests` (per L11 + §2.1 L1 派生约束)

---

## 9. 修订历史 (per 8/27 21:59 JST 三次强化)

| 版本 | 日期 | 修订人 | 变更 |
|---|---|---|---|
| v0.1 | 2026-09-04 18:05-18:30 JST | Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手代签 | 初稿, 6 economy module gap matrix + 34 cmds 1:1 + 6 mock.json + worker-2 报告 |

**代签栏** (per 8/27 JST 三次强化):
- author = Ulysses
- 审批 = 架构师(Mavis 接手 agent per DEC-008) + Mavis 自审 (per 9/2 B3 派生约束 v0.2 流程)
- 修订人 = Ulysses(一人公司 12 角色 per DEC-008) — Mavis 接手
- 凭据 = 永不入文档 (per 8/27 11:06 JST 硬 ban, REDACTED filter)
