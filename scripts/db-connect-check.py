#!/usr/bin/env python3
"""验证 k3s secret 凭证 + psql 连通性. 不打印 secret 值."""
import base64
import json
import os
import subprocess
import sys


def get_secret(name, key):
    out = subprocess.run(
        ["k3s", "kubectl", "get", "secret", name, "-n", "rust-game-server", "-o", "json"],
        check=True, capture_output=True, text=True,
    ).stdout
    # k8s Secret JSON: data is { "<key>": "<base64>" }
    data_map = json.loads(out)["data"]
    return base64.b64decode(data_map[key]).decode()


def main():
    user = get_secret("player-db-credentials", "username")
    db = get_secret("player-db-credentials", "database")
    pw = get_secret("player-db-credentials", "password")
    port = os.environ.get("LOCAL_PORT", "5432")
    env = {"PGPASSWORD": pw, "PATH": "/usr/bin:/bin"}
    res = subprocess.run(
        ["psql", "-h", "localhost", "-p", port, "-U", user, "-d", db,
         "-c", "SELECT current_user, current_database();"],
        env=env, capture_output=True, text=True,
    )
    print(f"exit_code: {res.returncode}")
    print(f"stdout: {res.stdout.strip()}")
    print(f"stderr: {res.stderr.strip()}")
    return res.returncode


if __name__ == "__main__":
    sys.exit(main())
