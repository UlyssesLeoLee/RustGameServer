#!/usr/bin/env python3
"""Insert w5-2 entries into registry.rs cleanly."""
with open(r'tools/rgs-shim-rust/src/registry.rs', 'r', encoding='utf-8') as f:
    orig = f.read()
with open(r'/tmp/registry_w5_2.rs', 'r', encoding='utf-8') as f:
    w52 = f.read()

marker = 'Registry { map }'
idx = orig.rfind(marker)
if idx == -1:
    print('Marker not found')
else:
    before = orig[:idx]
    after = orig[idx + len(marker):]

    lines = w52.split('\n')
    w52_data = [l + '\n' for l in lines[5:] if l.strip()]
    w52_block = ''.join(w52_data)

    insertion = (
        '        // v0.6.0 w5-2 (per 2026-09-09 20:17 JST Mavis 娲惧伐): 173 cmd real handler\n'
        '        // 110 welfare (24000-24999) + 30 partner (11000-11999) + 33 social (16000-17999)\n'
        '        // 瀛楄妭绾у榻?proto_mate.js send cmd, simple real handler pattern\n'
        '        // 瑕嗙洊 registry_stubs.rs 涓殑瀵瑰簲 stub, 鏀圭敤 real handler\n'
        '        // 宸茬煡缂哄彛: main HEAD 鏈?pre-existing build 閿欒 (w2 鍚堝苟閬楃暀), w5-2 鑼冨洿鍐?3 澶勫凡鏈€灏忎慨澶峔n'
        + w52_block
    )

    new_content = before + insertion + marker + after
    with open(r'tools/rgs-shim-rust/src/registry.rs', 'w', encoding='utf-8') as f:
        f.write(new_content)

    with open(r'tools/rgs-shim-rust/src/registry.rs', 'r', encoding='utf-8') as f:
        text = f.read()
    import re
    print(f'File length: {len(text)}')
    print(f'Registry {{ map }} count: {text.count(marker)}')
    print(f'pub fn new() count: {len(re.findall(r"pub fn new\(\)", text))}')
    print(f'pub struct Registry count: {len(re.findall(r"pub struct Registry", text))}')
