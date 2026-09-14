// tools/h5_e2e/zsyz_protocol.js
//
// Pure-JS reference implementation of the zsyz_client_h5 binary wire protocol,
// 1:1 with `assets/Scripts/sys/game-core-js-min.js` SmartSocket +
// `assets/Scripts/net/proto_mate.js` + zsyz_server `protocol:pack` (Erlang).
//
// Used by:
//   - tools/h5_e2e/mock_server.mjs  (decode incoming frame, encode reply)
//   - tools/h5_e2e/d4_heartbeat.mjs (build raw cmd=1199 frame, verify reply)
//   - tools/h5_e2e/e2e_login.mjs    (browser-side, exposed as gcore.SmartSocket)
//
// Reference (verified against game-core-js-min.js):
//   send(cmd, data):
//     i = packData(cmd, data)        // [cmd u16 BE][payload TLV...]
//     buf = ArrayBuffer(i.length + 4)
//     r.setUint32(0, i.length)        // length = cmd(2) + payload(N)
//     r.setUint8(4.., i...)           // cmd u16 BE + payload
//
//   unpackBuffer(e):
//     i = t.getUint32(0, false)       // length = cmd(2) + payload(N)
//     r = t.getUint16(4, false)       // cmd u16 BE
//     payload starts at offset 6, length = i - 2
//
// TLV types (t=1..9):
//   1 int8, 2 uint8, 3 int16 BE, 4 uint16 BE, 5 int32 BE, 6 uint32 BE,
//   7 str = [u16 len][utf8 bytes], 8 bytes = [u32 len][raw],
//   9 array = [u16 count][recursive element(s)]
//
// Per issue ULYS-6 (任务 D), tracking issue ULYS-2 (01a092ae-2faa-799b-b07e-ae6a9b80c166).
//
// All rights reserved by D-Boy (lidian727@gmail.com). 2026-09-12.

'use strict';

// ============================================================================
// Big-endian helpers (Buffer on Node, DataView on browser — both supported)
// ============================================================================
const DVT = typeof DataView !== 'undefined' ? DataView : null;

function readU16BE(buf, off) {
  if (DVT && buf instanceof ArrayBuffer) {
    return new DVT(buf).getUint16(off, false);
  }
  return ((buf[off] << 8) | buf[off + 1]) >>> 0;
}
function readI16BE(buf, off) {
  if (DVT && buf instanceof ArrayBuffer) {
    return new DVT(buf).getInt16(off, false);
  }
  const v = (buf[off] << 8) | buf[off + 1];
  return v & 0x8000 ? v - 0x10000 : v;
}
function readU32BE(buf, off) {
  if (DVT && buf instanceof ArrayBuffer) {
    return new DVT(buf).getUint32(off, false) >>> 0;
  }
  return ((buf[off] * 0x1000000) + ((buf[off + 1] << 16) | (buf[off + 2] << 8) | buf[off + 3])) >>> 0;
}
function readI32BE(buf, off) {
  if (DVT && buf instanceof ArrayBuffer) {
    return new DVT(buf).getInt32(off, false);
  }
  return (buf[off] << 24) | (buf[off + 1] << 16) | (buf[off + 2] << 8) | buf[off + 3];
}
function writeU16BE(out, v) {
  out.push((v >> 8) & 0xff, v & 0xff);
}
function writeI16BE(out, v) {
  // big-endian; sign-extended to 16 bits
  if (v < 0) v = 0x10000 + v;
  out.push((v >> 8) & 0xff, v & 0xff);
}
function writeU32BE(out, v) {
  out.push((v >>> 24) & 0xff, (v >>> 16) & 0xff, (v >>> 8) & 0xff, v & 0xff);
}
function writeI32BE(out, v) {
  if (v < 0) v = 0x100000000 + v;
  out.push((v >>> 24) & 0xff, (v >>> 16) & 0xff, (v >>> 8) & 0xff, v & 0xff);
}

// ============================================================================
// TLV pack (server → client)
// ============================================================================
// schema: [{s: name, t: 1..9, f?: [...]}]; f is array element schema (for t=9).
// values: { [name]: value }; array elements are objects per element schema.
function packFields(schema, values) {
  const out = [];
  for (let i = 0; i < schema.length; i++) {
    const f = schema[i];
    const v = values ? values[f.s] : undefined;
    packOne(out, f, v);
  }
  return Buffer.from(out);
}

