// tools/h5_e2e/zsyz_protocol.browser.js
//
// Browser-friendly version of zsyz_protocol.js. Same semantics, but uses
// Uint8Array + DataView (browser-native) instead of Buffer (Node-only).
// Loaded by smart_socket_test.html via <script src=>.
//
// Reference (same as Node version): zsyz_client_h5 SmartSocket in
// assets/Scripts/sys/game-core-js-min.js.
//
// Per issue ULYS-6 (任务 D).

(function (root) {
  'use strict';

  // ---------- big-endian helpers on Uint8Array ----------
  function readU16BE(u8, off) {
    return ((u8[off] << 8) | u8[off + 1]) >>> 0;
  }
  function readI16BE(u8, off) {
    let v = (u8[off] << 8) | u8[off + 1];
    return v & 0x8000 ? v - 0x10000 : v;
  }
  function readU32BE(u8, off) {
    return ((u8[off] * 0x1000000) + ((u8[off + 1] << 16) | (u8[off + 2] << 8) | u8[off + 3])) >>> 0;
  }
  function readI32BE(u8, off) {
    return (u8[off] << 24) | (u8[off + 1] << 16) | (u8[off + 2] << 8) | u8[off + 3];
  }
  function writeU16BE(out, v) {
    out.push((v >> 8) & 0xff, v & 0xff);
  }
  function writeI16BE(out, v) {
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

  // ---------- TLV pack ----------
  function packOne(out, f, v) {
    switch (f.t) {
      case 1: out.push((v | 0) & 0xff); break;
      case 2: out.push((v >>> 0) & 0xff); break;
      case 3: writeI16BE(out, v | 0); break;
      case 4: writeU16BE(out, (v >>> 0) & 0xffff); break;
      case 5: writeI32BE(out, v | 0); break;
      case 6: writeU32BE(out, v >>> 0); break;
      case 7: {
        const s = v == null ? '' : String(v);
        const enc = unescape(encodeURIComponent(s));
        const bytes = [];
        for (let i = 0; i < enc.length; i++) bytes.push(enc.charCodeAt(i) & 0xff);
        writeU16BE(out, bytes.length);
        for (let i = 0; i < bytes.length; i++) out.push(bytes[i]);
        break;
      }
      case 8: {
        const u8 = v instanceof Uint8Array ? v : new Uint8Array(v || []);
        writeU32BE(out, u8.length);
        for (let i = 0; i < u8.length; i++) out.push(u8[i]);
        break;
      }
      case 9: {
        const arr = Array.isArray(v) ? v : [];
        writeU16BE(out, arr.length);
        const elemSchema = Array.isArray(f.f) ? f.f : [];
        for (let i = 0; i < arr.length; i++) {
          const elemOut = [];
          for (let j = 0; j < elemSchema.length; j++) {
            packOne(elemOut, elemSchema[j], arr[i] ? arr[i][elemSchema[j].s] : undefined);
          }
          for (let k = 0; k < elemOut.length; k++) out.push(elemOut[k]);
        }
        break;
      }
      default: throw new Error('pack: unknown TLV type ' + f.t);
    }
  }

  function packFields(schema, values) {
    const out = [];
    for (let i = 0; i < schema.length; i++) {
      packOne(out, schema[i], values ? values[schema[i].s] : undefined);
    }
    return new Uint8Array(out);
  }

  function unpackOne(u8, off, f) {
    switch (f.t) {
      case 1: {
        const v = (u8[off] << 24) >> 24;
        return { v, next: off + 1 };
      }
      case 2: return { v: u8[off], next: off + 1 };
      case 3: return { v: readI16BE(u8, off), next: off + 2 };
      case 4: return { v: readU16BE(u8, off), next: off + 2 };
      case 5: return { v: readI32BE(u8, off), next: off + 4 };
      case 6: return { v: readU32BE(u8, off) >>> 0, next: off + 4 };
      case 7: {
        const len = readU16BE(u8, off);
        off += 2;
        let s = '';
        for (let i = 0; i < len; i++) s += String.fromCharCode(u8[off + i]);
        return { v: decodeURIComponent(escape(s)), next: off + len };
      }
      case 8: {
        const len = readU32BE(u8, off);
        off += 4;
        return { v: u8.slice(off, off + len), next: off + len };
      }
      case 9: {
        const count = readU16BE(u8, off);
        off += 2;
        const arr = [];
        const elemSchema = Array.isArray(f.f) ? f.f : [];
        for (let i = 0; i < count; i++) {
          const obj = {};
          for (let j = 0; j < elemSchema.length; j++) {
            const r = unpackOne(u8, off, elemSchema[j]);
            obj[elemSchema[j].s] = r.v;
            off = r.next;
          }
          arr.push(obj);
        }
        return { v: arr, next: off };
      }
      default: throw new Error('unpack: unknown TLV type ' + f.t);
    }
  }

  function unpackFields(schema, u8, off) {
    const out = {};
    for (let i = 0; i < schema.length; i++) {
      const r = unpackOne(u8, off, schema[i]);
      out[schema[i].s] = r.v;
      off = r.next;
    }
    return out;
  }

  // ---------- frame ----------
  function encodeFrame(cmd, payload) {
    const p = payload || new Uint8Array(0);
    const out = new Uint8Array(6 + p.length);
    // length = cmd(2) + payload(N); 4B length field
    out[0] = ((2 + p.length) >>> 24) & 0xff;
    out[1] = ((2 + p.length) >>> 16) & 0xff;
    out[2] = ((2 + p.length) >>> 8) & 0xff;
    out[3] = (2 + p.length) & 0xff;
    out[4] = (cmd >> 8) & 0xff;
    out[5] = cmd & 0xff;
    out.set(p, 6);
    return out.buffer;
  }

  function decodeOneFrame(u8, off) {
    if (u8.length - off < 6) return null;
    const length = readU32BE(u8, off);
    const total = 4 + length;
    if (u8.length - off < total) return null;
    const cmd = readU16BE(u8, off + 4);
    return { cmd, payload: u8.slice(off + 6, off + total), next: off + total };
  }

  // ---------- protocol-specific helpers ----------
  const CMD_HEARTBEAT = 1199;
  const CMD_LOGIN = 1110;

  // cmd=1199 client send: empty payload (length=2, just cmd bytes)
  function buildHeartbeatPayload() { return new Uint8Array(0); }

  // cmd=1199 server reply: { time: u32 } → length=6, payload=u32 BE
  function parseHeartbeatPayload(u8) {
    if (u8.length < 4) throw new Error('heartbeat reply too short');
    return { time: readU32BE(u8, 0) };
  }

  // cmd=1110 client send (per proto_11.erl pack):
  //   args: array<{ key: str, val: str }>
  function buildLoginPayload(data) {
    const args = (data && data.args) || [];
    return packFields([
      { s: 'args', t: 9, f: [{ s: 'key', t: 7 }, { s: 'val', t: 7 }] }
    ], { args });
  }

  // cmd=1110 server reply:
  //   code:u8, msg:str, roles:array<{name:str}>, least_career:u8
  const LOGIN_REPLY_SCHEMA = [
    { s: 'code', t: 2 },
    { s: 'msg', t: 7 },
    { s: 'roles', t: 9, f: [{ s: 'name', t: 7 }] },
    { s: 'least_career', t: 2 },
  ];
  function parseLoginPayload(u8) {
    return unpackFields(LOGIN_REPLY_SCHEMA, u8, 0);
  }

  // dispatch helper used by smart_socket_test.html
  function packPayload(cmd, data) {
    switch (cmd) {
      case CMD_HEARTBEAT: return buildHeartbeatPayload();
      case CMD_LOGIN: return buildLoginPayload(data);
      default: return new Uint8Array(0);
    }
  }
  function unpackPayload(cmd, u8, off, len) {
    const view = (off === 0 && len === u8.length) ? u8 : u8.slice(off, off + len);
    switch (cmd) {
      case CMD_HEARTBEAT: return parseHeartbeatPayload(view);
      case CMD_LOGIN: return parseLoginPayload(view);
      default: return {};
    }
  }

  root.ZSYZ = {
    encodeFrame,
    decodeOneFrame,
    packFields,
    unpackFields,
    packPayload,
    unpackPayload,
    buildHeartbeatPayload,
    parseHeartbeatPayload,
    buildLoginPayload,
    parseLoginPayload,
    CMD_HEARTBEAT,
    CMD_LOGIN,
    LOGIN_REPLY_SCHEMA,
  };
})(typeof window !== 'undefined' ? window : globalThis);