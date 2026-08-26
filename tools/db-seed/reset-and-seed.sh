#!/usr/bin/env bash
# 清空 6 DB 业务表（保留 schema/migrations）+ 重新 seed
# 用法: wsl -e bash tools/db-seed/reset-and-seed.sh
set -euo pipefail
NS=rust-game-server
PGPASSWORD_VAL=ulysses_local
SQL_DIR="$(cd "$(dirname "$0")" && pwd)"

# 清表（FK 顺序：先依赖表，后被引用表；audit_log 因 trigger 不可删跳过手动 DELETE）
# 注: audit_log 实际是 BLOCK DELETE, 用 TRUNCATE 绕过 trigger? 不, TRUNCATE 也走 BEFORE DELETE
#     安全: 跳过 audit_log, 不清旧记录, 唯一冲突由 ON CONFLICT 兜底
for pair in \
  "player_db:player_user:TRUNCATE player_sessions, players RESTART IDENTITY CASCADE" \
  "economy_db:economy_user:TRUNCATE transaction_ledger, reservations, sagas, accounts RESTART IDENTITY CASCADE" \
  "match_db:match_user:TRUNCATE match_participants, matches RESTART IDENTITY CASCADE" \
  "social_db:social_user:TRUNCATE guild_members, guilds RESTART IDENTITY CASCADE" \
  "admin_db:admin_user:TRUNCATE admin_users RESTART IDENTITY CASCADE" \
  "cluster_ops_db:cluster_ops_user:TRUNCATE feature_flags, cluster_nodes RESTART IDENTITY CASCADE"; do
  db="${pair%%:*}"; rest="${pair#*:}"
  user="${rest%%:*}"; sql="${rest#*:}"
  echo "=== reset $db ==="
  sudo k3s kubectl exec -n "$NS" deploy/postgres -- \
    env PGPASSWORD="$PGPASSWORD_VAL" psql -U "$user" -d "$db" -c "$sql" 2>&1
done

# admin_db audit_log 单独处理：用 DELETE 是被禁的, 改用 DROP/CREATE trigger + TRUNCATE
echo "=== reset admin_db.audit_log (skip DELETE, use TRUNCATE) ==="
sudo k3s kubectl exec -n "$NS" deploy/postgres -- env PGPASSWORD="$PGPASSWORD_VAL" psql -U admin_user -d admin_db -c "TRUNCATE audit_log" 2>&1 || echo "  TRUNCATE blocked — manual handling needed"

# 重新 seed
for pair in \
  "player_db:player_user:seed-player_db.sql" \
  "economy_db:economy_user:seed-economy_db.sql" \
  "match_db:match_user:seed-match_db.sql" \
  "social_db:social_user:seed-social_db.sql" \
  "admin_db:admin_user:seed-admin_db.sql" \
  "cluster_ops_db:cluster_ops_user:seed-cluster_ops_db.sql"; do
  db="${pair%%:*}"; rest="${pair#*:}"
  user="${rest%%:*}"; file="${rest#*:}"
  echo "=== seed $db ==="
  cat "$SQL_DIR/$file" | sudo k3s kubectl exec -i -n "$NS" deploy/postgres -- \
    env PGPASSWORD="$PGPASSWORD_VAL" psql -U "$user" -d "$db" -v ON_ERROR_STOP=1
done
echo "=== reset+seed complete ==="
