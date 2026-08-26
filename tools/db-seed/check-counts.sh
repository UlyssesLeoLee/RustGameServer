#!/usr/bin/env bash
# 看每个 DB 各表当前行数（pre-seed 实证）
set -euo pipefail
NS=rust-game-server
for pair in \
  "player_db:player_user:players:player_sessions" \
  "economy_db:economy_user:accounts:transaction_ledger:sagas:reservations" \
  "match_db:match_user:matches:match_participants" \
  "social_db:social_user:guilds:guild_members" \
  "admin_db:admin_user:admin_users:audit_log" \
  "cluster_ops_db:cluster_ops_user:cluster_nodes:feature_flags"; do
  db="${pair%%:*}"; rest="${pair#*:}"
  user="${rest%%:*}"; tbls="${rest#*:}"
  echo "=== $db ==="
  sel=""
  for t in ${tbls//:/ }; do
    sel="${sel:+${sel} UNION ALL }SELECT '$t'::text AS tbl, count(*)::int AS n FROM $t"
  done
  sudo k3s kubectl exec -n "$NS" deploy/postgres -- env PGPASSWORD=ulysses_local psql -U "$user" -d "$db" -c "$sel" 2>&1
done
