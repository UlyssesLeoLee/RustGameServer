# 详细设计书（詳細設計書 / Detailed Design Document）

**客户端资源分发与热更新：asset_db物理数据库设计・清单查询协议格式・分桶灰度与校验算法详细设计**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-DTL-027 |
| 版本 | 0.2 |
| 父文档 | RGS-BAS-027 客户端资源分发与热更新 基本设计书（本文档为其详细化，不改变任何既有决定，仅将逻辑设计落实为物理/实现级设计） |
| 依据标准 | IPA『共通フレーム 2013（SLCP-JCF2013）』详细设计工程 |
| 制定日 | 2026-08-17 |
| 制定者 | 架构师 |
| 保密级别 | 内部限定（Internal Use Only） |
| 适用许可 | Apache-2.0（本仓库） |

---

## 修订历史（改訂履歴 / Revision History）

| 版本 | 修订日 | 修订者 | 审批者 | 修订内容 | 影响章节 |
|---|---|---|---|---|---|
| 0.1 | 2026-08-17 | 架构师 | — | 初版制定（负责人指示"继续"推进详细设计，本文档为第五份详细设计文档）。细化RGS-BAS-027§2.2数据模型为`asset_db`内`asset_manifests`／`asset_file_entries`／`rollouts`三表具体DDL、§2.3差异计算/§4完整性校验/§5.1确定性分桶落实为可直接翻译为Rust实现的伪代码、§2.4匿名查询接口落实为具体协议格式，并列出自托管对象存储与反向代理**候选**。该候选在 v0.2 起不再构成默认或许可符合性结论，生产启用以 `ApprovedBackendProfile` 审批门禁为准。**本版本不覆盖**：§6.3商业CDN可选实现的具体接入代码、发布流水线CI具体脚本（复用RGS-DTL-002§4已确立的CI阶段模式，本文档不重复展开）。见§7 | 全部 |
| 0.2 | 2026-08-20 | 架构师 | — | 纳入RGS-REQ-030-ADD1：将候选组件与生产后端解耦，定义 `ApprovedBackendProfile` 的 BOM/许可证/ADR 启用门禁、边缘 TTL 和回源契约；候选实现不得视为已批准默认值。 | §6.1〜§6.4、§7、追溯性 |

## 审批栏（承認欄 / Approval）

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 制定（起草） | 架构师 | 2026-08-17 | — |
| 评审（技术） | | | DDL是否与RGS-BAS-027§2.2逻辑模型一致，分桶哈希是否与既有分片路由一致性哈希实现真正复用（而非另起一套） |
| 评审（安全） | | | §4清单签名验签伪代码是否存在绕过路径，公钥内置分发链路是否真正独立于本系统动态更新链路 |
| 评审（负责人） | | | 本文档的基准化；TBD-CDN-002候选后端是否已具备 BOM、许可证/商业条款审查和 ADR，不得因本文档列举候选而直接采纳。 |

---

## 目录

