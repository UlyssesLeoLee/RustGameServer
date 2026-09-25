#!/usr/bin/env bash
# Force-kill only stale cargo/rustc processes (don't kill bash — that's our shell).
echo "before:"
tasklist.exe 2>&1 | grep -iE "cargo|rustc" | wc -l
cmd.exe //c "taskkill /F /IM cargo.exe /T" >/dev/null 2>&1
cmd.exe //c "taskkill /F /IM rustc.exe /T" >/dev/null 2>&1
sleep 3
echo "after kill:"
tasklist.exe 2>&1 | grep -iE "cargo|rustc" | wc -l
# Remove cargo lock
rm -f /e/DevCache/cargo/target/debug/.cargo-lock 2>/dev/null || true
ls /e/DevCache/cargo/target/debug/.cargo-lock 2>&1 | tail -1
