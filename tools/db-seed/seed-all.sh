#!/usr/bin/env bash
# 6 DB seed 跑入 (per WF-1-db-seed-psql-9464 W3 worker)
# 用法: wsl -e bash tools/db-seed/seed-all.sh
set -euo pipefail
NS=rust-game-server
PGPASSWORD_VAL=ulysses_local
SQL_DIR="$(cd "$(dirname "$0")" && pwd)"

for pair in \
  "player_db:player_user:seed-player_db.sql" \
  "economy_db:economy_user:seed-economy_db.sql" \
  "match_db:match_user:seed-match_db.sql" \
  "social_db:social_user:seed-social_db.sql" \
  "admin_db:admin_user:seed-admin_db.sql" \
  "cluster_ops_db:cluster_ops_user:seed-cluster_ops_db.sql"; do
  db="${pair%%:*}"; rest="${pair#*:}"
  user="${rest%%:*}"; file="${rest#*:}"
  echo "=== $db ==="
  cat "$SQL_DIR/$file" | sudo k3s kubectl exec -i -n "$NS" deploy/postgres -- \
    env PGPASSWORD="$PGPASSWORD_VAL" psql -U "$user" -d "$db" -v ON_ERROR_STOP=1
done
echo "=== seed complete ==="
