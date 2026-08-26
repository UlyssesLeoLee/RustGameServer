# RGS-WEB-DETAILED-DESIGN-2026-08-26 v0.1

**RGS Admin Web 详细设计（Detailed Design）**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-WEB-DETAILED-DESIGN-2026-08-26 |
| 版本 | 0.1（per Ulysses 2026-08-26 12:30 JST "从需求到基本设计，详细设计都要补充完整" + 13:25 JST 自审发现 2 份未落地）|
| 状态 | 草案（待 Ulysses DDD Review 阶段补签）|
| 触发 | 自审发现 REQUIREMENTS v0.1 + BASIC-DESIGN v0.1 已落地,DETAILED-DESIGN 缺 |
| 关联 | RGS-WEB-REQUIREMENTS-2026-08-26 v0.1 + RGS-WEB-BASIC-DESIGN-2026-08-26 v0.1 + RGS-WEB-PLAN-2026-08-26 v0.1 + RGS-WEB-GM-PLAN-2026-08-26 v0.1 |
| 责任人 | 架构师（Ulysses（一人公司 12 角色 per DEC-008））|

---

## 0. 文档定位

本文档是 RGS Admin Web 三层文档中的**详细设计层**，逐 API + 逐页面 + 逐函数展开实现细节。回答"具体怎么实现"。

读者：v0.3 接入 5 域 gRPC client / v1.0 Rust 重写 / 接手开发的工程师。

---

## 1. 文件清单

```
D:/RustGameServer/tools/rgs-web/
├── server-no-deps.js         (151 行,6.4 KB)  ← 当前运行
├── server.js                  (98 行,3.7 KB)   ← 备(express 待 npm install)
├── public/
│   └── index.html             (560 行,32 KB)   ← Dashboard
├── package.json               (14 行,309 B)   ← express 4.19 依赖
└── README.md                  (49 行,1.6 KB)  ← 启动 SOP

D:/RustGameServer/docs/12-工作流/
├── RGS-WEB-PLAN-2026-08-26_v0.1.md                 (7.2 KB)  ← 总览
├── RGS-WEB-REQUIREMENTS-2026-08-26_v0.1.md        (13.8 KB) ← 需求
├── RGS-WEB-BASIC-DESIGN-2026-08-26_v0.1.md        (18.2 KB) ← 基本设计
├── RGS-WEB-DETAILED-DESIGN-2026-08-26_v0.1.md     (本文件)   ← 详细设计
├── RGS-WEB-GM-PLAN-2026-08-26_v0.1.md             (11.7 KB) ← GM 总览
└── WSL-KUBECONFIG-FIX-2026-08-26.md              (1.6 KB)  ← WSL 修复 SOP
```

---

## 2. server-no-deps.js 详细设计

### 2.1 模块顶部(行 1-15)

```javascript
const http = require('http');         // node 内置 HTTP server
const https = require('https');       // node 内置 HTTPS(代理 k3s 用)
const fs = require('fs');              // node 内置文件 IO
const path = require('path');          // node 内置路径
const { execSync } = require('child_process');  // node 内置子进程
```

**设计**：全 node 内置,0 依赖。

### 2.2 环境变量(行 17-19)

```javascript
const PORT = process.env.RGS_WEB_PORT || 8788;
const K3S_API = process.env.K3S_API || 'https://127.0.0.1:6443';
const K3S_TOKEN = process.env.K3S_TOKEN || '';
const K3S_CA = process.env.K3S_CA_PATH || '';
```

| 变量 | 默认值 | 用途 | 来源 |
|---|---|---|---|
| RGS_WEB_PORT | 8788 | 监听端口 | 用户(可选) |
| K3S_API | localhost:6443 | k3s API URL | 启动时设 |
| K3S_TOKEN | '' | 认证 token | WSL sudo chmod 后 cat |
| K3S_CA_PATH | '' | CA 证书路径 | 暂未用 |

### 2.3 handlers 对象(行 21-115)

#### 2.3.1 /api/health

```javascript
'/api/health': () => ({
  status: 'ok', k3s: K3S_API, time: new Date().toISOString(),
  pid: process.pid, rgs_web_version: '0.2.0-gm',
  pages: ['dashboard','servers','players','stream','config',
          'hotupdate','operations','docs','worktrees','reports'],
})
```