function packOne(out, f, v) {
  switch (f.t) {
    case 1: // int8
      out.push((v | 0) & 0xff);
      break;
    case 2: // uint8
      out.push((v >>> 0) & 0xff);
      break;
    case 3: // int16 BE
      writeI16BE(out, v | 0);
      break;
    case 4: // uint16 BE
      writeU16BE(out, (v >>> 0) & 0xffff);
      break;
    case 5: // int32 BE
      writeI32BE(out, v | 0);
      break;
    case 6: // uint32 BE
      writeU32BE(out, v >>> 0);
      break;
    case 7: { // str = [u16 len][utf8 bytes]
      const s = v == null ? '' : String(v);
      // Match JS `unescape(encodeURIComponent(s))` semantics: encode UTF-8 manually.
      const enc = unescape(encodeURIComponent(s));
      const bytes = [];
      for (let i = 0; i < enc.length; i++) bytes.push(enc.charCodeAt(i) & 0xff);
      writeU16BE(out, bytes.length);
      for (let i = 0; i < bytes.length; i++) out.push(bytes[i]);
      break;
    }
    case 8: { // bytes = [u32 len][raw]
      const buf = v == null ? Buffer.alloc(0) : Buffer.isBuffer(v) ? v : Buffer.from(v);
      writeU32BE(out, buf.length);
      for (let i = 0; i < buf.length; i++) out.push(buf[i]);
      break;
    }
    case 9: { // array = [u16 count][each element is encoded per f.f schema]
      const arr = Array.isArray(v) ? v : [];
      writeU16BE(out, arr.length);
      const elemSchema = Array.isArray(f.f) ? f.f : [];
      for (let i = 0; i < arr.length; i++) {
        // Each element is an OBJECT whose fields are described by elemSchema.
        // packFields will iterate over elemSchema, calling packOne per field.
        const elemOut = [];
        for (let j = 0; j < elemSchema.length; j++) {
          packOne(elemOut, elemSchema[j], arr[i] ? arr[i][elemSchema[j].s] : undefined);
        }
        for (let k = 0; k < elemOut.length; k++) out.push(elemOut[k]);
      }
      break;
    }
    default:
      throw new Error('pack: unknown TLV type ' + f.t);
  }
}

// ============================================================================
// TLV unpack (client → server, or any direction)
// ============================================================================
// Returns { [name]: value, _next: nextOffset }
function unpackFields(schema, buf, off) {
  const out = {};
  for (let i = 0; i < schema.length; i++) {
    const f = schema[i];
    const r = unpackOne(buf, off, f);
    out[f.s] = r.v;
    off = r.next;
  }
  return out;
}

function unpackOne(buf, off, f) {
  switch (f.t) {
    case 1: { // int8
      const v = (buf[off] << 24) >> 24; // sign-extend
      return { v, next: off + 1 };
    }
    case 2: // uint8
      return { v: buf[off], next: off + 1 };
    case 3: // int16 BE
      return { v: readI16BE(buf, off), next: off + 2 };
    case 4: // uint16 BE
      return { v: readU16BE(buf, off), next: off + 2 };
    case 5: // int32 BE
      return { v: readI32BE(buf, off), next: off + 4 };
    case 6: // uint32 BE
      return { v: readU32BE(buf, off) >>> 0, next: off + 4 };
    case 7: { // str = [u16 len][utf8 bytes]
      const len = readU16BE(buf, off);
      off += 2;
      const bytes = buf.slice(off, off + len);
      // Mirror JS: `decodeURIComponent(escape(String.fromCharCode.apply(null, bytes)))`
      let s = '';
      for (let i = 0; i < bytes.length; i++) s += String.fromCharCode(bytes[i]);
      const decoded = decodeURIComponent(escape(s));
      return { v: decoded, next: off + len };
    }
    case 8: { // bytes = [u32 len][raw]
      const len = readU32BE(buf, off);
      off += 4;
      const slice = buf.slice(off, off + len);
      return { v: slice, next: off + len };
    }
    case 9: { // array = [u16 count][each element is a flat object per f.f schema]
      const count = readU16BE(buf, off);
      off += 2;
      const arr = [];
      const elemSchema = Array.isArray(f.f) ? f.f : [];
      for (let i = 0; i < count; i++) {
        const obj = {};
        for (let j = 0; j < elemSchema.length; j++) {
          const r = unpackOne(buf, off, elemSchema[j]);
          obj[elemSchema[j].s] = r.v;
          off = r.next;
        }
        arr.push(obj);
      }
      return { v: arr, next: off };
    }
    default:
      throw new Error('unpack: unknown TLV type ' + f.t);
  }
}

