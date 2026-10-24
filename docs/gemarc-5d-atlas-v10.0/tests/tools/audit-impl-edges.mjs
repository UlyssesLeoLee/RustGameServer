/** 監査用: Implemented 辺の「実使用点」が本当に使用点か (use インポートや文字列リテラルを誤認していないか) */
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

const M = join(process.cwd(), 'models');
const arch = JSON.parse(readFileSync(join(M, 'architecture.json'), 'utf8'));
const ev = JSON.parse(readFileSync(join(M, 'evidence.json'), 'utf8'));
const map = new Map(ev.records.map((e) => [e.id, e]));

let bad = 0;
console.log('=== Implemented 辺の実使用点形态 ===');
arch.edges.forEach((e, i) => {
  const rec = map.get(e.evidenceIds[0]);
  const line = (rec?.snippet ?? '').trim();
  let flag = '';
  if (/^use\s/.test(line)) { flag = '  <== use インポート (呼び出し点ではない)'; bad++; }
  else if (/^["']/.test(line)) { flag = '  <== 文字列リテラル (呼び出し点ではない)'; bad++; }
  else if (/\/\//.test(line) && !/^\/\//.test(line) === false) { /* noop */ }
  console.log(
    `${String(i + 1).padStart(3)}. ${e.properties.callerCrate.padEnd(20)} -> ${e.properties.calleeCrate.padEnd(16)} [${e.properties.usageForm.padEnd(16)}] ${line.slice(0, 76)}${flag}`,
  );
});
console.log(`\n誤認疑い: ${bad} / ${arch.edges.length}`);
