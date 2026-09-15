// tools/h5_e2e/_smoke_codec.js — quick round-trip test for zsyz_protocol.js
'use strict';

const {
  encodeFrame, decodeFrames, packFields, unpackFields,
  buildHeartbeatFrame, parseHeartbeatReply, buildLoginReply,
  CMD_HEARTBEAT, CMD_LOGIN,
} = require('./zsyz_protocol.js');

function hex(b) {
  return Buffer.from(b).toString('hex').match(/.{1,2}/g).join(' ');
}

let pass = 0, fail = 0;
function assert(cond, msg) {
  if (cond) { pass++; console.log('  ✓', msg); }
  else { fail++; console.error('  ✗', msg); }
}

console.log('=== test 1: heartbeat frame roundtrip (cmd=1199, empty payload) ===');
const hb = buildHeartbeatFrame();
console.log('  encoded:', hex(hb));
assert(hb.length === 6, 'encoded length = 6 (4B length + 2B cmd)');
assert(hb.toString('hex') === '00000002 04af'.replace(/\s+/g, ''), 'hex == 00 00 00 02 04 af');
const { frames, remaining } = decodeFrames(hb);
assert(remaining.length === 0, 'no remaining bytes');
assert(frames.length === 1 && frames[0].cmd === CMD_HEARTBEAT, 'decoded cmd = 1199');
assert(frames[0].payload.length === 0, 'empty payload');

console.log('=== test 2: heartbeat reply (cmd=1199, u32 time) ===');
// Build a reply: length=6, cmd=1199, payload=[u32 BE time]
const fakeReply = Buffer.alloc(10);
fakeReply.writeUInt32BE(6, 0); // length = 6 (2 cmd + 4 payload)
fakeReply.writeUInt16BE(CMD_HEARTBEAT, 4);
fakeReply.writeUInt32BE(1700000000, 6);
const { frames: fr2 } = decodeFrames(fakeReply);
const reply = parseHeartbeatReply(fr2[0].payload);
assert(reply.time === 1700000000, 'reply.time == 1700000000');

console.log('=== test 3: 9 TLV pack/unpack roundtrip ===');
const schema = [
  { s: 'a', t: 1 }, // int8
  { s: 'b', t: 2 }, // uint8
  { s: 'c', t: 3 }, // int16
  { s: 'd', t: 4 }, // uint16
  { s: 'e', t: 5 }, // int32
  { s: 'f', t: 6 }, // uint32
  { s: 'g', t: 7 }, // str
  { s: 'h', t: 8 }, // bytes
  { s: 'i', t: 9, f: [{ s: 'k', t: 7 }, { s: 'v', t: 6 }] }, // array of obj
];
const vals = {
  a: -5, b: 200, c: -32000, d: 65000, e: -2000000000, f: 4000000000,
  g: 'héllo🚀', h: Buffer.from([1, 2, 3, 4]),
  i: [{ k: 'x', v: 1 }, { k: 'yy', v: 2 }],
};
const packed = packFields(schema, vals);
console.log('  packed hex:', hex(packed));
const unpacked = unpackFields(schema, packed, 0);
assert(unpacked.a === -5, 'int8 roundtrip');
assert(unpacked.b === 200, 'uint8 roundtrip');
assert(unpacked.c === -32000, 'int16 roundtrip');
assert(unpacked.d === 65000, 'uint16 roundtrip');
assert(unpacked.e === -2000000000, 'int32 roundtrip');
assert(unpacked.f === 4000000000, 'uint32 roundtrip');
assert(unpacked.g === 'héllo🚀', 'utf8 string roundtrip');
assert(Buffer.from(unpacked.h).equals(Buffer.from([1, 2, 3, 4])), 'bytes roundtrip');
assert(unpacked.i.length === 2 && unpacked.i[0].k === 'x' && unpacked.i[1].v === 2, 'array roundtrip');

console.log('=== test 4: login reply (cmd=1110) ===');
const reply2 = buildLoginReply({ code: 0, msg: 'OK', roles: ['Alice', 'Bob'], least_career: 1 });
console.log('  encoded:', hex(reply2));
const { frames: fr3 } = decodeFrames(reply2);
assert(fr3[0].cmd === CMD_LOGIN, 'cmd = 1110');
const dec = unpackFields([
  { s: 'code', t: 2 },
  { s: 'msg', t: 7 },
  { s: 'roles', t: 9, f: [{ s: 'name', t: 7 }] },
  { s: 'least_career', t: 2 },
], fr3[0].payload, 0);
assert(dec.code === 0, 'login code = 0');
assert(dec.msg === 'OK', 'login msg = OK');
assert(dec.roles.length === 2 && dec.roles[0].name === 'Alice', 'roles roundtrip');
assert(dec.least_career === 1, 'least_career = 1');

console.log('=== test 5: golden vector for cmd=1199 per ULYS-2.1 ===');
// From ULYS-2.1: 客户端发 00 00 00 02 04 AF → 期望回 00 00 00 06 04 AF <u32 time>
const golden = buildHeartbeatFrame();
assert(golden.toString('hex') === '0000000204af', 'golden vector 1 (client send) match');

console.log(`\nResult: ${pass} pass, ${fail} fail`);
process.exit(fail === 0 ? 0 : 1);