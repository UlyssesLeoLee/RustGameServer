# Particle API Browser

> 一个常驻后台的纯前端工具页，聚合 **Unity / Unreal / Godot / Physis** 四个引擎的粒子 API，支持选项卡分类浏览 + 弹窗详情 + 跨引擎示例对比。

## 用途

- 客户端主程跨引擎移植粒子系统时，对照同一概念在各引擎下的语义差异
- 特效美术查阅参数含义、取值范围、引擎标签
- 自研引擎对接时查 `gvpe_particle_*` C ABI 签名 + 各引擎调用样例
- 新人入职时了解 RGS 客户端粒子技术栈

## 目录结构

```
particle-api-browser/
├── docs/                三份设计文档（需求 / 基本 / 详细）
├── data/                API 数据 JSON（v0.1 落盘 8 条骨架）
├── src/                 前端模块（v0.2 实现）
├── index.html           入口（v0.2 实现）
├── styles.css           样式（v0.2 实现）
└── README.md            本文件
```

## 当前状态

| 项 | 状态 |
|----|------|
| 需求定义书 | ✅ v0.1 完成 |
| 基本设计书 | ✅ v0.1 完成 |
| 详细设计书 | ✅ v0.1 完成 |
| 前端实现 | ⏳ 待 v0.2 开发（设计已完整，代码骨架可按详细设计书 §2 直接照搬） |
| 初始 8 条 API 数据 | ⏳ 待录入（schema 已定义，样例见详细设计书 §5） |

## 启动方式（实现完成后）

```bash
# 方式 1：复用 rgs-web dev server
cd D:/RustGameServer/tools/rgs-web
node server.js
# 打开 http://localhost:<port>/particle-api-browser/

# 方式 2：Python 内置服务
cd D:/RustGameServer/tools/rgs-web/particle-api-browser
python -m http.server 8080
# 打开 http://localhost:8080/

# 方式 3：npx
npx http-server -p 8080 -c-1
```

> ⚠️ **不要双击 `index.html` 直接打开**：`fetch('data/api.json')` 在 `file://` 协议下会被浏览器 CORS 拒绝。必须通过 HTTP 服务访问。

## 数据维护

- 所有 API 信息存在 `data/api.json`（UTF-8 无 BOM）
- 类别配置在 `data/categories.json`
- UI 文案在 `data/i18n.json`
- 添加新 API：编辑 `api.json`，无需重新构建（刷新页面即可）

## 文档索引

| 文档 | 内容 |
|------|------|
| [docs/01-需求定义书.md](./docs/01-需求定义书.md) | FR/NFR/AC 完整需求 |
| [docs/02-基本设计书.md](./docs/02-基本设计书.md) | 架构、信息架构、交互流 |
| [docs/03-详细设计书.md](./docs/03-详细设计书.md) | 数据 schema、组件实现、样式 |

## 文档治理

- 工具编号：`DEV-PAB-001`（Developer Tool）
- **不进入** RGS 三层治理体系（REQ / BAS / DTL）
- **不进入** RGS 5 域限界上下文
- 属于开发者工具，与 `tools/db-seed`、`tools/rgs-web` 平级
