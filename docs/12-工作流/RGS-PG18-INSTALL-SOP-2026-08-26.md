# RGS-PG18-INSTALL-SOP-2026-08-26 v0.1

**RGS PostgreSQL 18 安装 + 5 域 DB 创建 SOP(per Ulysses 16:58 JST "PG 18 是必须装的")**

| 项目 | 内容 |
|---|---|
| 文档编号 | RGS-PG18-INSTALL-SOP-2026-08-26 |
| 版本 | 0.1 |
| 触发 | 2026-08-26 16:58 JST Ulysses 确认 PG 18 必须装 |
| 责任人 | Ulysses(WSL 内执行)+ Mavis(装完后启 5 域) |

---

## 0. 背景

**per RGS-TS-001 §5 + ARC-008 5 域分 DB 原则**:PG 18 是 RGS 唯一支持的生产数据库。
- 5 域(player/economy/match/social/admin)+ cluster-ops 各自独立 DB
- 5 域 + cluster-ops + shared-platform 用 `RGS_INMEMORY=1` fallback **仅 dev/CI 试用**(per RGS-GM-V0.3-DEPLOY-SOP)
- **生产必须真 PG 18**

当前 WSL Ubuntu 24.04 内 **未装 PG**,Mavis 提权装不了(sudo 需密码)。

---

## 1. Ulysses 在 WSL Ubuntu terminal 执行(预计 5-8 分钟)

```bash
# === Step 1: 装 PostgreSQL 18 (Ubuntu 24.04 标准仓库默认 PG 16,需要 PGDG 仓库) ===
# 1.1 加 PGDG 仓库(Ubuntu 24.04 = noble)
sudo apt-get install -y curl ca-certificates gnupg lsb-release
sudo install -d /usr/share/postgresql-common/pgdg
sudo /usr/share/postgresql-common/pgdg/apt.postgresql.org.sh

# 1.2 装 PG 18(约 3-5 分钟)
sudo apt-get update
sudo DEBIAN_FRONTEND=noninteractive apt-get install -y postgresql-18 postgresql-client-18

# 1.3 启动 PG 18(不依赖 systemd,直接 pg_ctlcluster)
sudo pg_ctlcluster 18 main start
# 或: sudo service postgresql start

# 1.4 验证(关键!)
sudo -u postgres psql -c "SELECT version();"
# 期望: PostgreSQL 18.x on x86_64-pc-linux-gnu ...

# === Step 2: 创建 5 域 DB + 5 user(per ARC-008 5 域分 DB) ===
sudo -u postgres psql << 'EOF'
-- 5 user(各自独立 superuser,可建表/索引/迁移)
CREATE USER player     WITH PASSWORD 'rgs_dev' SUPERUSER;
CREATE USER economy    WITH PASSWORD 'rgs_dev' SUPERUSER;
CREATE USER match_user WITH PASSWORD 'rgs_dev' SUPERUSER;
CREATE USER social     WITH PASSWORD 'rgs_dev' SUPERUSER;
CREATE USER admin      WITH PASSWORD 'rgs_dev' SUPERUSER;

-- 5 DB(各自 owner 独立)
CREATE DATABASE player_db  OWNER player;
CREATE DATABASE economy_db OWNER economy;
CREATE DATABASE match_db   OWNER match_user;
CREATE DATABASE social_db  OWNER social;
CREATE DATABASE admin_db   OWNER admin;

-- 给 5 user 各自 superuser 已可,补 grant(可选)
GRANT ALL PRIVILEGES ON DATABASE player_db  TO player;
GRANT ALL PRIVILEGES ON DATABASE economy_db TO economy;
GRANT ALL PRIVILEGES ON DATABASE match_db   TO match_user;
GRANT ALL PRIVILEGES ON DATABASE social_db  TO social;
GRANT ALL PRIVILEGES ON DATABASE admin_db   TO admin;

\q
EOF

# === Step 3: WSL 端口暴露到 Windows(让 rgs-web 在 Windows 端可连) ===
# 默认 PG 监听 127.0.0.1:5432,如果 rgs-web 跑在 Windows 端,需要改 listen
sudo sed -i "s/^#listen_addresses = 'localhost'/listen_addresses = '*'/" /etc/postgresql/18/main/postgresql.conf
echo "host all all 0.0.0.0/0 md5" | sudo tee -a /etc/postgresql/18/main/pg_hba.conf
sudo pg_ctlcluster 18 main restart

# === Step 4: 验证 5 DB ===
for db in player_db economy_db match_db social_db admin_db; do
  echo "--- $db ---"
  PGPASSWORD=rgs_dev psql -h 127.0.0.1 -U ${db%_db} -d $db -c "SELECT current_database(), current_user, version();" 2>&1 | head -5
done
# 期望: 5 个全过,显示 5 个 DB + 5 个 user + PG 18.x
```

**输出示例**:
```
--- player_db ---
 current_database | current_user |                                        version
------------------+--------------+--------------------------------------------------------------------
 player_db        | player       | PostgreSQL 18.1 on x86_64-pc-linux-gnu, ...
(1 row)
...
```

---

## 2. 完成后告诉 Mavis "PG 18 ready"

Mavis 收到通知后会:
1. 验证 5 DB + 5 user
2. 设置 WSL2 端口转发(127.0.0.1:5432 → Windows 127.0.0.1:5432,如需)
3. **启动 6 个 binary**(5 域 + cluster-ops,后台进程)
4. 等待 6 个 gRPC port listen(50051-50056)
5. 调 gRPC `HealthCheck` 验证服务
6. **rgs-web 接 5 域真实 gRPC**(commit v0.3-gm)
7. 19 页面 ROPE_CS 完备(15 页面落地,4 页面 per DEC-008 不做)