// ============================================================================
// Frame encode/decode (matches SmartSocket exactly)
// ============================================================================
// encodeFrame(cmd: u16, payload: Buffer) → Buffer
//   wire = [length u32 BE][cmd u16 BE][payload]
//   length = 2 (cmd bytes) + payload.length  (does NOT count the 4B length field itself)
function encodeFrame(cmd, payload) {
  const p = payload || Buffer.alloc(0);
  const out = Buffer.alloc(6 + p.length);
  out.writeUInt32BE(2 + p.length, 0);
  out.writeUInt16BE(cmd & 0xffff, 4);
  p.copy(out, 6);
  return out;
}

// decodeFrames(buf) → Frame[] (one or more complete frames); updates remaining in last element.
// returns { frames, remaining } where remaining is tail bytes after last complete frame.
function decodeFrames(buf) {
  const frames = [];
  let off = 0;
  while (true) {
    if (buf.length - off < 6) break;
    const length = buf.readUInt32BE(off);
    const total = 4 + length;
    if (buf.length - off < total) break;
    const cmd = buf.readUInt16BE(off + 4);
    const payload = buf.slice(off + 6, off + total);
    frames.push({ cmd, payload });
    off += total;
  }
  return { frames, remaining: buf.slice(off) };
}

// ============================================================================
// Heartbeat protocol (cmd=1199) — used by D4 raw-hex test
// ============================================================================
// Client request: empty payload `{}` → length=2 (cmd only), no payload bytes
// Server reply:   { time: u32 }    → length=6 (cmd 2 + payload 4), payload is u32 timestamp

const CMD_HEARTBEAT = 1199;
const CMD_LOGIN = 1110;

function buildHeartbeatFrame() {
  return encodeFrame(CMD_HEARTBEAT, Buffer.alloc(0));
}

function parseHeartbeatReply(payload) {
  // payload = [u32 time] (BE)
  if (payload.length < 4) throw new Error('heartbeat reply too short: ' + payload.length);
  return { time: payload.readUInt32BE(0) };
}

// ============================================================================
// Login reply (cmd=1110) — schema per proto_11.erl (estimated from API list)
// srv: code:u8, msg:str, roles:array(string), least_career:u8
// ============================================================================
const LOGIN_REPLY_SCHEMA = [
  { s: 'code', t: 2 },
  { s: 'msg', t: 7 },
  { s: 'roles', t: 9, f: [{ s: 'name', t: 7 }] }, // simplified
  { s: 'least_career', t: 2 },
];

function buildLoginReply({ code = 0, msg = '', roles = [], least_career = 0 } = {}) {
  return encodeFrame(CMD_LOGIN, packFields(LOGIN_REPLY_SCHEMA, {
    code, msg,
    roles: roles.map((r) => ({ name: r })),
    least_career,
  }));
}

// ============================================================================
// Module exports (CommonJS for Node, plus globals for browser)
// ============================================================================
if (typeof module !== 'undefined' && module.exports) {
  module.exports = {
    encodeFrame,
    decodeFrames,
    packFields,
    unpackFields,
    buildHeartbeatFrame,
    parseHeartbeatReply,
    buildLoginReply,
    LOGIN_REPLY_SCHEMA,
    CMD_HEARTBEAT,
    CMD_LOGIN,
    // helpers exposed for tests
    _readU16BE: readU16BE,
    _readU32BE: readU32BE,
  };
}
if (typeof globalThis !== 'undefined') {
  globalThis.ZSYZ = {
    encodeFrame,
    decodeFrames,
    packFields,
    unpackFields,
    buildHeartbeatFrame,
    parseHeartbeatReply,
    buildLoginReply,
    CMD_HEARTBEAT,
    CMD_LOGIN,
  };
}