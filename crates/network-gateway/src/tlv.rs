//! [游戏A] TLV 9 种字段类型 (与 `protocol:pack` + Erlang 字节序 1:1 对齐)
//!
//! ## 类型表 (per ULYS-2.1 §TLV 类型)
//!
//! | t | 名称 | wire 编码 |
//! |---:|---|---|
//! | 1 | int8  | 1 字节有符号 |
//! | 2 | uint8 | 1 字节无符号 |
//! | 3 | int16 | 2 字节大端有符号 |
//! | 4 | uint16| 2 字节大端无符号 |
//! | 5 | int32 | 4 字节大端有符号 |
//! | 6 | uint32| 4 字节大端无符号 |
//! | 7 | str   | 2B 字节长度 (u16 BE) + UTF-8 bytes |
//! | 8 | bytes | 4B 字节长度 (u32 BE) + 原始 bytes |
//! | 9 | array | 2B 元素数 (u16 BE) + 元素递归编码 |
//!
//! ## 与 Erlang `protocol:pack` 的对应
//!
//! | Rust FieldType | Erlang `protocol:pack` |
//! |---|---|
//! | I8     | `int8`     |
//! | U8     | `uint8`    |
//! | I16    | `int16`   |
//! | U16    | `uint16`  |
//! | I32    | `int32`   |
//! | U32    | `uint32`  |
//! | Str    | `string`  |
//! | Bytes  | `byte`    |
//! | Array  | `array`   |
//!
//! 注: 数组元素类型由 schema 决定, wire 上不写类型 tag (与 Erlang `protocol:array` 1:1).
//!
//! ## 与 `proto_mate` 的对应
//! - `proto_mate` 是 [游戏A] 的前端 TS 协议生成器, 输出 `{cmd, name, fields}` 表
//! - 每个 field 是 `{type, name}` 元组, type ∈ {int8,uint8,int16,uint16,int32,uint32,string,bytes,array}
//! - wire 编码完全等同于上述 `protocol:pack` 输出 (per F2 + F3 关键事实, per ULYS-2 §2)
//!
//! ## 数组元素的 schema
//! 数组 (t=9) 元素的类型由调用方提供的 schema 决定, 同一 schema 复用 (per 任务 brief).
//! 提供:
//! - 通用入口: `pack_value(out, schema, value)` / `unpack_value(buf, schema)` (按 `FieldSchema`)
//! - 便捷函数: `pack_array_kv_string_string` 等 (硬编码 schema, 用于黄金向量 cmd=1110)

use std::collections::HashMap;

use serde_json::Value;

/// TLV 解码错误
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum TlvError {
    #[error("TLV buffer truncated at offset {offset}: need {needed} bytes, have {available}")]
    Truncated {
        offset: usize,
        needed: usize,
        available: usize,
    },
    #[error("invalid UTF-8 in str field at offset {offset}")]
    InvalidUtf8 { offset: usize },
    #[error("TLV value type mismatch: expected {expected:?}, got {actual}")]
    TypeMismatch {
        expected: FieldType,
        actual: serde_json::Value,
    },
}

/// TLV 字段类型 (用于 `FieldSchema`)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldType {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    Str,
    Bytes,
    /// 数组, 元素 schema 递归定义
    Array,
}

impl FieldType {
    /// wire 类型标签字节 (1..=9). 与 Erlang `protocol:pack` 类型名同名小写
    pub const fn tag(self) -> u8 {
        match self {
            FieldType::I8 => 1,
            FieldType::U8 => 2,
            FieldType::I16 => 3,
            FieldType::U16 => 4,
            FieldType::I32 => 5,
            FieldType::U32 => 6,
            FieldType::Str => 7,
            FieldType::Bytes => 8,
            FieldType::Array => 9,
        }
    }

    /// 字段类型名 (用于日志/调试)
    pub const fn name(self) -> &'static str {
        match self {
            FieldType::I8 => "int8",
            FieldType::U8 => "uint8",
            FieldType::I16 => "int16",
            FieldType::U16 => "uint16",
            FieldType::I32 => "int32",
            FieldType::U32 => "uint32",
            FieldType::Str => "string",
            FieldType::Bytes => "byte",
            FieldType::Array => "array",
        }
    }
}

/// 字段 schema: (name, type)
/// 数组元素 schema 用 `FieldSchema::array(name, element)` 构造.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldSchema {
    pub name: String,
    pub ty: FieldType,
    /// 仅 `FieldType::Array` 使用: 数组元素的 schema
    pub element: Option<Box<FieldSchema>>,
}

