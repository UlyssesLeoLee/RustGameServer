#!/usr/bin/env python3
"""RGS Flash Mock mock_switch reader tests (per ULYS-190 §4.4 stage4 cross-project pattern).

Mirrors Star `tests/test_mock_switch_plugins.py` (PR #151) 範式 + IM1.0
`tests/im_testkit_module_switch.rs` (PR #24) 範式 + CATs
`tests/cats_mock_module_switch.rs` (PR #18) 範式.

Assertions:
- 5 plugins total (player / economy / match / social / admin) per §4.3 brief
- 12 modules total (per pub mod 12 子模块 in handlers.rs)
- per plugin × module count matches
- cluster enabled + mode assertions
- aci_compat_version consistency
- mock_switch_trace_format 真拼接
- module_count_total/enabled field verify
- run_helper 调 _lib_mock_switch_rgs.py read-plugins (跨 Python invocations)
"""

import json
import os
import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]  # tests/ → rgs-flash-mock/ → tools/ → <worktree_root>
ACI = ROOT / "tools" / "rgs-flash-mock" / ".aci.json"
CLUSTER = ROOT / "tools" / "rgs-flash-mock" / ".mock-cluster.json"
HELPER = ROOT / "tools" / "rgs-flash-mock" / "scripts" / "_lib_mock_switch_rgs.py"


class TestRgsModuleSwitch(unittest.TestCase):
    def setUp(self):
        self.aci = json.loads(ACI.read_text(encoding="utf-8"))
        self.cluster = json.loads(CLUSTER.read_text(encoding="utf-8"))

    # ---- helper 基础 ----

    def test_aci_file_exists(self):
        self.assertTrue(ACI.exists(), f".aci.json missing at {ACI}")

    def test_cluster_file_exists(self):
        self.assertTrue(CLUSTER.exists(), f".mock-cluster.json missing at {CLUSTER}")

    def test_helper_exists(self):
        self.assertTrue(HELPER.exists(), f"helper script missing at {HELPER}")

    # ---- 总数 ----

    def test_module_switch_total_count_is_12(self):
        plugins = self.aci["plugins"]
        total = sum(len(p.get("modules", {})) for p in plugins.values())
        self.assertEqual(
            total,
            12,
            f"expected 12 modules across 5 plugins, got {total} (per handlers.rs 12 pub mod)",
        )

    def test_module_switch_all_enabled_by_default(self):
        plugins = self.aci["plugins"]
        for pid, pconf in plugins.items():
            for mid, mconf in pconf.get("modules", {}).items():
                self.assertTrue(
                    mconf.get("enabled", False),
                    f"plugin={pid} module={mid} should be enabled",
                )

    # ---- per plugin × module count 验证 ----

    def test_player_plugin_has_3_modules(self):
        mods = self.aci["plugins"]["player"]["modules"]
        self.assertEqual(set(mods.keys()), {"role", "scene", "friend"})

    def test_economy_plugin_has_3_modules(self):
        mods = self.aci["plugins"]["economy"]["modules"]
        self.assertEqual(set(mods.keys()), {"econ", "pay", "event"})

    def test_match_plugin_has_2_modules(self):
        mods = self.aci["plugins"]["match"]["modules"]
        self.assertEqual(set(mods.keys()), {"combat", "pvp"})

    def test_social_plugin_has_2_modules(self):
        mods = self.aci["plugins"]["social"]["modules"]
        self.assertEqual(set(mods.keys()), {"guild", "rank"})

    def test_admin_plugin_has_2_modules(self):
        mods = self.aci["plugins"]["admin"]["modules"]
        self.assertEqual(set(mods.keys()), {"gm", "card"})

    # ---- cluster assertions ----

    def test_cluster_enabled_true_and_mode_offline(self):
        self.assertEqual(self.cluster["enabled"], True)
        self.assertEqual(self.cluster["mode"], "offline")

    # ---- compat validation ----

    def test_aci_compat_version_consistent_across_cluster_and_aci(self):
        cluster_v = self.cluster.get("aci_compat_version")
        aci_v = self.aci.get("aci_compat_version")
        self.assertIsNotNone(cluster_v, "cluster.aci_compat_version missing")
        self.assertIsNotNone(aci_v, "aci.aci_compat_version missing")
        self.assertEqual(
            cluster_v,
            aci_v,
            f"cluster.aci_compat_version={cluster_v} != aci.aci_compat_version={aci_v}",
        )

    # ---- plugins count vs summary plugins_total ----

    def test_plugins_count_matches_summary_plugins_total(self):
        plugins = self.aci["plugins"]
        self.assertEqual(len(plugins), 5, "expected 5 plugins (per §4.3 brief)")

    # ---- cross Python invocation: run helper subprocess ----

    def test_run_helper_read_plugins(self):
        proc = subprocess.run(
            [
                sys.executable,
                str(HELPER),
                "--aci-config",
                str(ACI),
                "read-plugins",
            ],
            capture_output=True,
            text=True,
            cwd=str(ROOT),
        )
        self.assertEqual(
            proc.returncode,
            0,
            f"helper failed: stderr={proc.stderr}",
        )
        out = json.loads(proc.stdout)
        self.assertEqual(out["plugins_total"], 5)
        self.assertEqual(out["modules_total"], 12)
        self.assertEqual(out["modules_enabled"], 12)

    def test_run_helper_trace(self):
        proc = subprocess.run(
            [
                sys.executable,
                str(HELPER),
                "--aci-config",
                str(ACI),
                "trace",
            ],
            capture_output=True,
            text=True,
            cwd=str(ROOT),
        )
        self.assertEqual(proc.returncode, 0, f"trace failed: {proc.stderr}")
        # 验证 trace 真拼接 (per IM1.0 §4 範式)
        trace = proc.stdout.strip()
        self.assertIn("cluster.enabled=", trace)
        self.assertIn("mode=offline", trace)
        self.assertIn("=12/12 modules", trace)

    def test_run_helper_validate_compat(self):
        proc = subprocess.run(
            [
                sys.executable,
                str(HELPER),
                "--aci-config",
                str(ACI),
                "validate-compat",
            ],
            capture_output=True,
            text=True,
            cwd=str(ROOT),
        )
        self.assertEqual(proc.returncode, 0, f"validate-compat failed: {proc.stderr}")
        self.assertIn("ACI_COMPAT=OK", proc.stdout)

    # ---- module_count_total / enabled field verify ----

    def test_cluster_module_count_total_12(self):
        self.assertEqual(
            self.cluster.get("module_count_total"),
            12,
            f"cluster.module_count_total expected 12, got {self.cluster.get('module_count_total')}",
        )

    def test_cluster_module_count_enabled_12(self):
        self.assertEqual(
            self.cluster.get("module_count_enabled"),
            12,
        )


if __name__ == "__main__":
    unittest.main(verbosity=2)
