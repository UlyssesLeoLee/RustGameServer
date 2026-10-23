/** 探针: 在 node 里逐步复现 3D 场景构建, 定位第一个抛错的步骤 */
import { buildAtlasModel } from '../src/three/model';
import { LeftHandAdapter } from '../src/three/adapter';
import { gemGeometry, ringGeometry, slabGeometry, wallGeometry, portGeometry, arrowGeometry } from '../src/three/gems';
import { buildEdgeCurve, solidTube, dashedTube, orientArrow } from '../src/three/edges';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

const M = join(process.cwd(), 'models');
const arch = JSON.parse(readFileSync(join(M, 'architecture.json'), 'utf8'));
const spatial = JSON.parse(readFileSync(join(M, 'spatial.json'), 'utf8'));
const rules = JSON.parse(readFileSync(join(M, 'causal-rules.json'), 'utf8')).rules;
const cases = JSON.parse(readFileSync(join(M, 'cases.json'), 'utf8')).cases;

// graph core 需要 GraphIndex
const { buildIndex } = await import('../src/core/graph');
const idx = buildIndex(arch);

const step = (name: string, fn: () => void) => {
  try { fn(); console.log(`  OK   ${name}`); }
  catch (e) { console.log(`  FAIL ${name}: ${(e as Error).message}`); console.log((e as Error).stack?.split('\n').slice(0, 4).join('\n')); throw e; }
};

console.log('=== 1. buildAtlasModel ===');
let atlas: ReturnType<typeof buildAtlasModel>;
step('buildAtlasModel', () => { atlas = buildAtlasModel(arch, idx, spatial, rules, cases); });
console.log(`  nodes=${atlas!.nodes.size} edges=${atlas!.edges.length} rooms=${atlas!.rooms.length} ports=${atlas!.portIndex.size}`);

const adapter = new LeftHandAdapter();
console.log('=== 2. 幾何 ===');
step('slabGeometry', () => { slabGeometry(adapter); });
step('wallGeometry', () => { wallGeometry(adapter); });
step('gemGeometry', () => { gemGeometry('Processor', adapter); });
step('ringGeometry', () => { ringGeometry(adapter); });
step('portGeometry', () => { portGeometry(adapter); });
step('arrowGeometry', () => { arrowGeometry(adapter); });

console.log('=== 3. ノード位置 ===');
for (const n of atlas!.nodes.values()) {
  if (!n.position) { console.log(`  FAIL ノード ${n.id} (${n.name}) に position が無い`); break; }
  step(`toThree(${n.id})`, () => { adapter.toThree(n.position.x, n.position.y, n.position.z); });
  break;
}
step('adapter.toThree(n1)', () => {
  const n = [...atlas!.nodes.values()][0];
  adapter.toThree(n.position.x, n.position.y, n.position.z);
});

console.log('=== 4. ルーム中心 ===');
step('room.center', () => {
  const r = atlas!.rooms[0];
  if (!r.center) throw new Error(`room ${r.id} に center が無い`);
  adapter.toThree(r.center.x, r.center.y, r.center.z);
});

console.log('=== 5. ポート位置 ===');
step('port.position', () => {
  const p = [...atlas!.portIndex.values()][0];
  if (!p.position) throw new Error(`port ${p.id} に position が無い`);
  adapter.toThree(p.position.x, p.position.y, p.position.z);
});

console.log('=== 6. エッジ曲線 ===');
step('buildEdgeCurve', () => {
  const e = atlas!.edges[0];
  const src = atlas!.nodes.get(e.sourceId);
  const tgt = atlas!.nodes.get(e.targetId);
  console.log(`  edge=${e.id} src=${e.sourceId} tgt=${e.targetId} srcFound=${!!src} tgtFound=${!!tgt}`);
  if (!src || !tgt) throw new Error('端点ノードが見つからない');
  buildEdgeCurve(adapter, e, src, tgt);
});

console.log('=== 7. チューブ ===');
step('solidTube', () => {
  const e = atlas!.edges[0];
  const src = atlas!.nodes.get(e.sourceId)!;
  const tgt = atlas!.nodes.get(e.targetId)!;
  const ec = buildEdgeCurve(adapter, e, src, tgt);
  solidTube(ec.curve, 0.18, 48);
});
step('dashedTube', () => {
  const e = atlas!.edges.find((x) => x.designedOnly) ?? atlas!.edges[0];
  const src = atlas!.nodes.get(e.sourceId)!;
  const tgt = atlas!.nodes.get(e.targetId)!;
  const ec = buildEdgeCurve(adapter, e, src, tgt);
  const arr = dashedTube(ec.curve, 0.22, 12, 0.55);
  console.log(`  dashedTube → ${Array.isArray(arr) ? arr.length + ' 個' : typeof arr}`);
});
