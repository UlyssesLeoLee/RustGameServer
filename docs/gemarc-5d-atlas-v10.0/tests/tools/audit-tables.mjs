/** 監査: 幻影テーブル ("IF" など) と重複 qualifiedName の残存確認 */
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

const M = join(process.cwd(), 'models');
const arch = JSON.parse(readFileSync(join(M, 'architecture.json'), 'utf8'));
const tbl = arch.nodes.filter((n) => n.kind === 'Table');

const phantom = tbl.filter((t) => /^(IF|NOT|EXISTS|TABLE|ONLY)$/i.test(t.name));
const byQ = new Map();
for (const t of tbl) byQ.set(t.qualifiedName, (byQ.get(t.qualifiedName) ?? 0) + 1);
const dups = [...byQ.entries()].filter(([, c]) => c > 1);

const noPk = tbl.filter((t) => !(t.properties.pkColumns ?? []).length);
const withFk = tbl.filter((t) => (t.properties.fkColumns ?? []).length);
const pkInType = tbl.filter((t) => (t.properties.columns ?? []).some((c) => c.pk)).length;

console.log(`表节点総数        : ${tbl.length}`);
console.log(`幻影名 (IF 等)     : ${phantom.length} ${phantom.map((t) => t.name).join(',') || '✅ なし'}`);
console.log(`qualifiedName 重複 : ${dups.length} ${dups.map(([k, c]) => `${k}×${c}`).join(', ') || '✅ なし'}`);
console.log(`PK を持つ表        : ${noPk.length === 0 ? tbl.length : tbl.length - noPk.length} / ${tbl.length}`);
console.log(`PK 列検出の一致    : pkColumns 保持 ${tbl.filter((t) => (t.properties.pkColumns ?? []).length).length} / columns[].pk 保持 ${pkInType}`);
console.log(`FK を持つ表        : ${withFk.length} (${withFk.map((t) => t.name).join(', ')})`);
console.log(`\nFK 表のモデル内位置: ${withFk.map((t) => `${t.name}=#${tbl.findIndex((x) => x.id === t.id) + 1}`).join(', ')}`);
console.log(`ER scope 上限      : 30 → 最早的 FK 表が scope に入っているか: ${
  withFk.length ? (tbl.findIndex((x) => x.id === withFk[0].id) < 30 ? '✅ 入る' : '❌ 範囲外 (要修正)') : 'n/a'}`);
