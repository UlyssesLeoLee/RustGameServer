// extract-proto-mate.js — 提取 H5 真实 [游戏A]_client 协议字典
const fs = require('fs');
const path = 'E:/[跨盘-某发行商目录]/[游戏A]/server分析/[游戏A]_client_h5/temp/quick-scripts/src/assets/Scripts/net/proto_mate.js';
const content = fs.readFileSync(path, 'utf8');

// 提取 send = {...} 和 recv = {...} 块
const sendMatch = content.match(/module\.exports\.send\s*=\s*\{([\s\S]*?)\n\};/);
const recvMatch = content.match(/module\.exports\.recv\s*=\s*\{([\s\S]*?)\n\};/);

function extractCmds(text) {
  const cmds = new Set();
  const re = /^\s*(\d{5})\s*:/gm;
  let m;
  while ((m = re.exec(text)) !== null) cmds.add(parseInt(m[1]));
  return [...cmds].sort((a, b) => a - b);
}

const sendCmds = sendMatch ? extractCmds(sendMatch[1]) : [];
const recvCmds = recvMatch ? extractCmds(recvMatch[1]) : [];

console.log(`proto_mate.js H5 [游戏A]_client 协议字典 (per temp/quick-scripts/src/assets/Scripts/net/proto_mate.js)`);
console.log('');
console.log(`send (cli → srv) 唯一 cmd: ${sendCmds.length}`);
console.log(`recv (srv → cli) 唯一 cmd: ${recvCmds.length}`);
console.log('');
console.log('=== send cmds (cli, [游戏A]_client 发出的) ===');
console.log(sendCmds.join(', '));
console.log('');
console.log('=== recv cmds (srv, [游戏A]_client 接收的, 前 30) ===');
console.log(recvCmds.slice(0, 30).join(', '));
console.log('');
console.log('=== shim 已实现 ===');
const shimCmds = [10101, 10102, 10103, 10200, 10215, 10300, 10301, 10302, 10309, 10315];
const shimInternal = [10400, 11001];
console.log('real [游戏A] cmd: ' + shimCmds.join(', '));
console.log('shim-internal: ' + shimInternal.join(', '));
console.log('');
console.log('=== 缺口 (send cmds 减去已实现) ===');
const gap = sendCmds.filter(c => !shimCmds.includes(c));
console.log(`待实现: ${gap.length} cmd`);
console.log(gap.join(', '));
