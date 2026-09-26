#!/usr/bin/env python3
"""RGS Flash Mock cluster_switch + plugin_switch + module_switch reader.

Per ULYS-190 §4.4 stage4 (cross-project pattern G-MS-04):
- IM1.0 PR #24 (commit 96a2e28 on dev): 5 plugin x 28 module
- CATs PR #18 (commit 9b97d6b on main): 4 plugin x 13 module
- Star PR #151 (commit 55cf3794 on dev): 7 plugin x 7 module
- RGS stage4 (this file): 5 plugin x 12 module
  (player(3) + economy(3) + match(2) + social(2) + admin(2))

CLI subcommands:
- is-enabled / get-mode / trace / validate-compat: 既有 4 个, 跟 Star / IM1.0 範式一致
- read-plugins (NEW): 输出 plugin x module 树 JSON, 跨 Python invocation tests 必用
"""

import argparse
import json
import sys
from pathlib import Path


class MockSwitchReader:
    """Read RGS rgs-flash-mock cluster_switch / plugin_switch / module_switch config.

    Files:
    - .mock-cluster.json: cluster level (enabled / mode / fallback_to_real / mock_switch_trace_format)
    - .aci.json: plugin and module definitions
    """

    def __init__(self, aci_config_path: str, cluster_config_path: str = None):
        self.aci_path = Path(aci_config_path)
        self.cluster_path = (
            Path(cluster_config_path) if cluster_config_path else None
        )
        self.aci = json.loads(self.aci_path.read_text(encoding="utf-8"))
        self.cluster = (
            json.loads(self.cluster_path.read_text(encoding="utf-8"))
            if self.cluster_path and self.cluster_path.exists()
            else {}
        )

    def is_enabled(self) -> bool:
        return bool(self.cluster.get("enabled", False))

    def get_mode(self) -> str:
        return self.cluster.get("mode", "unknown")

    def validate_compat(self):
        cluster_v = self.cluster.get("aci_compat_version")
        aci_v = self.aci.get("aci_compat_version")
        if cluster_v is None or aci_v is None:
            return False, f"missing: cluster={cluster_v} aci={aci_v}"
        return cluster_v == aci_v, f"cluster={cluster_v} aci={aci_v}"

    def build_trace(self) -> str:
        tmpl = self.cluster.get(
            "mock_switch_trace_format",
            "cluster.enabled={cluster.enabled},mode={cluster.mode},plugins=[TBD]",
        )
        # Substitute placeholders manually because key names contain dots
        # (str.format() can't index dotted keys as kwargs).
        out = tmpl
        out = out.replace("{cluster.enabled}", str(self.is_enabled()).lower())
        out = out.replace("{cluster.mode}", self.get_mode())
        return out

    def read_plugins(self) -> dict:
        plugins = self.aci.get("plugins", {})
        plugins_total = len(plugins)
        plugins_enabled = sum(1 for p in plugins.values() if p.get("enabled", False))
        modules_total = 0
        modules_enabled = 0
        out_plugins = {}
        for pid, pconf in plugins.items():
            mods = pconf.get("modules", {})
            pmods_total = len(mods)
            pmods_enabled = sum(1 for m in mods.values() if m.get("enabled", False))
            modules_total += pmods_total
            modules_enabled += pmods_enabled
            out_plugins[pid] = {
                "plugin_id": pconf.get("plugin_id", pid),
                "enabled": pconf.get("enabled", False),
                "default_mode": pconf.get("default_mode", "offline"),
                "modules_total": pmods_total,
                "modules_enabled": pmods_enabled,
                "modules": {
                    mid: {
                        "module_id": mconf.get("module_id", mid),
                        "enabled": mconf.get("enabled", False),
                        "mode": mconf.get("mode", "offline"),
                    }
                    for mid, mconf in mods.items()
                },
            }
        return {
            "plugins_total": plugins_total,
            "plugins_enabled": plugins_enabled,
            "modules_total": modules_total,
            "modules_enabled": modules_enabled,
            "plugins": out_plugins,
        }


def _default_cluster(aci_path: str) -> str:
    p = Path(aci_path).resolve()
    c = p.parent / ".mock-cluster.json"
    return str(c) if c.exists() else None


def _cli():
    parser = argparse.ArgumentParser(description="RGS rgs-flash-mock mock_switch reader")
    parser.add_argument(
        "--cluster-config", required=False,
        help="path to .mock-cluster.json (default: <aci_dir>/.mock-cluster.json)",
    )
    parser.add_argument(
        "--aci-config", required=False,
        help="path to .aci.json (default: ./tools/rgs-flash-mock/.aci.json)",
    )
    sub = parser.add_subparsers(dest="cmd", required=False)

    sub.add_parser("is-enabled")
    sub.add_parser("get-mode")
    sub.add_parser("trace")
    sub.add_parser("validate-compat")
    sub.add_parser("read-plugins")

    args = parser.parse_args()
    cmd = args.cmd or "read-plugins"
    if not args.aci_config:
        parser.error("--aci-config is required")

    cluster = args.cluster_config or _default_cluster(args.aci_config)
    reader = MockSwitchReader(args.aci_config, cluster)

    if cmd == "is-enabled":
        print("CLUSTER_ENABLED=" + ("true" if reader.is_enabled() else "false"))
        return 0 if reader.is_enabled() else 1
    if cmd == "get-mode":
        print("CLUSTER_MODE=" + reader.get_mode())
        return 0
    if cmd == "trace":
        print(reader.build_trace())
        return 0
    if cmd == "validate-compat":
        ok, msg = reader.validate_compat()
        print(f"ACI_COMPAT={'OK' if ok else 'FAIL'} ({msg})")
        return 0 if ok else 1
    if cmd == "read-plugins":
        print(json.dumps(reader.read_plugins(), indent=2, ensure_ascii=False))
        return 0
    return 2


if __name__ == "__main__":
    sys.exit(_cli())
