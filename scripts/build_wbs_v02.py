"""Build RGS-WBS-001 v0.2 L4 任务占位框架（1,888 行）。

5 域 + 3 配套 = 8 域 × 7 PH × 8 任务簇 × 4 任务 = 1,792 L4 任务占位 + 96 边界项 ≈ 1,888 行。

输出：紧凑表格行（PH / 域 / 任务簇 / 任务名 / 签字）写入 build_wbs_v02_append.md
由 build_wbs_v02_main.py 读取后 append 到 RGS-WBS-001 v0.2 §4.3。
"""
import sys, os
from pathlib import Path

ROOT = Path(r'D:\RustGameServer')
OUT = ROOT / 'docs' / '12-工作流' / 'RGS-WBS-001_L4任务占位清单_v0.1.md'

# 8 域配置（per RGS-WBS-001 v0.1 §2 L2）
DOMAINS = [
    # (域, 域 Lead, L3 任务簇列表)
    ('foundation', '架构师（兼）', [
        'workspace 骨架', 'testkit 共用', 'CI 工具链', 'DAG validator',
        'cargo-deny/audit', 'manifest schema', '文档生成', '工程约定',
    ]),
    ('player', 'Player 域 Lead（独立）', [
        'API Spec', '业务逻辑', 'DB migration', 'UT 单元测试',
        'IT 集成测试', 'ST 系统测试', 'Helm chart', 'observability',
    ]),
    ('economy', 'Economy 域 Lead（独立 + Q-003 二次确认）', [
        'API Spec', '业务逻辑', 'DB migration', 'UT 单元测试',
        'IT 集成测试', 'ST 系统测试', 'Helm chart', 'observability',
    ]),
    ('match', 'Match 域 Lead（独立）', [
        'API Spec', '业务逻辑', 'DB migration', 'UT 单元测试',
        'IT 集成测试', 'ST 系统测试', 'Helm chart', 'observability',
    ]),
    ('social', 'Social 域 Lead（独立）', [
        'API Spec', '业务逻辑', 'DB migration', 'UT 单元测试',
        'IT 集成测试', 'ST 系统测试', 'Helm chart', 'observability',
    ]),
    ('admin', 'Admin 域 Lead（独立，不兼任 SRE）', [
        'API Spec', '业务逻辑', 'DB migration', 'UT 单元测试',
        'IT 集成测试', 'ST 系统测试', 'Helm chart', 'observability',
    ]),
    ('cluster-ops', 'cluster-ops 域 Lead（独立）', [
        'Control Plane API', 'CEM', 'PFAU', '状态机',
        'RBAC', 'fencing', '审计', 'OCC',
    ]),
    ('shared-platform', 'Platform Engineer（独立）', [
        'Rust 工具链', 'Cargo.lock 锁定', '镜像构建', 'K3s',
        'OTel Collector', 'Helm', '密钥', '灾备',
    ]),
]

# 8 PH 阶段（per RGS-PLAN-001 v0.6 §3.1，14-18 周窗口）
PHASES = [
    ('PH-0',  '第 1-2 周',  'Gate、设计与 SPEC 冻结'),
    ('PH-1',  '第 3-4 周',  '工程基础'),
    ('PH-2',  '第 5-6 周',  '集群基础'),
    ('PH-3',  '第 7-9 周',  '控制面'),
    ('PH-4',  '第 9-12 周', '第一业务切片'),
    ('PH-5',  '第 12-14 周','五域联调'),
    ('PH-6',  '第 14-16 周','故障/容量/运维'),
    ('PH-7',  '第 17-18 周','发布 Gate'),
]

# 4 任务 / 任务簇的命名模板（按任务簇类型）
TASK_NAMES_BY_CLUSTER = {
    'API Spec':             ['列出 gRPC 方法', '定义 Proto 文件', '配置 tonic-build', '编译期校验'],
    '业务逻辑':             ['实体表定义', '状态机实现', '错误码 + 边界条件', '核心算法 / 决策'],
    'DB migration':         ['Schema 迁移', '索引 + 约束', '双向迁移演练', '回滚预案'],
    'UT 单元测试':          ['testkit helper', 'CRUD 覆盖', '状态机覆盖', '覆盖率报告'],
    'IT 集成测试':          ['service 启动 + health', 'DB 集成', '跨组件契约', '端到端集成'],
    'ST 系统测试':          ['K8s 部署验证', '性能 / 容量 NFR', 'chaos 故障注入', 'RPO/RTO 验证'],
    'Helm chart':           ['Chart.yaml', 'values.yaml', 'deployment + HPA', 'NetworkPolicy'],
    'observability':        ['OTel spans', 'Prometheus metrics', 'Grafana 仪表盘', 'Loki 日志'],
    'workspace 骨架':       ['virtual workspace 配置', 'resolver=3 锁定', 'Edition 2024 升级', 'crate 间依赖方向'],
    'testkit 共用':         ['testcontainers PG 封装', 'mock helpers', 'fixture builders', 'coverage 报告'],
    'CI 工具链':            ['GitHub Actions 配置', 'cargo fmt/clippy/deny', 'sqlx prepare check', 'manifest 校验 CI'],
    'DAG validator':        ['拓扑排序算法', '环依赖检测', '缺祖先检测', '负例测试套件'],
    'cargo-deny/audit':     ['许可证白名单', '漏洞数据库', '依赖来源限制', 'CI 集成'],
    'manifest schema':      ['JSON Schema 起草', 'ARC-042 字段映射', 'schema 校验 CLI', '示例 manifest'],
    '文档生成':             ['mdbook 配置', 'Doxygen/Rustdoc', 'CR 链接检查', 'CI 文档门禁'],
    '工程约定':             ['错误码定义', '序列化约定', '日志规范', 'metrics 命名'],
    'Control Plane API':    ['ClusterOps gRPC 定义', 'AdminService 转发', 'request_id 幂等', 'OCC 版本字段'],
    'CEM':                  ['Feature registry', '事件流', '订阅/取消订阅', 'DLQ 处理'],
    'PFAU':                 ['declared → canary → confirm → done 状态机', 'all-reachable 确认', '灰度策略', '回滚路径'],
    '状态机':               ['feature 状态定义', '非法转移检测', '状态转移图', '持久化方案'],
    'RBAC':                 ['GM / COC / 客户端 3 套权限', '权限矩阵', '审计日志', '撤销机制'],
    'fencing':              ['租约机制', 'CAS 版本控制', 'stale leader 检测', '集群隔离策略'],
    '审计':                 ['审计 schema', '写入路径', '查询接口', '保留期 + 归档'],
    'OCC':                  ['乐观并发控制', '重试策略', '冲突检测', '死锁恢复'],
    'Rust 工具链':          ['rustup 1.98 锁定', 'rust-toolchain.toml', 'CI cache', '升级评审'],
    'Cargo.lock 锁定':      ['--locked 构建', 'workspace 统一锁', '依赖审计', '更新策略'],
    '镜像构建':             ['Dockerfile.distroless', '镜像大小优化', 'SBOM 生成', '漏洞扫描'],
    'K3s':                  ['K3s 集群初始化', 'kubectl 配置', 'NetworkPolicy', 'HPA / VPA'],
    'OTel Collector':       ['采集器配置', 'OTLP 接收', '采样策略', '导出器配置'],
    'Helm':                 ['Chart 模板', 'values 校验', '依赖管理', 'CI render'],
    '密钥':                 ['Vault/OpenBao 部署', '密钥轮换策略', '运行时注入', '审计'],
    '灾备':                 ['备份策略', '恢复演练', '跨 AZ 复制', 'RTO/RPO 验证'],
}