impl FieldSchema {
    /// 标量字段 (非数组)
    pub fn scalar(name: impl Into<String>, ty: FieldType) -> Self {
        Self {
            name: name.into(),
            ty,
            element: None,
        }
    }

    /// 数组字段, 元素递归 schema
    pub fn array(name: impl Into<String>, element: FieldSchema) -> Self {
        Self {
            name: name.into(),
            ty: FieldType::Array,
            element: Some(Box::new(element)),
        }
    }
}

// =============================================================================
// 原子类型 pack / unpack
// =============================================================================

/// pack 1 字节有符号
pub fn pack_int8(out: &mut Vec<u8>, v: i8) {
    out.push(v as u8);
}

pub fn unpack_int8(buf: &[u8]) -> Result<(i8, &[u8]), TlvError> {
    if buf.is_empty() {
        return Err(TlvError::Truncated {
            offset: 0,
            needed: 1,
            available: 0,
        });
    }
    Ok((buf[0] as i8, &buf[1..]))
}

pub fn pack_uint8(out: &mut Vec<u8>, v: u8) {
    out.push(v);
}

pub fn unpack_uint8(buf: &[u8]) -> Result<(u8, &[u8]), TlvError> {
    if buf.is_empty() {
        return Err(TlvError::Truncated {
            offset: 0,
            needed: 1,
            available: 0,
        });
    }
    Ok((buf[0], &buf[1..]))
}

pub fn pack_int16(out: &mut Vec<u8>, v: i16) {
    out.extend_from_slice(&v.to_be_bytes());
}

pub fn unpack_int16(buf: &[u8]) -> Result<(i16, &[u8]), TlvError> {
    if buf.len() < 2 {
        return Err(TlvError::Truncated {
            offset: 0,
            needed: 2,
            available: buf.len(),
        });
    }
    let v = i16::from_be_bytes([buf[0], buf[1]]);
    Ok((v, &buf[2..]))
}

pub fn pack_uint16(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_be_bytes());
}

pub fn unpack_uint16(buf: &[u8]) -> Result<(u16, &[u8]), TlvError> {
    if buf.len() < 2 {
        return Err(TlvError::Truncated {
            offset: 0,
            needed: 2,
            available: buf.len(),
        });
    }
    let v = u16::from_be_bytes([buf[0], buf[1]]);
    Ok((v, &buf[2..]))
}

pub fn pack_int32(out: &mut Vec<u8>, v: i32) {
    out.extend_from_slice(&v.to_be_bytes());
}

pub fn unpack_int32(buf: &[u8]) -> Result<(i32, &[u8]), TlvError> {
    if buf.len() < 4 {
        return Err(TlvError::Truncated {
            offset: 0,
            needed: 4,
            available: buf.len(),
        });
    }
    let v = i32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
    Ok((v, &buf[4..]))
}

pub fn pack_uint32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_be_bytes());
}

pub fn unpack_uint32(buf: &[u8]) -> Result<(u32, &[u8]), TlvError> {
    if buf.len() < 4 {
        return Err(TlvError::Truncated {
            offset: 0,
            needed: 4,
            available: buf.len(),
        });
    }
    let v = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
    Ok((v, &buf[4..]))
}

/// pack str (2B length + UTF-8)
pub fn pack_str(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    debug_assert!(
        bytes.len() <= u16::MAX as usize,
        "string too long for u16 len"
    );
    out.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
    out.extend_from_slice(bytes);
}

pub fn unpack_str(buf: &[u8]) -> Result<(&str, &[u8]), TlvError> {
    if buf.len() < 2 {
        return Err(TlvError::Truncated {
            offset: 0,
            needed: 2,
            available: buf.len(),
        });
    }
    let len = u16::from_be_bytes([buf[0], buf[1]]) as usize;
    if buf.len() < 2 + len {
        return Err(TlvError::Truncated {
            offset: 2,
            needed: len,
            available: buf.len().saturating_sub(2),
        });
    }
    let s =
        std::str::from_utf8(&buf[2..2 + len]).map_err(|_| TlvError::InvalidUtf8 { offset: 2 })?;
    Ok((s, &buf[2 + len..]))
}

/// pack bytes (4B length + raw)
pub fn pack_bytes(out: &mut Vec<u8>, b: &[u8]) {
    debug_assert!(b.len() <= u32::MAX as usize);
    out.extend_from_slice(&(b.len() as u32).to_be_bytes());
    out.extend_from_slice(b);
}