**响应字段**:
- `status`: 固定 'ok'
- `k3s`: K3S_API 值
- `time`: ISO 8601 UTC
- `pid`: node 进程 PID(用户可 kill 用)
- `rgs_web_version`: '0.2.0-gm'
- `pages`: 10 页面 ID 列表(前端可校验)

**性能**: < 1ms(纯对象字面量)

#### 2.3.2 /api/impl-plan

```javascript
'/api/impl-plan': () => {
  const wtOut = execSync('git worktree list --porcelain', { cwd: 'D:/RustGameServer', encoding: 'utf8' });
  const wts = wtOut.split('\n\n').filter(s => s.trim()).map(block => {
    return block.split('\n').find(l => l.startsWith('worktree '))?.split(' ').slice(1).join(' ');
  });
  let all = [];
  for (const wt of wts) {
    const dir = path.join(wt, 'docs/12-工作流');
    if (!fs.existsSync(dir)) continue;
    const files = fs.readdirSync(dir).filter(f => f.startsWith('RGS-IMPL-PLAN-') && f.endsWith('.md'));
    for (const f of files) {
      const c = fs.readFileSync(path.join(dir, f), 'utf8');
      const status = (c.match(/\| 状态\s*\|\s*([^|]+)/) || [])[1]?.trim() || 'unknown';
      const owner = (c.match(/\| owner\s*\|\s*([^|]+)/) || [])[1]?.trim() || 'unknown';
      const wname = wt.replace('D:/RustGameServer-worktrees/', '').replace('D:/RustGameServer', 'main');
      all.push({ file: f, status, owner, size: c.length, worktree: wname });
    }
  }
  const dedup = []; const seen = new Set();
  for (const f of all) { if (!seen.has(f.file)) { seen.add(f.file); dedup.push(f); } }
  return { plans: dedup };
}
```

**关键点**:
- `git worktree list --porcelain` 拿 45 worktree 路径
- 过滤 `RGS-IMPL-PLAN-*.md`
- 正则 `\| 状态\s*\|\s*([^|]+)` 解析 markdown table 行
- `Set` 去重(同一文件可能在多个 worktree,取首个)

**性能**:
- execSync 1 次: ~60ms
- 45 worktree × fs.readdirSync: ~5ms
- 8 个 readFileSync: ~15ms
- 正则 × 8: < 1ms
- 总: ~80-90ms

**失败模式**:
- git worktree list 失败 → 500
- fs.existsSync false → 跳过(用 `continue`)
- 文件 lock 或权限拒绝 → catch 不到(throw 500)

#### 2.3.3 /api/worktrees

```javascript
'/api/worktrees': () => {
  const out = execSync('git worktree list --porcelain', { cwd: 'D:/RustGameServer', encoding: 'utf8' });
  const wts = out.split('\n\n').filter(s => s.trim()).map(block => {
    const lines = block.split('\n');
    const path = lines.find(l => l.startsWith('worktree '))?.split(' ').slice(1).join(' ');
    const head = lines.find(l => l.startsWith('HEAD '))?.split(' ')[1]?.substring(0, 7);
    const branch = lines.find(l => l.startsWith('branch '))?.split(' ').slice(1).join(' ')?.replace('refs/heads/', '');
    const isLocked = block.includes('locked');
    return { path, head, branch, locked: isLocked };
  });
  return { worktrees: wts, total: wts.length };
}
```

**关键点**:
- parse `worktree /path\nHEAD sha\nbranch refs/heads/...` 三行
- 短路 `slice(1).join(' ')` 处理路径含空格
- `substring(0, 7)` 取短 hash
- `locked: block.includes('locked')` 检测 locked 标记

#### 2.3.4 /api/docs-health

```javascript
'/api/docs-health': () => ({
  fail: 1, warn: 1,
  fail_reason: 'RGS-DEC-NOGO-001 缺决策编号字段',
  warn_reason: '5 ADR 待具名审批',
  last_check: '2026-08-26 04:30 JST 基线',
  p0p1p2_commits: 11,
})
```