---

## 3. 5 域 + cluster-ops 启动命令(Mavis 跑)

Mavis 在 Windows 端,启 6 个 binary(后台):

```powershell
# 6 个 binary 在 E:\DevCache\cargo\target\debug\
$binaries = @(
  @{name='player-service';   port=50051; db='player'},
  @{name='economy-service';  port=50052; db='economy'},
  @{name='match-service';    port=50053; db='match_user'},
  @{name='social-service';   port=50054; db='social'},
  @{name='admin-service';    port=50055; db='admin'},
  @{name='cluster-ops';       port=50056; db='admin'}
)
$binDir = 'E:\DevCache\cargo\target\debug'
foreach ($svc in $binaries) {
  $env:GRPC_ADDR = "0.0.0.0:$($svc.port)"
  $env:DATABASE_URL = "postgres://$($svc.db):rgs_dev@127.0.0.1:5432/$($svc.db)_db"
  $env:RGS_ALLOW_INSECURE_GRPC = '1'  # dev only,生产 mTLS
  $env:NATS_URI = 'nats://localhost:4222'  # 可选
  $psi = New-Object Diagnostics.ProcessStartInfo
  $psi.FileName = Join-Path $binDir "$($svc.name).exe"
  $psi.EnvironmentVariables['GRPC_ADDR'] = $env:GRPC_ADDR
  $psi.EnvironmentVariables['DATABASE_URL'] = $env:DATABASE_URL
  $psi.EnvironmentVariables['RGS_ALLOW_INSECURE_GRPC'] = $env:RGS_ALLOW_INSECURE_GRPC
  $psi.EnvironmentVariables['NATS_URI'] = $env:NATS_URI
  $psi.RedirectStandardOutput = "D:\tmp\$($svc.name).log"
  $psi.RedirectStandardError = "D:\tmp\$($svc.name).err"
  $psi.UseShellExecute = $false
  $psi.CreateNoWindow = $true
  $proc = [Diagnostics.Process]::new()
  $proc.StartInfo = $psi
  [void]$proc.Start()
  Write-Host "$($svc.name) PID: $($proc.Id) on port $($svc.port)"
}
# 验证 6 个 gRPC port listen
Start-Sleep -Seconds 30
Get-NetTCPConnection -LocalPort 50051,50052,50053,50054,50055,50056 -State Listen | Format-Table
```

---

## 4. rgs-web 接 5 域 gRPC(v0.3-gm)

| 页面 | v0.2-gm | v0.3-gm 真实 |
|---|---|---|
| Players | mock | `player-service.GetPlayer(player_id)` 真实 |
| Servers | k3s 代理 | + 5 域 binary 进程状态(Get-NetTCPConnection) |
| Live Console | setInterval mock | WSL tail `/var/log/5 域.log` |
| Operations SQL | mock | `psql -h 127.0.0.1 -U ... -d <db> -c <SELECT>`(per ARC-008 + SELECT only) |
| Hot Update | git log | cluster-ops PFAU phase gRPC |
| Config | 静态 | cluster-ops config_dump gRPC |

---

## 5. 故障排查

### 5.1 PG 装不上
```bash
# 看错误
sudo apt-get install -y postgresql-18 2>&1 | tail -20
# 看 PGDG 仓库
cat /etc/apt/sources.list.d/pgdg.list
# 强制重装
sudo apt-get update && sudo apt-get install --reinstall -y postgresql-18
```

### 5.2 PG 起不来
```bash
# 看 status
sudo pg_lsclusters
sudo pg_ctlcluster 18 main status
# 看 log
sudo tail -50 /var/log/postgresql/postgresql-18-main.log
```

### 5.3 5 域 binary 起不来
```powershell
# 看 log
Get-Content D:\tmp\player-service.err
Get-Content D:\tmp\player-service.log -Tail 20
# 常见:DATABASE_URL 格式错 / port 占用 / cert 文件缺
```

### 5.4 rgs-web 5 域调用失败
```powershell
# 看 rgs-web /api/k3s 或新加 /api/grpc/player/health
$body = Invoke-WebRequest 'http://127.0.0.1:8788/api/grpc/player/health' -UseBasicParsing
$body.Content
# 期望: {"status":"SERVING"}
```

---

## 6. 修订历史

| 版本 | 日期 | 修订者 | 修订内容 |
|---|---|---|---|
| 0.1 | 2026-08-26 | 架构师(Mavis 接手 agent per DEC-008)| 初版:PG 18 + 5 域 DB + 5 binary 启动 SOP |

## A. v0.1 升版增量

### A.1 源 0 → v0.1

- 0 状态:WSL Ubuntu 无 PG,5 域 binary 未跑
- v0.1 新增:PG 18 PGDG 仓库 + 5 域 DB + 6 binary 启动脚本 + 故障排查

### A.2 引用链与证据

- per RGS-TS-001 §5(PG 18 唯一支持)
- per ARC-008(5 域分 DB)
- per RGS-REV-008 AC-1(mTLS fail-closed)
- per RGS-REV-009 V3 H-1(NoopMock deprecation)
- per RGS-GM-V0.3-DEPLOY-SOP-2026-08-26 v0.1
- 修订历史代签新规则 per 2026-08-26 08:40 JST