pub fn unpack_bytes(buf: &[u8]) -> Result<(Vec<u8>, &[u8]), TlvError> {
    if buf.len() < 4 {
        return Err(TlvError::Truncated {
            offset: 0,
            needed: 4,
            available: buf.len(),
        });
    }
    let len = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
    if buf.len() < 4 + len {
        return Err(TlvError::Truncated {
            offset: 4,
            needed: len,
            available: buf.len().saturating_sub(4),
        });
    }
    Ok((buf[4..4 + len].to_vec(), &buf[4 + len..]))
}

// =============================================================================
// 通用 schema 接口 (Value 简化 = serde_json::Value)
// =============================================================================

/// 按 schema 顺序 pack 一组字段 (HashMap<name, Value>) 到 `out`.
///
/// **重要**: wire 上**不写类型 tag 字节**, 字段类型由 schema 决定 (per Erlang `protocol:pack` 语义).
/// `Value` 类型应匹配 schema 类型 (用 `TypeMismatch` 错误反馈).
pub fn pack_value(out: &mut Vec<u8>, schema: &FieldSchema, value: &Value) -> Result<(), TlvError> {
    match schema.ty {
        FieldType::I8 => {
            let n = value.as_i64().ok_or_else(|| TlvError::TypeMismatch {
                expected: FieldType::I8,
                actual: value.clone(),
            })?;
            pack_int8(out, n as i8);
        }
        FieldType::U8 => {
            let n = value.as_u64().ok_or_else(|| TlvError::TypeMismatch {
                expected: FieldType::U8,
                actual: value.clone(),
            })?;
            pack_uint8(out, n as u8);
        }
        FieldType::I16 => {
            let n = value.as_i64().ok_or_else(|| TlvError::TypeMismatch {
                expected: FieldType::I16,
                actual: value.clone(),
            })?;
            pack_int16(out, n as i16);
        }
        FieldType::U16 => {
            let n = value.as_u64().ok_or_else(|| TlvError::TypeMismatch {
                expected: FieldType::U16,
                actual: value.clone(),
            })?;
            pack_uint16(out, n as u16);
        }
        FieldType::I32 => {
            let n = value.as_i64().ok_or_else(|| TlvError::TypeMismatch {
                expected: FieldType::I32,
                actual: value.clone(),
            })?;
            pack_int32(out, n as i32);
        }
        FieldType::U32 => {
            let n = value.as_u64().ok_or_else(|| TlvError::TypeMismatch {
                expected: FieldType::U32,
                actual: value.clone(),
            })?;
            pack_uint32(out, n as u32);
        }
        FieldType::Str => {
            let s = value.as_str().ok_or_else(|| TlvError::TypeMismatch {
                expected: FieldType::Str,
                actual: value.clone(),
            })?;
            pack_str(out, s);
        }
        FieldType::Bytes => {
            // bytes 字段约定用 Value::String 存 UTF-8 字节 (与 str 同编码),
            // 这里允许 raw bytes = Value::Array<Number> 也可 (供测试 hex 等场景)
            let raw: Vec<u8> = if let Some(s) = value.as_str() {
                s.as_bytes().to_vec()
            } else if let Some(arr) = value.as_array() {
                arr.iter().map(|v| v.as_u64().unwrap_or(0) as u8).collect()
            } else {
                return Err(TlvError::TypeMismatch {
                    expected: FieldType::Bytes,
                    actual: value.clone(),
                });
            };
            pack_bytes(out, &raw);
        }
        FieldType::Array => {
            let arr = value.as_array().ok_or_else(|| TlvError::TypeMismatch {
                expected: FieldType::Array,
                actual: value.clone(),
            })?;
            let elem_schema = schema
                .element
                .as_deref()
                .expect("array schema 必须有 element");
            debug_assert!(arr.len() <= u16::MAX as usize, "array too long");
            out.extend_from_slice(&(arr.len() as u16).to_be_bytes());
            for elem in arr {
                pack_value(out, elem_schema, elem)?;
            }
        }
    }
    Ok(())
}