1. [前言](#1-前言)
2. [物理数据库设计：asset_db](#2-物理数据库设计assetdb)
3. [清单查询协议格式](#3-清单查询协议格式)
4. [完整性校验与签名验证算法详细设计](#4-完整性校验与签名验证算法详细设计)
5. [确定性分桶灰度算法详细设计](#5-确定性分桶灰度算法详细设计)
6. [经批准分发后端具体部署形态](#6-经批准分发后端具体部署形态)
7. [本文档的覆盖范围与后续计划](#7-本文档的覆盖范围与后续计划)

---

## 1. 前言

### 1.1 定位

RGS-BAS-027给出了`AssetManifest`/`AssetFileEntry`/`Rollout`的逻辑字段表、差分流水线与完整性校验的流程/时序图、分桶灰度的伪代码级描述，以及`DistributionBackend`接口契约与既有部署方向。本文档将其落实为可执行DDL、清单查询接口的具体协议格式、签名验证与分桶算法的完整伪代码，以及经批准分发后端的部署契约与候选配置示例。

### 1.2 本文档不做什么

- 不重新决定RGS-BAS-027已确定的任何结构性选择（`ManifestService`无状态、匿名查询不接入会话鉴权、完整性校验不可绕过）；既有“自托管优先/商业可选”方向不等于任何实现已获批准。
- 不选定TBD-CDN-002之外的开放问题——本文档对自托管对象存储/反向代理给出**候选提案**（如 MinIO＋Nginx），但不作许可证符合性或生产选型结论；最终选型须核对附件D的OSS许可一览并完成 BOM、许可证/商业条款审查、ADR 与具名审批，本文档的候选不构成评审本身。
- 不覆盖§6.3商业CDN可选实现的具体接入代码，也不覆盖发布流水线的完整CI脚本（复用RGS-DTL-002§4已确立的CI阶段模式与写法，此处不重复展开，仅在§7声明复用关系）。

### 1.3 记述规则

沿用既有DTL文档记述规则：DDL以PostgreSQL为准，协议格式以Protobuf/HTTP风格给出（清单查询接口为匿名HTTP查询，非既有gRPC鉴权路径，故本文档在协议格式章节区分标注），算法伪代码可直接对应Rust `Result`实现。

---

## 2. 物理数据库设计：asset_db

对应RGS-BAS-027§2.1/§2.2。`asset_db`为按ARC-008独立DB边界原则新挂载的数据库，遵循RGS-DTL-002已确立的挂载脚手架物理落地方式（Helm模板/CI流水线/Mount Record格式不在本文档重复，直接复用）。

```sql
-- 资源清单表，对应FR-CDN-001/003/024
CREATE TABLE asset_manifests (
    manifest_version              BIGSERIAL PRIMARY KEY,  -- 单调递增,由序列保证,不使用UUID(需要严格递增语义支撑§2.3差异比对与§5.2 stable/target判定)
    release_notes                  TEXT NOT NULL DEFAULT '',
    is_forced_update                BOOLEAN NOT NULL DEFAULT FALSE,
    min_supported_client_version     TEXT NOT NULL,
    published_at                     TIMESTAMPTZ NOT NULL DEFAULT now(),
    rollout_id                        BIGINT NULL REFERENCES rollouts(rollout_id),
    manifest_signature                 BYTEA NOT NULL   -- §4.2清单签名,发布流水线写入时计算
);

-- 资源文件条目表，对应FR-CDN-001/002
CREATE TABLE asset_file_entries (
    manifest_version   BIGINT NOT NULL REFERENCES asset_manifests(manifest_version),
    file_path            TEXT NOT NULL,
    file_version           BIGINT NOT NULL,
    checksum                 TEXT NOT NULL,     -- 十六进制编码,算法见§4.1
    size_bytes                BIGINT NOT NULL,
    delta_available_from       BIGINT[] NOT NULL DEFAULT '{}',  -- 适用的源file_version列表,对应§3.1
    PRIMARY KEY (manifest_version, file_path)
);
CREATE INDEX idx_asset_file_entries_path_version
    ON asset_file_entries (file_path, file_version);  -- 支撑§2.3逐字段差异比对查询

-- 灰度批次表，对应FR-CDN-020〜022
CREATE TABLE rollouts (
    rollout_id                BIGSERIAL PRIMARY KEY,
    target_manifest_version     BIGINT NOT NULL REFERENCES asset_manifests(manifest_version),
    stable_manifest_version      BIGINT NOT NULL REFERENCES asset_manifests(manifest_version),
    percentage                    SMALLINT NOT NULL DEFAULT 0 CHECK (percentage BETWEEN 0 AND 100),
    status                          TEXT NOT NULL DEFAULT '灰度中'
                                       CHECK (status IN ('灰度中', '已全量', '已回滚')),
    updated_at                       TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

`asset_manifests.rollout_id`与`rollouts.target_manifest_version`互相引用（一个清单版本可能是某灰度批次的目标，一个灰度批次固定引用一个目标清单版本）：写入顺序为先插入`asset_manifests`（`rollout_id`暂空）→ 插入`rollouts`（引用刚创建的`manifest_version`）→ 回填`asset_manifests.rollout_id`，三步在同一事务内完成，避免两张表存在瞬时不一致的可见窗口。

---

## 3. 清单查询协议格式

对应RGS-BAS-027§2.1/§2.4——`ManifestService`是匿名可访问的只读查询服务，不接入既有gRPC会话鉴权路径，故采用HTTP+JSON而非既有内部服务间的Protobuf gRPC格式（更贴合客户端在未建立游戏会话前即可发起查询的场景，同时便于自托管反向代理层§6按URL模式做缓存策略配置）：

```
GET /v1/manifest/latest?client_manifest_version={N}&device_id={D}&player_id={P?}
```

- `client_manifest_version`：客户端当前持有的`manifest_version`，缺省视为0（首次安装场景，返回全量清单）
- `device_id`：必填，未登录场景的分桶依据（§5）
- `player_id`：可选，已登录场景优先使用（同一玩家换设备时分桶结果保持稳定，避免设备迁移导致灰度状态跳变）

响应体（JSON，字段名与§2表结构一一对应，蛇形命名遵循既有API既定风格）：

```json
{
  "manifest_version": 1024,
  "release_notes": "...",
  "is_forced_update": false,
  "min_supported_client_version": "3.2.0",
  "manifest_signature": "base64(...)",
  "files": [
    {
      "file_path": "assets/scene_01.bin",
      "file_version": 17,
      "checksum": "sha256:...",
      "size_bytes": 4831201,
      "download_url": "https://.../assets/scene_01.bin.delta-16-17",
      "is_delta": true
    }
  ]
}
```

`download_url`由`ManifestService`在响应时调用`DistributionBackend.get_url`（§6.1既定接口）实时生成，不在`asset_file_entries`表内持久化具体URL——避免经批准后端的 URL 格式变化（如从一个 `ApprovedBackendProfile` 切换至另一个）需要回填历史清单数据。`is_delta`标记该条目返回的是差分包还是全量基线，由`ManifestService`按§3.1判定逻辑与客户端携带的`client_manifest_version`比对后决定。

---

## 4. 完整性校验与签名验证算法详细设计

对应RGS-BAS-027§4.1〜4.3时序图，落实为客户端SDK`asset_update`模块内的伪代码。

### 4.1 清单签名验证（对应§4.2/§4.3，校验不可绕过）

```rust
// 内置公钥,随客户端构建分发,不经本系统动态更新链路(§4.2既定约束)
const MANIFEST_VERIFY_PUBLIC_KEY: &[u8] = include_bytes!("embedded_manifest_pubkey.pem");

fn verify_manifest(manifest: &AssetManifestResponse) -> Result<(), AssetUpdateError> {
    let signed_payload = canonical_encode(manifest_without_signature_field(manifest));  // 签名前的规范化编码,须与发布流水线签名时使用的编码方式完全一致
    let sig = base64_decode(&manifest.manifest_signature)?;
    verify_signature(MANIFEST_VERIFY_PUBLIC_KEY, &signed_payload, &sig)
        .map_err(|_| AssetUpdateError::ManifestSignatureInvalid)
    // 验签失败: 直接返回Err,调用方(§4.1时序"客户端校验清单签名")在此中断流程,不进入下载阶段
    // 无任何配置项可跳过本函数调用本身(NFR-CDN-002,§4.3既定)
}
```

### 4.2 文件内容校验（对应§4.1时序图后半段）

```rust
fn apply_downloaded_file(entry: &AssetFileEntry, bytes: &[u8]) -> Result<(), AssetUpdateError> {
    let actual_checksum = sha256_hex(bytes);
    if actual_checksum != entry.checksum {
        return Err(AssetUpdateError::ChecksumMismatch {
            file_path: entry.file_path.clone(),
            retryable: true,  // 对应FR-CDN-012"可重试"
        });
    }
    if entry.is_delta {
        apply_delta_patch(&entry.file_path, bytes)?;  // 差分应用: 读取本地文件+补丁字节流合成新文件
    } else {
        write_full_file(&entry.file_path, bytes)?;
    }
    Ok(())
}
```

`apply_delta_patch`失败（本地文件在差分应用过程中发现与预期基线不符，例如本地文件在两次检查之间被外部篡改）时，落实§3.1"自愈基线"设计：调用方捕获该错误后应重新以`is_delta=false`向`ManifestService`请求全量基线（而非本文档定义的错误重试逻辑本身继续尝试差分），此处不展开该上层重试策略，因其属于RGS-BAS-027§3.1已完整描述的既定行为，本文档仅确保底层`apply_delta_patch`在校验失败时明确返回可区分错误类型，供上层据此选择自愈路径。

---

## 5. 确定性分桶灰度算法详细设计

对应RGS-BAS-027§5.1伪代码，落实为具体哈希实现与`ManifestService`响应生成逻辑。

```rust
// 复用既有分片路由一致性哈希函数(RGS-BAS-022§3.1既定),非本文档新增算法
fn bucket_for(identity: &str) -> u8 {
    let h = existing_consistent_hash(identity.as_bytes());  // 复用点: 与分片路由同一函数,避免维护两套哈希实现
    (h % 100) as u8
}

fn resolve_manifest_for_client(req: &ManifestQuery, rollout: Option<&Rollout>) -> ManifestVersion {
    match rollout {
        None => latest_stable_manifest_version(),  // 无灰度批次进行中: 直接返回最新稳定版
        Some(ro) if ro.status == RolloutStatus::已回滚 => ro.stable_manifest_version,
        Some(ro) => {
            let identity = req.player_id.clone().unwrap_or_else(|| req.device_id.clone());  // §3既定: player_id优先,缺省用device_id
            let bucket = bucket_for(&identity);
            if (bucket as i16) < ro.percentage as i16 {
                ro.target_manifest_version
            } else {
                ro.stable_manifest_version
            }
        }
    }
}
```

分桶结果的确定性（FR-CDN-022）由`bucket_for`是纯函数（相同`identity`输入恒定输出相同`bucket`）保证，`ManifestService`本身不缓存/记录每个客户端的分桶结果——不需要，因为函数天然可复现，缓存反而引入"缓存与实时计算结果不一致"的额外风险，故本文档明确**不**引入分桶结果的持久化缓存层。

---

## 6. 经批准分发后端具体部署形态

对应RGS-BAS-027§6.2组件框图，落实为经批准后端契约与候选实现（供TBD-CDN-002评审参考，候选不是生产结论）。

### 6.1 组件选型提案

| 组件 | 候选实现 | 许可／审核状态 | 备注 |
|---|---|---|---|
| 自托管对象存储 | MinIO（S3兼容API）；若其许可证评审不通过，可候选 Ceph RGW／Garage 等 | MinIO 的 AGPLv3 需与CON-001及附件D OSS许可一览逐项核对；**未批准** | 仅承载持久化存储层，不直接对客户端暴露 |
| 反向代理/缓存层 | Nginx（`proxy_cache`模块）或 Varnish | BSD-2-Clause / BSD-2-Clause；仍须纳入生产 BOM 与审批 profile | 具体二选一留待实现阶段，不构成默认批准 |

### 6.2 `DistributionBackend`接口的具体实现

```rust
struct SelfHostedBackend {
    object_store_endpoint: String,   // 已批准的 S3 兼容源站 endpoint
    proxy_public_base_url: String,   // 反向代理对外可访问的base URL,put/get_url均以此为准而非直连对象存储
}

impl DistributionBackend for SelfHostedBackend {
    fn put(&self, file_path: &str, bytes: &[u8]) -> Result<Url, DistError> {
        s3_compatible_put(&self.object_store_endpoint, file_path, bytes)?;  // 走S3兼容API直接写对象存储,不经反向代理(写路径无需缓存)
        Ok(self.get_url(file_path)?)
    }
    fn get_url(&self, file_path: &str) -> Result<Url, DistError> {
        Ok(format!("{}/{}", self.proxy_public_base_url, file_path).parse()?)  // 读路径始终经反向代理,享受边缘缓存
    }
    fn exists(&self, file_path: &str) -> Result<bool, DistError> {
        s3_compatible_head(&self.object_store_endpoint, file_path)
    }
}
```

`put`直连对象存储、`get_url`固定返回反向代理URL——这一读写路径分离的设计对应RGS-BAS-027§6.2框图"写入不经缓存层，读取经缓存层"的隐含要求（框图未显式说明但从组件连接关系可推导，本文档在此明确其为有意设计而非疏漏）。

### 6.3 反向代理缓存策略配置骨架

```nginx
# 对应§6.2反向代理/缓存层,示例为Nginx方案
location /v1/assets/ {
    proxy_cache asset_cache;
    proxy_cache_valid 200 30d;          # 资源文件一经发布内容不可变(每个file_version对应固定checksum),可长期缓存
    proxy_cache_key $uri;
    proxy_pass http://approved_origin_upstream;
}
```

不可变资源文件可设置较长周期，因为§2.2数据模型中同一`(file_path, file_version)`组合的`checksum`不可变（新内容总是产生新`file_version`）。manifest 与 patch 则使用 RGS-REQ-030-ADD1 FR-CDN-030 规定的 5 分钟 TTL，并同样经 `approved_origin_upstream` 回源。

### 6.4 RGS-REQ-030-ADD1：`ApprovedBackendProfile` 启用门禁

`DistributionBackend` 只描述读写能力；实际生产激活还必须引用经批准的 profile，避免候选组件因出现在文档或代码中而被误当作默认后端：

```rust
struct ApprovedBackendProfile {
    backend_id: String,
    backend_kind: BackendKind,       // SelfHosted | Commercial
    bom_ref: String,
    license_or_contract_review_ref: String,
    adr_ref: String,
    allowed_regions: Vec<String>,
    status: ApprovalStatus,          // Approved 才可激活
}

fn activate_backend(profile: &ApprovedBackendProfile) -> Result<(), ActivationError> {
    require_non_empty(&profile.bom_ref)?;
    require_non_empty(&profile.license_or_contract_review_ref)?;
    require_non_empty(&profile.adr_ref)?;
    require_eq(profile.status, ApprovalStatus::Approved)?;
    Ok(())
}
```

部署控制面在创建边缘缓存或切换源站前调用 `activate_backend`，缺少任一引用、审批状态不是 `Approved`、或区域不在 `allowed_regions` 时必须拒绝部署并记录审计事件。自托管和商业 CDN 都实现相同的 `DistributionBackend`/profile 契约；MinIO、Nginx、Varnish 等候选只有在生成有效 profile 后才可被使用。边缘 miss 必须依 profile 解析源站，源站失败时按 FR-CDN-032 返回上一稳定版本，而不是隐式回退至未批准候选。

---

## 7. 本文档的覆盖范围与后续计划

本文档覆盖：`asset_db`三表（`asset_manifests`/`asset_file_entries`/`rollouts`）物理DDL、清单查询接口的HTTP+JSON具体协议格式、清单签名验证与文件完整性校验的完整伪代码、确定性分桶算法的具体哈希复用方式与`ManifestService`响应生成逻辑、经批准 `DistributionBackend` 的候选实现、`ApprovedBackendProfile` 启用门禁与边缘缓存策略配置示例。

本版本明确不覆盖、留待后续：

- TBD-CDN-002的最终组件选型评审结论——本文档§6.1的MinIO/Nginx/Varnish是**候选**，非结论；任何候选均须按RGS-BAS-027§6.2交叉核对附件D OSS许可一览，并形成 BOM、许可证/商业条款审查及 ADR 后才可生成 `ApprovedBackendProfile`。
- 商业CDN可选实现的具体接入代码——按RGS-BAS-027§6.3与RSK-CDN-001既定，需专项评审并记录ADR后才启动，本文档不预先设计。
- 发布流水线（增量补丁生成的CI触发部分，RGS-BAS-027§3.1流程图）的完整CI脚本——复用RGS-DTL-002§4已确立的CI阶段模式与GitHub Actions写法，本文档不重复展开，仅新增该流水线特有的"二进制差分计算"阶段（可作为RGS-DTL-002 CI模板中`build-and-push`前的新增阶段），具体脚本留待实现阶段补充。
- 发布高峰容量弹性规划的具体预留策略数值——RGS-BAS-027已标注为RSK-CDN-002留待详细设计阶段结合发布高峰期实测数据确定，本文档同样不给出具体数值，因尚无实测数据支撑。

后续详细设计建议顺序：至此，四个已完成基本设计的新增业务域（ANT/MM/CDN）与挂载脚手架（RGS-DTL-002）均已有对应详细设计文档，均验证了RGS-DTL-002模板的可复用性；建议下一步转向RGS-DTL-001遗留的核心架构剩余部分（match_db／social_db／admin_db物理设计、MatchService/SocialService/AdminService协议细节），因ANT（RGS-DTL-025）与MM（RGS-DTL-026）均已提前引用`admin_db`/`match_db`的部分表结构，若核心架构自身的DTL-001不尽快补齐，将出现"业务域DTL引用的库由谁最终定义全貌"的文档权责模糊风险。

---

## 追溯性

| 需求/设计来源 | 本文档章节 |
|---|---|
| RGS-BAS-027§2.1 组件定位 | §2、§3 |
| RGS-BAS-027§2.2 数据模型 | §2 |
| RGS-BAS-027§2.3 差异计算 | §3 |
| RGS-BAS-027§2.4 匿名访问 | §3 |
| RGS-BAS-027§3.1 增量补丁流水线 | §7（明确排除CI脚本，声明复用RGS-DTL-002） |
| RGS-BAS-027§3.2 客户端侧应用逻辑 | §4.2 |
| RGS-BAS-027§4.1〜4.3 完整性校验与签名 | §4 |
| RGS-BAS-027§5.1〜5.2 确定性分桶与灰度控制 | §2（`rollouts`表）、§5 |
| RGS-BAS-027§5.3 与协议版本协商衔接 | §3（响应字段`is_forced_update`/`min_supported_client_version`） |
| RGS-BAS-027§6.1〜6.2 分发后端接口与既有默认方向（本版以审批 profile 约束） | §6 |
| RGS-BAS-027§6.3 可选商业CDN实现 | §7（明确排除） |
| RGS-DTL-002（挂载脚手架物理落地） | 前提依赖，本文档假定`asset_db`已按RGS-DTL-002完成挂载 |
| RGS-REQ-030-ADD1 FR-CDN-030〜035、NFR-CDN-101〜105 | §6.3〜§6.4 |
| RGS-REQ-030-ADD1 AC-CDN-101〜106 | §6.3〜§6.4 |