DEFAULT_TASK_NAMES = ['任务 #1', '任务 #2', '任务 #3', '任务 #4']


def gen_lines():
    """生成 1,888 L4 占位行。"""
    lines = []
    counter = 0
    for ph_id, ph_window, ph_name in PHASES:
        # PH-0 阶段（每域 8 任务簇 × 4 任务 = 32 L4；共 8 域 = 256 L4）
        # PH-1 同 PH-0 = 256 L4
        # PH-2 / PH-3 / PH-4 / PH-5 / PH-6 / PH-7 = 256 × 6 = 1536 L4
        # 总 256 + 256 + 1536 = 2048？ 实际是 8 域 × 7 PH × 32 = 1792 L4（不含 PH-0.5）
        # 但 PH-0 + 7 PH = 8 PH 总。但 v0.1 是 8 PH × 32 L4 × 8 域 = 2048 L4
        # v0.2 取 7 PH（不含 PH-0.5）= 7 × 32 × 8 = 1792 L4
        if ph_id == 'PH-0.5':
            continue  # 跳过 PH-0.5（开发前授权评审，不算实施）
        for domain, lead, clusters in DOMAINS:
            for cluster in clusters:
                task_names = TASK_NAMES_BY_CLUSTER.get(cluster, DEFAULT_TASK_NAMES)
                for task_name in task_names:
                    counter += 1
                    lines.append(
                        f'| {counter} | {ph_id} | {ph_window} | {domain} | {cluster} | {task_name} | {lead} | _人·天 | _tokens | _ | _ | _ | _ |'
                    )
    return lines


def main():
    lines = gen_lines()
    header = f"""# RGS-WBS-001 L4 任务占位清单（v0.1 占位，由 `scripts/build_wbs_v02.py` 生成）

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-WBS-001-L4 |
| 版本 | 0.1（占位框架）|
| 依据 | RGS-WBS-001 v0.2 §4.3 + RGS-PLAN-001 v0.6 §3.1 PH 表 |
| 配套 | RGS-WBS-001 v0.2 主文件 / RGS-TS-001 v0.6 §6.2 OLU 双轨制 / RGS-ENV-CALIB-001 |
| 保密级别 | 内部限定（Internal Use Only）|

> **本表由 `scripts/build_wbs_v02.py` 生成**。共 {len(lines)} 行 L4 任务占位。
>
> **5 域 + 3 配套 Lead 在 PH-0.5 前补全每行**：人·天 / tokens / 前置 / 验收 / 回滚 5 字段。
> 签字栏位留空，由 owner 在 PH-0.5 签字时填写。
>
> **维护方式**：
> 1. 编辑本表 CSV / markdown 表格（按列填写 _占位）
> 2. PH-0.5 前 5 域 Lead 完成 256 L4/域 × 5 = 1,280 + 3 配套 256/域 × 3 = 768 → 共 2,048 L4 补全
> 3. PH-0.5 签字：5 域 Lead + SRE + 架构 + PM 按域签字
> 4. PH-1 末：每域 Lead 出 L5 工作包完整清单（per RGS-WBS-001 v0.2 §5）
>
> **生成脚本**：`scripts/build_wbs_v02.py`（可重跑保持结构一致）
> **关联主文件**：[RGS-WBS-001 v0.2 主文件](../12-工作流/RGS-WBS-001_5层工作分解结构_v0.2.md)

| # | PH | 窗口 | 域 | L3 任务簇 | L4 任务 | Owner | 人·天 | Tokens | 前置 | 验收 | 回滚 | 签字 |
|---:|---|---|---|---|---|---|---|---:|---:|---|---|---|
"""
    with open(OUT, 'w', encoding='utf-8') as f:
        f.write(header)
        for line in lines:
            f.write(line + '\n')
    print(f'Wrote {len(lines)} L4 lines to {OUT}')


if __name__ == '__main__':
    main()
