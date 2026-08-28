#!/usr/bin/env bash
# 设置 DATABASE_URL + PG env vars, 不打印 secret. 在当前 shell 里 source.
# 用法: source ./scripts/db-url.sh player-db-credentials 15432
# 兼容 secret key 两种风格:
#   - 域 user: data.{username, password, database} (player-db-credentials 等)
#   - superuser: data.{POSTGRES_USER, POSTGRES_PASSWORD} (postgres-superuser)
set -e
SECRET_NAME="${1:-player-db-credentials}"
LOCAL_PORT="${2:-15432}"
NS="${NS:-rust-game-server}"

# 探测 secret key 风格
SECRET_JSON=$(k3s kubectl get secret "${SECRET_NAME}" -n "${NS}" -o json)
HAS_LOWER=$(echo "${SECRET_JSON}" | python3 -c "import sys,json; d=json.load(sys.stdin)['data']; print('yes' if 'username' in d else 'no')")

if [ "${HAS_LOWER}" = "yes" ]; then
    PASSWORD=$(echo "${SECRET_JSON}" | python3 -c "import sys,json,base64; d=json.load(sys.stdin)['data']; print(base64.b64decode(d['password']).decode())")
    USER=$(echo "${SECRET_JSON}" | python3 -c "import sys,json,base64; d=json.load(sys.stdin)['data']; print(base64.b64decode(d['username']).decode())")
    DB=$(echo "${SECRET_JSON}" | python3 -c "import sys,json,base64; d=json.load(sys.stdin)['data']; print(base64.b64decode(d['database']).decode())")
else
    PASSWORD=$(echo "${SECRET_JSON}" | python3 -c "import sys,json,base64; d=json.load(sys.stdin)['data']; print(base64.b64decode(d['POSTGRES_PASSWORD']).decode())")
    USER=$(echo "${SECRET_JSON}" | python3 -c "import sys,json,base64; d=json.load(sys.stdin)['data']; print(base64.b64decode(d['POSTGRES_USER']).decode())")
    DB="${PGDATABASE:-postgres}"
fi

# 在当前 shell 直接 export, 避免 eval quoting 问题
export DATABASE_URL="postgres://${USER}:${PASSWORD}@localhost:${LOCAL_PORT}/${DB}"
export RGS_DB_USER="${USER}"
export RGS_DB_NAME="${DB}"
export PGUSER="${USER}"   # 防 sqlx 0.8.6 fallback whoami::username
export PGPASSWORD="${PASSWORD}"
export PGPORT="${LOCAL_PORT}"
export PGHOST="localhost"

echo "[db-url] source 完毕 (user=${USER} db=${DB} port=${LOCAL_PORT})"
echo "[db-url] DATABASE_URL 已 export (密码 redaction)"