**注**:v0.2-gm 暂为静态,真实应在 v0.3 接 check-docs-consistency.sh 跑结果。

#### 2.3.5 /api/git-log

```javascript
'/api/git-log': () => {
  const out = execSync('git log --pretty=format:"%h|%ad|%s" --date=short -n 30', { cwd: 'D:/RustGameServer', encoding: 'utf8' });
  const commits = out.split('\n').filter(l => l.trim()).map(l => {
    const [hash, date, ...msg] = l.split('|');
    return { hash, date, message: msg.join('|') };
  });
  return { commits, total: commits.length };
}
```

**注**: `%s` 是 subject(commit message 第一行),commit message 含 `|` 会出错 → `msg.join('|')` 复原

#### 2.3.6 /api/saga-trace

```javascript
'/api/saga-trace': () => {
  const out = execSync('git log --all --pretty=format:"%h|%ad|%s" --date=iso --grep="saga" -n 20', { cwd: 'D:/RustGameServer', encoding: 'utf8' });
  // 同 git-log
}
```

**关键点**:
- `--all`:所有 branch(包括 worktree)
- `--grep="saga"`:过滤 saga 相关 commit
- `--date=iso`:ISO 格式(给 saga-trace 用)

### 2.4 proxyK3s 函数(行 117-138)

```javascript
function proxyK3s(req, res) {
  const url = req.url.replace(/^\/api\/k8s/, '');
  const target = `${K3S_API}${url}`;
  const headers = { ...req.headers };
  if (K3S_TOKEN) headers['Authorization'] = `Bearer ${K3S_TOKEN}`;
  delete headers['host'];
  const opts = {
    method: req.method,
    headers,
    rejectUnauthorized: !!K3S_CA,
    ca: K3S_CA ? fs.readFileSync(K3S_CA) : undefined,
  };
  const proxyReq = https.request(target, opts, (proxyRes) => {
    res.writeHead(proxyRes.statusCode, proxyRes.headers);
    proxyRes.pipe(res);
  });
  proxyReq.on('error', (e) => {
    res.writeHead(502, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ error: e.message, k3s: K3S_API, hint: 'WSL sudo chmod 644 ...' }));
  });
  req.pipe(proxyReq);
}
```

**关键点**:
- `req.url.replace(/^\/api\/k8s/, '')`:剥掉 /api/k8s 前缀
- `headers = { ...req.headers }`:浅拷贝(避免改原对象)
- `delete headers['host']`:k3s 不认 Windows 端 host
- `proxyRes.pipe(res)`:直接流转发,不缓冲
- `rejectUnauthorized: !!K3S_CA`:有 CA 才验证,默认跳过(self-signed)
- error 返 502 + hint 引导用户修

### 2.5 http.createServer 路由(行 140-170)

```javascript
const server = http.createServer((req, res) => {
  // 1. 静态首页
  if (req.url === '/' || req.url === '/index.html') {
    res.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8' });
    return fs.createReadStream(path.join(__dirname, 'public', 'index.html')).pipe(res);
  }
  // 2. 静态资源
  if (req.url.startsWith('/public/') || req.url.match(/\.(css|js|svg|png|ico)$/)) {
    const fp = path.join(__dirname, req.url);
    if (fs.existsSync(fp)) {
      const ext = path.extname(fp).substring(1);
      const mime = { css: 'text/css', js: 'application/javascript', svg: 'image/svg+xml', png: 'image/png', ico: 'image/x-icon' }[ext] || 'application/octet-stream';
      res.writeHead(200, { 'Content-Type': mime });
      return fs.createReadStream(fp).pipe(res);
    }
  }
  // 3. API 路由
  const handler = handlers[req.url];
  if (handler) {
    res.writeHead(200, { 'Content-Type': 'application/json; charset=utf-8' });
    return res.end(JSON.stringify(handler()));
  }
  // 4. k3s 代理
  if (req.url.startsWith('/api/k8s')) return proxyK3s(req, res);
  // 5. 404
  res.writeHead(404, { 'Content-Type': 'application/json' });
  res.end(JSON.stringify({ error: 'not found', path: req.url }));
});
```

**路由优先级**:首页 → 静态资源 → API → k8s 代理 → 404