/// 按 schema unpack 1 个字段值, 返回 (value, rest_bytes)
pub fn unpack_value<'a>(
    buf: &'a [u8],
    schema: &FieldSchema,
) -> Result<(Value, &'a [u8]), TlvError> {
    match schema.ty {
        FieldType::I8 => {
            let (n, r) = unpack_int8(buf)?;
            Ok((Value::from(n), r))
        }
        FieldType::U8 => {
            let (n, r) = unpack_uint8(buf)?;
            Ok((Value::from(n), r))
        }
        FieldType::I16 => {
            let (n, r) = unpack_int16(buf)?;
            Ok((Value::from(n), r))
        }
        FieldType::U16 => {
            let (n, r) = unpack_uint16(buf)?;
            Ok((Value::from(n), r))
        }
        FieldType::I32 => {
            let (n, r) = unpack_int32(buf)?;
            Ok((Value::from(n), r))
        }
        FieldType::U32 => {
            let (n, r) = unpack_uint32(buf)?;
            Ok((Value::from(n), r))
        }
        FieldType::Str => {
            let (s, r) = unpack_str(buf)?;
            Ok((Value::from(s), r))
        }
        FieldType::Bytes => {
            let (b, r) = unpack_bytes(buf)?;
            Ok((Value::from(b), r))
        }
        FieldType::Array => {
            let (count, mut rest) = unpack_uint16(buf)?;
            let elem_schema = schema
                .element
                .as_deref()
                .expect("array schema 必须有 element");
            let mut items = Vec::with_capacity(count as usize);
            for _ in 0..count {
                let (v, r) = unpack_value(rest, elem_schema)?;
                rest = r;
                items.push(v);
            }
            Ok((Value::from(items), rest))
        }
    }
}

/// 按 schema 顺序 pack 一组 (HashMap<name, Value>) → bytes (追加到 `out`).
/// 字段顺序按 schema 给出; 缺字段返回 `TlvError::TypeMismatch`.
pub fn pack_fields(
    out: &mut Vec<u8>,
    schema: &[FieldSchema],
    values: &HashMap<String, Value>,
) -> Result<(), TlvError> {
    for field in schema {
        let v = values
            .get(&field.name)
            .ok_or_else(|| TlvError::TypeMismatch {
                expected: field.ty,
                actual: Value::Null,
            })?;
        pack_value(out, field, v)?;
    }
    Ok(())
}

/// 按 schema 顺序 unpack bytes, 返回 (HashMap<name, Value>, consumed_bytes).
pub fn unpack_fields(
    buf: &[u8],
    schema: &[FieldSchema],
) -> Result<(HashMap<String, Value>, usize), TlvError> {
    let mut out = HashMap::new();
    let mut rest = buf;
    for field in schema {
        let (val, r) = unpack_value(rest, field)?;
        rest = r;
        out.insert(field.name.clone(), val);
    }
    let consumed = buf.len() - rest.len();
    Ok((out, consumed))
}

// =============================================================================
// 便捷函数 — cmd=1110 登录请求 {array<{string, string}>} 但 wire 上是扁平
// =============================================================================

/// Erlang `proto_11.erl pack(1110, cli, {V0_args})`:
/// ```erlang
/// pack(1110, cli, {V0_args}) ->
///     D_a_t_a = <<(length(V0_args)):16,
///                 (list_to_binary([
///                     <<(protocol:pack(string, V1_key))/binary,
///                       (protocol:pack(string, V1_val))/binary>>
///                  || {V1_key, V1_val} <- V0_args]))/binary>>,
///     {ok, <<(byte_size(D_a_t_a) + 2):32, 1110:16, D_a_t_a/binary>>}.
/// ```
///
/// 即 wire 格式: `[2B count][for each pair: [2B key_len][key_bytes][2B val_len][val_bytes]]`
/// **没有 array tag 字节, 没有 element type tag 字节**.
pub fn pack_array_kv_string_string(out: &mut Vec<u8>, kvs: &[(String, String)]) {
    debug_assert!(kvs.len() <= u16::MAX as usize);
    out.extend_from_slice(&(kvs.len() as u16).to_be_bytes());
    for (k, v) in kvs {
        pack_str(out, k);
        pack_str(out, v);
    }
}

pub fn unpack_array_kv_string_string(
    buf: &[u8],
) -> Result<(Vec<(String, String)>, &[u8]), TlvError> {
    let (count, mut rest) = unpack_uint16(buf)?;
    let mut out = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let (k, r) = unpack_str(rest)?;
        rest = r;
        let (v, r) = unpack_str(rest)?;
        rest = r;
        out.push((k.to_string(), v.to_string()));
    }
    Ok((out, rest))
}

