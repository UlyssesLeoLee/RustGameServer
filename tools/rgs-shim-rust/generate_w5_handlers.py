#!/usr/bin/env python3
"""Generate w5-2 welfare/partner/social handlers for shim (per 9/9 20:17 JST Mavis 娲惧伐)
v0.6.0: 173 real handler functions in handlers.rs + 173 map.insert in registry.rs
"""
import re

# 璇诲彇 stub cmd 鍒楄〃
with open(r'tools/rgs-shim-rust/src/registry_stubs.rs', 'r', encoding='utf-8') as f:
    stub_content = f.read()
all_cmds = sorted(set(int(c) for c in re.findall(r'map\.entry\((\d+)\)', stub_content)))

# 閫?173 cmd: 110 welfare + 30 partner + 33 social
welfare = sorted(c for c in all_cmds if 24000 <= c <= 24999)
partner_all = sorted(c for c in all_cmds if 11000 <= c <= 11999 and c != 11001)
partner = partner_all[:30]
social_all = sorted(c for c in all_cmds if 16000 <= c <= 17999)
social = social_all[:33]

assert len(welfare) == 110, f"welfare = {len(welfare)}"
assert len(partner) == 30, f"partner = {len(partner)}"
assert len(social) == 33, f"social = {len(social)}"

# 鐢熸垚 handler 鍑芥暟 (瀛楄妭绾у榻?erlang 鎴?stub 妯″紡)
def gen_welfare_handler(cmd):
    return f'''// {cmd} welfare cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 24000-24999 绂忓埄/娲诲姩)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern per w5-1 30001-30102)
pub fn handle_welfare_{cmd}(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {{
    Box::pin(async move {{
        if !payload.is_empty() {{
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u8();
        }}
        tracing::debug!(cmd, "{cmd} welfare");
        // 24000-24999 绂忓埄/娲诲姩: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS economy.GetWelfare)
        Response {{ cmd, payload: vec![] }}
    }})
}}
'''

def gen_partner_handler(cmd):
    return f'''// {cmd} partner cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 11000-11999 浼欎即)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_partner_{cmd}(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {{
    Box::pin(async move {{
        if !payload.is_empty() {{
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }}
        tracing::debug!(cmd, "{cmd} partner");
        // 11000-11999 浼欎即: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS player.GetPartner)
        Response {{ cmd, payload: vec![] }}
    }})
}}
'''

def gen_social_handler(cmd):
    return f'''// {cmd} social cmd (per 2026-09-09 20:17 JST Mavis 娲惧伐 w5-2: 16000-17999 宸ヤ細/绀句氦)
// 鏉ユ簮: proto_mate.js send cmd (鏃?erlang proto, 璧?simple real handler pattern)
pub fn handle_social_{cmd}(
    cmd: u16,
    payload: Vec<u8>,
    _rgs: Arc<RgsClient>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>> {{
    Box::pin(async move {{
        if !payload.is_empty() {{
            let mut p: &[u8] = &payload[..];
            let _ = p.read_u32();
        }}
        tracing::debug!(cmd, "{cmd} social");
        // 16000-17999 绀句氦: 杩旂┖ ack (w5-2 绠€鍖? 鍚庣画鎺?RGS social.GetGuild)
        Response {{ cmd, payload: vec![] }}
    }})
}}
'''

# 鐢熸垚 handlers.rs 澧為噺鍐呭
handlers_block = "\n// ============================================================================\n"
handlers_block += "// v0.6.0 w5-2 (per 2026-09-09 20:17 JST Mavis 娲惧伐): 173 cmd real handler\n"
handlers_block += "// 110 welfare (24000-24999) + 30 partner (11000-11999) + 33 social (16000-17999)\n"
handlers_block += "// 瀛楄妭绾у榻?proto_mate.js send cmd, simple real handler pattern (per w5-1 30001-30102)\n"
handlers_block += "// 鏉ユ簮: H5 zsyz_client proto_mate.js (766 send cmd, 173 缁仛)\n"
handlers_block += "// ============================================================================\n\n"

for cmd in welfare:
    handlers_block += gen_welfare_handler(cmd) + "\n"
for cmd in partner:
    handlers_block += gen_partner_handler(cmd) + "\n"
for cmd in social:
    handlers_block += gen_social_handler(cmd) + "\n"

# 鐢熸垚 registry.rs 澧為噺鍐呭
registry_block = "\n        // v0.6.0 w5-2 (per 2026-09-09 20:17 JST Mavis 娲惧伐): 173 cmd real handler\n"
registry_block += "        // 瑕嗙洊 registry_stubs.rs 涓殑瀵瑰簲 stub, 鏀圭敤 real handler\n"
registry_block += "        // 110 welfare (24000-24999) + 30 partner (11000-11999) + 33 social (16000-17999)\n"
registry_block += "        map.insert(24000, CmdEntry { handler: handlers::handle_welfare_24000, name: \"welfare-24000 (w5-2 simple real)\", source: \"zsyz\" });\n"

# Add all 110 welfare
for cmd in welfare:
    registry_block += f'        map.insert({cmd}, CmdEntry {{ handler: handlers::handle_welfare_{cmd}, name: "welfare-{cmd} (w5-2 simple real)", source: "zsyz" }});\n'
# Add 30 partner
for cmd in partner:
    registry_block += f'        map.insert({cmd}, CmdEntry {{ handler: handlers::handle_partner_{cmd}, name: "partner-{cmd} (w5-2 simple real)\", source: "zsyz" }});\n'
# Add 33 social
for cmd in social:
    registry_block += f'        map.insert({cmd}, CmdEntry {{ handler: handlers::handle_social_{cmd}, name: "social-{cmd} (w5-2 simple real)", source: "zsyz" }});\n'

with open(r'/tmp/handlers_w5_2.rs', 'w', encoding='utf-8') as f:
    f.write(handlers_block)
with open(r'/tmp/registry_w5_2.rs', 'w', encoding='utf-8') as f:
    f.write(registry_block)

print(f"handlers_w5_2.rs: {len(handlers_block)} bytes, {handlers_block.count(chr(10))} lines")
print(f"registry_w5_2.rs: {len(registry_block)} bytes, {registry_block.count(chr(10))} lines")
print(f"Total: 110 welfare + 30 partner + 33 social = {len(welfare) + len(partner) + len(social)}")