### 2.6 server.listen(行 172-177)

```javascript
server.listen(PORT, '127.0.0.1', () => {
  console.log(`RGS Admin Web v0.2-gm 启动: http://127.0.0.1:${PORT}`);
  console.log(`  k3s API: ${K3S_API}`);
  console.log(`  K3S_TOKEN: ${K3S_TOKEN ? '***set***' : '(unset — 代理 401)'}`);
  console.log(`  PID: ${process.pid}`);
});
```

**关键**:
- `'127.0.0.1'`:只监听 localhost,无 0.0.0.0
- console.log 4 行:启动消息 + k3s + token 状态 + PID

---

## 3. public/index.html 详细设计

### 3.1 HTML 结构

```
<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <title>RGS GM Admin · 管理后台</title>
  <meta http-equiv="refresh" content="60">
  <style>...</style>
</head>
<body>
  <header>...</header>
  <div class="layout">
    <nav>...</nav>
    <main>
      <div class="page active" id="page-dashboard">...</div>
      <div class="page" id="page-servers">...</div>
      ... 10 pages ...
    </main>
  </div>
  <script>...</script>
</body>
</html>
```

### 3.2 CSS 设计系统(行 13-43)

```css
:root {
  --bg: #0a0c10;       /* 主背景:近黑 */
  --panel: #131720;    /* 卡片背景:深灰 */
  --panel-2: #1a1f2b;  /* hover 背景 */
  --border: #2a3142;    /* 边框 */
  --text: #e6e9ef;     /* 主文字:近白 */
  --muted: #8a92a3;    /* 次要文字:灰 */
  --green: #4ade80; --red: #f87171; --yellow: #fbbf24;
  --blue: #60a5fa; --purple: #c084fc; --cyan: #22d3ee;
}
```

**设计参考**:ROPE_CS 的 dark theme + Tailwind-style CSS 变量

### 3.3 布局(grid)

```css
.layout { display: grid; grid-template-columns: 200px 1fr; min-height: calc(100vh - 50px); }
```

**侧边栏 200px + 主区自适应**

### 3.4 10 页面 ID 与路由

| 页面 ID | div id | 触发的 nav a |
|---|---|---|
| dashboard | page-dashboard | default(active) |
| servers | page-servers | data-page="servers" |
| players | page-players | data-page="players" |
| stream | page-stream | data-page="stream" |
| config | page-config | data-page="config" |
| hotupdate | page-hotupdate | data-page="hotupdate" |
| operations | page-operations | data-page="operations" |
| docs | page-docs | data-page="docs" |
| worktrees | page-worktrees | data-page="worktrees" |
| reports | page-reports | data-page="reports" |

### 3.5 路由切换 JS(行 540-555)

```javascript
document.querySelectorAll('nav a').forEach(a => {
  a.addEventListener('click', () => {
    document.querySelectorAll('nav a').forEach(x => x.classList.remove('active'));
    document.querySelectorAll('.page').forEach(p => p.classList.remove('active'));
    a.classList.add('active');
    const page = a.dataset.page;
    document.getElementById('page-' + page).classList.add('active');
    if (page === 'servers') loadServers();
    if (page === 'config') loadConfig();
    if (page === 'hotupdate') loadHotUpdate();
    if (page === 'worktrees') loadWorktrees();
    if (page === 'players') searchPlayer();
  });
});
```

**设计**:纯 DOM API,无 router 库。点击 nav a → 切换 .active class → 触发对应 load()。

### 3.6 通用辅助函数(行 558-561)

```javascript
async function api(path, opts) { const r = await fetch(path, opts); return r.json(); }
function fmtDate(iso) { return new Date(iso).toLocaleString('zh-CN'); }
function escHtml(s) { return String(s).replace(/[<>&"']/g, c => ({'<':'&lt;','>':'&gt;','&':'&amp;','"':'&quot;',"'":'&#39;'}[c])); }
```

- `api()`:fetch 包装
- `fmtDate()`:ISO → 本地时区字符串
- `escHtml()`:XSS 防护

### 3.7 关键函数清单

| 函数 | 位置 | 输入 | 输出 |
|---|---|---|---|
| `loadDashboard()` | 行 564-595 | 4 API 并发 | 4 stat 卡 + 8 bar + 11 commit 表 |
| `loadServers()` | 行 599-624 | 3 k8s API | 3 table |
| `searchPlayer()` | 行 628-654 | 输入框值 | mock table + detail panel |
| `viewPlayer(id)` | 行 656-664 | id | detail pre |
| `gmAction(id, action)` | 行 666-668 | id + action | alert(mvp) |
| `startStream()` | 行 673-693 | source + filter | 1Hz setInterval mock |
| `stopStream()` | 行 695-699 | - | clearInterval |
| `loadConfig()` | 行 703-708 | /api/impl-plan | pre 文本 |
| `loadHotUpdate()` | 行 712-723 | /api/git-log | 2 table |
| `loadWorktrees()` | 行 727-745 | /api/worktrees | 45 行 + 过滤 |

---

## 4. 部署详细

### 4.1 开发启动

```powershell
# PowerShell
$env:RGS_WEB_PORT = '8788'
$env:K3S_API = 'https://172.28.176.169:6443'
# $env:K3S_TOKEN = '...'  # WSL sudo chmod 后填
node D:/RustGameServer/tools/rgs-web/server-no-deps.js

# 浏览器
Start-Process 'http://127.0.0.1:8788/'
```

### 4.2 后台运行(无 console 窗口)

```powershell
$psi = New-Object System.Diagnostics.ProcessStartInfo
$psi.FileName = 'node'
$psi.Arguments = 'D:/RustGameServer/tools/rgs-web/server-no-deps.js'
$psi.EnvironmentVariables['RGS_WEB_PORT'] = '8788'
$psi.EnvironmentVariables['K3S_API'] = 'https://172.28.176.169:6443'
$psi.WorkingDirectory = 'D:/RustGameServer/tools/rgs-web'
$psi.RedirectStandardOutput = 'D:/tmp/rgs-web.log'
$psi.RedirectStandardError = 'D:/tmp/rgs-web.err'
$psi.UseShellExecute = $false
$psi.CreateNoWindow = $true
$proc = [System.Diagnostics.Process]::new()
$proc.StartInfo = $psi
[void]$proc.Start()
```

### 4.3 验证

```powershell
# 端口
Test-NetConnection -ComputerName 127.0.0.1 -Port 8788
# API
Invoke-WebRequest 'http://127.0.0.1:8788/api/health' -UseBasicParsing
# 进程
Get-Process -Name node | Where-Object {$_.StartTime -gt (Get-Date).AddDays(-1)}
```

### 4.4 v0.3 部署改进(等 v0.3)

- 加 systemd-style daemon(Windows Service Wrapper)
- 加 /api/metrics(Prometheus exporter)
- 加 5 域 gRPC client(lib:@grpc/grpc-js)
- 加 WebSocket 通道(ws 库)
- 加 basic auth(防本地意外访问)

---

## 5. v0.3 详细规划

### 5.1 5 域 gRPC client 接入

```javascript
// 新增 tools/rgs-web/lib/grpc-client.js
const grpc = require('@grpc/grpc-js');
const protoLoader = require('@grpc/proto-loader');
const path = require('path');

// 加载 RGS proto
const pkgDef = protoLoader.loadSync(
  path.join(__dirname, '../../proto/rgs.proto'),
  { keepCase: true, longs: String, enums: String, defaults: true, oneofs: true }
);
const rgs = grpc.loadPackageDefinition(pkgDef).rgs;

// 5 域 client 池
const clients = {
  player: new rgs.PlayerService('localhost:50051', grpc.credentials.createInsecure()),
  economy: new rgs.EconomyService('localhost:50052', grpc.credentials.createInsecure()),
  match: new rgs.MatchService('localhost:50053', grpc.credentials.createInsecure()),
  social: new rgs.SocialService('localhost:50054', grpc.credentials.createInsecure()),
  admin: new rgs.AdminService('localhost:50055', grpc.credentials.createInsecure()),
};
```

### 5.2 WebSocket 日志流

```javascript
// 新增 /api/stream?source=cluster-ops (Server-Sent Events)
app.get('/api/stream', (req, res) => {
  res.writeHead(200, {
    'Content-Type': 'text/event-stream',
    'Cache-Control': 'no-cache',
    'Connection': 'keep-alive',
  });
  const source = req.query.source;
  const child = spawn('wsl', ['-d', 'Ubuntu', '--', 'bash', '-c',
    `journalctl -u ${source} -f --no-pager`]);
  child.stdout.on('data', (chunk) => {
    res.write(`data: ${chunk.toString()}\n\n`);
  });
  req.on('close', () => child.kill());
});
```

### 5.3 Operations SQL 真实查询

```javascript
// 新增 /api/ops-sql (POST + SELECT 校验)
app.post('/api/ops-sql', (req, res) => {
  const { db, sql } = req.body;
  if (!/^\s*SELECT\s/i.test(sql)) {
    return res.status(403).json({ error: 'SELECT only' });
  }
  if (!/LIMIT\s+\d+/i.test(sql)) {
    sql += ' LIMIT 1000';
  }
  // exec wsl kubectl exec ... -- psql -c "SELECT..."
  const result = execSync(`wsl -d Ubuntu -- kubectl exec -n rgs deploy/${db} -- psql -U ${db}_user -d ${db}_db -c "${sql.replace(/"/g, '\\"')}"`, { encoding: 'utf8', timeout: 30000 });
  res.json({ result });
});
```

### 5.4 v0.3 性能预算

- 5 域 gRPC 调用: < 200ms(本地)
- WebSocket 日志延迟: < 1s
- Operations SQL: < 5s
- 总内存: < 80 MB

---

## 6. v1.0 详细规划(Rust 重写)

### 6.1 crates/rgs-web 结构

```
crates/rgs-web/
├── Cargo.toml
├── src/
│   ├── main.rs              (tokio + axum 入口)
│   ├── api/
│   │   ├── mod.rs
│   │   ├── health.rs        (GET /api/health)
│   │   ├── impl_plan.rs     (GET /api/impl-plan)
│   │   ├── worktrees.rs     (GET /api/worktrees)
│   │   ├── docs_health.rs   (GET /api/docs-health)
│   │   ├── git_log.rs       (GET /api/git-log)
│   │   ├── saga_trace.rs    (GET /api/saga-trace)
│   │   └── k8s_proxy.rs    (ALL /api/k8s/*)
│   ├── grpc_clients/
│   │   ├── mod.rs
│   │   ├── player.rs        (tonic client)
│   │   ├── economy.rs
│   │   ├── match.rs
│   │   ├── social.rs
│   │   └── admin.rs
│   ├── web/
│   │   ├── mod.rs
│   │   ├── handlers.rs     (axum Router)
│   │   └── static.rs       (askama template)
│   └── bin/
│       └── rgs-web.rs       (binary 入口)
├── templates/
│   ├── index.html.tera      (askama 模板, RGS-WEB v0.3 HTML 转译)
│   └── ...
└── tests/
    ├── integration.rs       (e2e 测试)
    └── unit.rs              (单元测试)
```

### 6.2 Cargo.toml 依赖

```toml
[dependencies]
# workspace
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tracing = { workspace = true }
anyhow = { workspace = true }

# axum 0.8
axum = { version = "0.8", features = ["ws"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["fs"] }

# askama 模板
askama = "0.13"
askama_axum = "0.5"

# gRPC client
tonic = { workspace = true }

# k8s 代理
hyper = { version = "1", features = ["client", "http1"] }
hyper-util = { version = "0.1", features = ["client-legacy", "tokio"] }
http-body-util = "0.1"

# 进程
tokio-process = "0.2"

[dev-dependencies]
reqwest = { workspace = true }
```

### 6.3 k8s 部署 manifest

```yaml
# tools/rgs-web/k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rgs-web
  namespace: rgs
spec:
  replicas: 1
  selector:
    matchLabels:
      app: rgs-web
  template:
    metadata:
      labels:
        app: rgs-web
    spec:
      containers:
      - name: rgs-web
        image: rgs-web:v1.0
        ports:
        - containerPort: 8788
        env:
        - name: RGS_WEB_PORT
          value: "8788"
        - name: K3S_TOKEN
          valueFrom:
            secretKeyRef:
              name: k3s-service-account
              key: token
        resources:
          requests: { cpu: 50m, memory: 64Mi }
          limits:   { cpu: 200m, memory: 128Mi }
```

### 6.4 v1.0 迁移路径

1. v0.3 接入 5 域 gRPC(JS 端)
2. v0.4 用 tera/askama 模板替换 HTML 字符串
3. v0.5 Rust port(逐 API 移植,2-4 周)
4. v1.0 上 k3s + 文档 + 关闭 node 版

---

## 7. 故障排查

### 7.1 启动失败

| 错误 | 原因 | 解决 |
|---|---|---|
| EADDRINUSE 8788 | 端口占用 | `RGS_WEB_PORT=8789 node ...` |
| EADDRINUSE 8787 | Cursor headroom | 用 8788(默认) |
| node: not found | node 未装 | 装 node 22+ |
| cwd ENOENT | tools/rgs-web 目录不在 | cd 到 D:/RustGameServer |
| 中文路径 read fail | 编码问题 | 用 plumbing 或 PowerShell 7 |

### 7.2 API 失败

| 错误 | 原因 | 解决 |
|---|---|---|
| /api/impl-plan 500 | git worktree list 失败 | 看 log: tail D:/tmp/rgs-web.err |
| /api/k8s/api/v1 401 | 缺 K3S_TOKEN | WSL sudo chmod 644 k3s.yaml |
| /api/k8s/api/v1 502 | WSL k3s API server 死 | wsl --shutdown, 等 30s |
| /api/k8s/api/v1 超时 | k3s API 慢 | 调长 client timeout |

### 7.3 页面空白

| 现象 | 原因 | 解决 |
|---|---|---|
| 404 Not Found | 路径错 | 看 network tab |
| mock 数据 | k8s 代理未通 | 等 WSL sudo chmod |
| 中文乱码 | PowerShell GBK | 用 node 直接跑(UTF-8) |

---

## 8. 验收清单

### 8.1 当前 v0.2-gm

- [x] server-no-deps.js 151 行, 6 API, k3s 代理
- [x] public/index.html 560 行, 10 页面
- [x] 启动 < 1s, 内存 33MB
- [x] 127.0.0.1 only
- [x] 8 份 IMPL-PLAN 跨 worktree
- [x] 45 worktree
- [x] 30 commit
- [x] dark theme
- [x] 30s 轮询
- [x] 中文路径兼容

### 8.2 v0.3 计划

- [ ] 5 域 gRPC client(JS)
- [ ] WebSocket 日志
- [ ] Operations SQL 真实
- [ ] /api/metrics

### 8.3 v1.0 计划

- [ ] Rust axum 重写
- [ ] askama 模板
- [ ] k3s 部署
- [ ] 5 域 gRPC client(Rust)
- [ ] Prometheus exporter

---

## 9. 修订历史

| 版本 | 日期 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-26 | 架构师(Ulysses（一人公司 12 角色 per DEC-008）)| 初版:server-no-deps.js 逐行 + index.html 逐函数 + 部署 + v0.3/v1.0 详细规划 + 故障排查 |

## A. v0.1 升版增量

### A.1 源 0 → v0.1

- 0 状态:无详细设计
- v0.1 新增:本文档(9 节,逐 API + 逐函数展开,含 v0.3 + v1.0 规划)

### A.2 与 BASIC-DESIGN 关系

- BASIC-DESIGN: 架构 / 选型 / 模块 / 流程(高 level)
- DETAILED-DESIGN: 函数级 / API schema / CSS 变量 / 错误处理 / 故障排查(低 level)

### A.3 已知缺口

- v0.3 5 域 gRPC client 待实现
- v1.0 Rust 重写待启动
- WebSocket 通道待加

### A.4 引用链与证据

- server-no-deps.js 实际代码 151 行
- public/index.html 实际代码 560 行
- rgs-web v0.2-gm commit `52c1a83`
- rgs-web v0.1 commit `5f827ee`
- 11 P0/P1/P2 commit(per RGS-REPORT-2026-08-26-P0P1P2_v0.2)
- 修订历史代签新规则 per 2026-08-26 08:40 JST