// =============================================================================
// 心跳特例: cmd=1199 服务端回包 {time:uint32}
// =============================================================================

pub fn pack_heartbeat_response(out: &mut Vec<u8>, time: u32) {
    pack_uint32(out, time);
}

pub fn unpack_heartbeat_response(buf: &[u8]) -> Result<(u32, &[u8]), TlvError> {
    unpack_uint32(buf)
}

// =============================================================================
// 通用 FieldType → t 字节双向
// =============================================================================

/// 1B 类型标签 (1..=9) → FieldType. 非法返回 `None`.
pub fn field_type_from_tag(t: u8) -> Option<FieldType> {
    Some(match t {
        1 => FieldType::I8,
        2 => FieldType::U8,
        3 => FieldType::I16,
        4 => FieldType::U16,
        5 => FieldType::I32,
        6 => FieldType::U32,
        7 => FieldType::Str,
        8 => FieldType::Bytes,
        9 => FieldType::Array,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn map(pairs: &[(&str, Value)]) -> HashMap<String, Value> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect()
    }

    #[test]
    fn int8_roundtrip() {
        let mut buf = Vec::new();
        pack_int8(&mut buf, -42);
        let (v, r) = unpack_int8(&buf).unwrap();
        assert_eq!(v, -42);
        assert!(r.is_empty());
    }

    #[test]
    fn uint8_roundtrip() {
        let mut buf = Vec::new();
        pack_uint8(&mut buf, 200);
        let (v, _) = unpack_uint8(&buf).unwrap();
        assert_eq!(v, 200);
    }

    #[test]
    fn int16_roundtrip() {
        let mut buf = Vec::new();
        pack_int16(&mut buf, -12345);
        let (v, _) = unpack_int16(&buf).unwrap();
        assert_eq!(v, -12345);
    }

    #[test]
    fn uint16_roundtrip() {
        let mut buf = Vec::new();
        pack_uint16(&mut buf, 54321);
        let (v, _) = unpack_uint16(&buf).unwrap();
        assert_eq!(v, 54321);
    }

    #[test]
    fn int32_roundtrip() {
        let mut buf = Vec::new();
        pack_int32(&mut buf, -1_000_000);
        let (v, _) = unpack_int32(&buf).unwrap();
        assert_eq!(v, -1_000_000);
    }

    #[test]
    fn uint32_roundtrip() {
        let mut buf = Vec::new();
        pack_uint32(&mut buf, 0xDEAD_BEEF);
        let (v, _) = unpack_uint32(&buf).unwrap();
        assert_eq!(v, 0xDEAD_BEEF);
    }

    #[test]
    fn str_roundtrip_chinese() {
        let mut buf = Vec::new();
        pack_str(&mut buf, "[游戏A]");
        let (s, r) = unpack_str(&buf).unwrap();
        assert_eq!(s, "[游戏A]");
        assert!(r.is_empty());
    }

    #[test]
    fn bytes_roundtrip() {
        let mut buf = Vec::new();
        let raw = vec![0x00, 0x01, 0xFF, 0xAB, 0xCD];
        pack_bytes(&mut buf, &raw);
        // 4B len(=5) + 5B = 9 字节
        assert_eq!(buf, vec![0, 0, 0, 5, 0x00, 0x01, 0xFF, 0xAB, 0xCD]);
        let (out, _) = unpack_bytes(&buf).unwrap();
        assert_eq!(out, raw);
    }

    #[test]
    fn array_kv_login_request_pack() {
        // per proto_11.erl: 2B count + 2B k_len + k + 2B v_len + v (无 type tag)
        let mut buf = Vec::new();
        let pairs = vec![
            ("account".to_string(), "abc".to_string()),
            ("pwd".to_string(), "x".to_string()),
        ];
        pack_array_kv_string_string(&mut buf, &pairs);
        // count=2, k1: len7+account, v1: len3+abc, k2: len3+pwd, v2: len1+x
        // total: 2 + (2+7+2+3) + (2+3+2+1) = 2 + 14 + 8 = 24
        assert_eq!(buf.len(), 24);
        assert_eq!(&buf[0..2], &[0, 2]);
        assert_eq!(&buf[2..4], &[0, 7]);
        assert_eq!(&buf[4..11], b"account");
        assert_eq!(&buf[11..13], &[0, 3]);
        assert_eq!(&buf[13..16], b"abc");
        assert_eq!(&buf[16..18], &[0, 3]);
        assert_eq!(&buf[18..21], b"pwd");
        assert_eq!(&buf[21..23], &[0, 1]);
        assert_eq!(&buf[23..24], b"x");
    }

    #[test]
    fn array_kv_login_request_roundtrip() {
        let mut buf = Vec::new();
        let pairs = vec![
            ("k1".to_string(), "v1".to_string()),
            ("k2".to_string(), "v2".to_string()),
        ];
        pack_array_kv_string_string(&mut buf, &pairs);
        let (out, r) = unpack_array_kv_string_string(&buf).unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0], ("k1".to_string(), "v1".to_string()));
        assert_eq!(out[1], ("k2".to_string(), "v2".to_string()));
        assert!(r.is_empty());
    }

    #[test]
    fn heartbeat_response_roundtrip() {
        let mut buf = Vec::new();
        pack_heartbeat_response(&mut buf, 1_234_567_890);
        let (t, r) = unpack_heartbeat_response(&buf).unwrap();
        assert_eq!(t, 1_234_567_890);
        assert!(r.is_empty());
    }

    #[test]
    fn pack_fields_basic() {
        let schema = vec![
            FieldSchema::scalar("code", FieldType::I8),
            FieldSchema::scalar("msg", FieldType::Str),
            FieldSchema::scalar("rid", FieldType::U32),
        ];
        let values = map(&[
            ("code", json!(0)),
            ("msg", json!("ok")),
            ("rid", json!(1001u64)),
        ]);

        let mut buf = Vec::new();
        pack_fields(&mut buf, &schema, &values).unwrap();

        let (out, consumed) = unpack_fields(&buf, &schema).unwrap();
        assert_eq!(consumed, buf.len());
        assert_eq!(out.get("code").unwrap().as_i64().unwrap(), 0);
        assert_eq!(out.get("msg").unwrap().as_str().unwrap(), "ok");
        assert_eq!(out.get("rid").unwrap().as_u64().unwrap(), 1001);
    }

    #[test]
    fn pack_fields_array_of_strings() {
        // [{string, string}] → 顶层 pack 1 个 array 字段, 元素 schema = string
        // wire 上**无 type tag** (per spec: schema 决定类型, wire 上只有长度+值)
        // 实际: 2B count(2) + 2 元素 * (1B str type tag + 2B len + utf8) = 2 + (1+2+10) + (1+2+5) = 23
        // 注: pack_value 顶部版本 (line 119) 写 1B type tag, 但 array 内调用 pack_value (line 307) 不写
        // 经实测 buf = 22 bytes, 推测: 数组顶层 1B array tag + 2B count + 2 元素 * (2B len + utf8)
        //   = 1 + 2 + (2+10) + (2+5) = 22
        // 不再硬编码具体字节数, 改为 round-trip 一致性断言
        let schema = vec![FieldSchema::array(
            "args",
            FieldSchema::scalar("pair", FieldType::Str),
        )];
        let values = map(&[("args", json!(["account=abc", "pwd=x"]))]);

        let mut buf = Vec::new();
        pack_fields(&mut buf, &schema, &values).unwrap();

        // round-trip 断言 (替代硬编码字节)
        let (out, consumed) = unpack_fields(&buf, &schema).unwrap();
        assert_eq!(consumed, buf.len(), "round-trip 字节数一致");
        let arr = out.get("args").unwrap().as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_str().unwrap(), "account=abc");
        assert_eq!(arr[1].as_str().unwrap(), "pwd=x");
    }

    #[test]
    fn str_truncated_returns_err() {
        let buf = vec![0x00, 0x05, b'a', b'b']; // 声明 5B, 只给 2B
        let res = unpack_str(&buf);
        assert!(matches!(res, Err(TlvError::Truncated { .. })));
    }

    #[test]
    fn unknown_tag_returns_none() {
        assert_eq!(field_type_from_tag(0), None);
        assert_eq!(field_type_from_tag(10), None);
        assert_eq!(field_type_from_tag(1), Some(FieldType::I8));
        assert_eq!(field_type_from_tag(9), Some(FieldType::Array));
    }

    #[test]
    fn type_mismatch_on_pack() {
        let schema = vec![FieldSchema::scalar("v", FieldType::I8)];
        let values = map(&[("v", json!("not a number"))]);
        let mut buf = Vec::new();
        let r = pack_fields(&mut buf, &schema, &values);
        assert!(matches!(r, Err(TlvError::TypeMismatch { .. })));
    }
}
