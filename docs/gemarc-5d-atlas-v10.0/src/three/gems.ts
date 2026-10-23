/**
 * gems.ts — 宝石几何工厂 (spec `spatial_visual_standard.gemstone_roles`)
 *
 * spec: 形状 = 功能角色, 颜色 = 业务域, 切面 = 角色子类, 环 = 运行状态,
 *       大小 = 有证据的数量指标, 线型 = 工程关系, 动画 = 事件/时间, 因果叠加 = A/B 分歧或合流。
 * 本文件只负责 **形状** (mesh 轮廓/切面/法线/深度), 颜色与尺寸由调用方按排他语义注入。
 *
 * 11 个角色形状 (spec 逐字对应):
 *   Actor 放射圆形 / Input 水滴切割 / Processor 六棱柱 / Decision 双锥八面体 /
 *   Storage 阶梯矩形 / Message 橄榄梭形 / Interface 三角晶体 / Security 盾形 /
 *   Agent 五角晶体 / Composition 皇冠晶体 / Output 梯形切割
 *
 * 几何要求 (spec `3d_geometry_requirements`):
 *   真实 mesh 有轮廓/切面/法线/深度; 正/顶/侧/45° 可辨; 简化到 24/32/48px 轮廓仍明确;
 *   禁用 Bloom 与 Transmission 后仍可辨识 → 本文件不使用任何后处理与透射材质。
 *
 * 所有几何体都经 LeftHandAdapter 落地 (见 adapter.ts): 一次镜像 + 绕序翻转。
 */

import * as THREE from 'three';
import { LeftHandAdapter } from './adapter';

export type GemRole =
  | 'Actor' | 'Input' | 'Processor' | 'Decision' | 'Storage' | 'Message'
  | 'Interface' | 'Security' | 'Agent' | 'Composition' | 'Output';

export const GEM_ROLE_LIST: GemRole[] = [
  'Actor', 'Input', 'Processor', 'Decision', 'Storage', 'Message',
  'Interface', 'Security', 'Agent', 'Composition', 'Output',
];

/** 角色的中文说明 (侧栏图例用) */
export const GEM_ROLE_DESC: Record<GemRole, string> = {
  Actor: '放射圆形',
  Input: '水滴切割',
  Processor: '六棱柱',
  Decision: '双锥八面体',
  Storage: '阶梯矩形',
  Message: '橄榄梭形',
  Interface: '三角晶体',
  Security: '盾形',
  Agent: '五角晶体',
  Composition: '皇冠晶体',
  Output: '梯形切割',
};

/* ── 自建体: 阶梯矩形 (Storage) ──────────────────────────────────────── */

/** 3 层台阶的箱体 (语义坐标, 底面在 y=-0.5) */
function steppedBox(sizes: [number, number, number][]): { positions: number[]; normals: number[]; indices: number[] } {
  const positions: number[] = [];
  const normals: number[] = [];
  const indices: number[] = [];
  let y = -0.5;
  for (const [w, h, d] of sizes) {
    const y0 = y;
    const y1 = y + h;
    const x = w / 2;
    const z = d / 2;
    // 8 角
    const corners: [number, number, number][] = [
      [-x, y0, -z], [x, y0, -z], [x, y1, -z], [-x, y1, -z],
      [-x, y0, z], [x, y0, z], [x, y1, z], [-x, y1, z],
    ];
    // 6 面 (每面独立顶点, 保证硬边法线)
    const faces: { idx: number[]; n: [number, number, number] }[] = [
      { idx: [0, 3, 2, 1], n: [0, 0, -1] },   // -Z
      { idx: [4, 5, 6, 7], n: [0, 0, 1] },    // +Z
      { idx: [0, 4, 7, 3], n: [-1, 0, 0] },   // -X
      { idx: [1, 2, 6, 5], n: [1, 0, 0] },    // +X
      { idx: [3, 7, 6, 2], n: [0, 1, 0] },    // +Y
      { idx: [0, 1, 5, 4], n: [0, -1, 0] },   // -Y
    ];
    for (const f of faces) {
      const base = positions.length / 3;
      const order = [f.idx[0], f.idx[1], f.idx[2], f.idx[0], f.idx[2], f.idx[3]];
      for (const ci of order) {
        positions.push(corners[ci][0], corners[ci][1], corners[ci][2]);
        normals.push(f.n[0], f.n[1], f.n[2]);
      }
      indices.push(base, base + 1, base + 2, base, base + 2, base + 3);
    }
    y = y1;
  }
  return { positions, normals, indices };
}

/* ── 自建体: 盾形 (Security) ──────────────────────────────────────────── */

function shieldPlate(): THREE.ExtrudeGeometry {
  const s = new THREE.Shape();
  // 盾形轮廓 (XY 平面, 顶边平, 底边收成尖)
  s.moveTo(-0.85, 0.9);
  s.lineTo(0.85, 0.9);
  s.lineTo(0.85, -0.1);
  s.quadraticCurveTo(0.7, -0.85, 0, -1.0);
  s.quadraticCurveTo(-0.7, -0.85, -0.85, -0.1);
  s.closePath();
  const g = new THREE.ExtrudeGeometry(s, {
    depth: 0.28, bevelEnabled: true, bevelSize: 0.06, bevelThickness: 0.05, bevelSegments: 1, curveSegments: 8,
  });
  g.center();
  return g;
}

/* ── 自建体: 水滴 (Input) ─────────────────────────────────────────────── */

function dropProfile(): THREE.LatheGeometry {
  const pts: THREE.Vector2[] = [
    new THREE.Vector2(0.0, 1.15),
    new THREE.Vector2(0.22, 0.92),
    new THREE.Vector2(0.44, 0.55),
    new THREE.Vector2(0.58, 0.1),
    new THREE.Vector2(0.55, -0.35),
    new THREE.Vector2(0.38, -0.72),
    new THREE.Vector2(0.16, -0.95),
    new THREE.Vector2(0.0, -1.0),
  ];
  const g = new THREE.LatheGeometry(pts, 18);
  return g;
}

/* ── 自建体: 皇冠 (Composition) ───────────────────────────────────────── */

function crownProfile(): THREE.LatheGeometry {
  // 锯齿轮廓: 5 个尖冠
  const pts: THREE.Vector2[] = [];
  const teeth = 5;
  for (let i = 0; i <= teeth; i++) {
    const t = i / teeth;
    const a = t * Math.PI * 2;
    const r = 0.85 + 0.15 * Math.cos(a * teeth);
    pts.push(new THREE.Vector2(r, i === 0 ? -0.85 : -0.35 + 0.1 * Math.cos(a * teeth)));
  }
  pts.unshift(new THREE.Vector2(0, -0.9));
  pts.push(new THREE.Vector2(0, 0.9));
  const g = new THREE.LatheGeometry(pts, 5);
  g.computeVertexNormals();
  return g;
}

/* ── 几何工厂 ─────────────────────────────────────────────────────────── */

const cache = new Map<string, THREE.BufferGeometry>();

/**
 * 取得某角色的单位几何体 (半径 ~1, 高度 ~2)。
 * 结果已缓存; 调用方需要独立副本时自行 clone()。
 */
export function gemGeometry(role: GemRole, adapter: LeftHandAdapter): THREE.BufferGeometry {
  const key = role;
  const hit = cache.get(key);
  if (hit) return hit;

  let g: THREE.BufferGeometry;
  switch (role) {
    case 'Actor': {
      // 放射圆形: 二十面体细分 + 硬边着色 → 辐射状切面
      const s = new THREE.IcosahedronGeometry(1, 1);
      g = s;
      break;
    }
    case 'Input':
      g = dropProfile();
      break;
    case 'Processor': {
      // 六棱柱
      g = new THREE.CylinderGeometry(0.92, 0.92, 1.7, 6, 1);
      break;
    }
    case 'Decision': {
      // 双锥八面体
      g = new THREE.OctahedronGeometry(1, 0);
      g.scale(1, 1.25, 1);
      break;
    }
    case 'Storage': {
      // 阶梯矩形
      const box = steppedBox([
        [1.7, 0.42, 1.25],
        [1.4, 0.36, 1.05],
        [1.05, 0.32, 0.8],
      ]);
      g = adapter.buildGeometry(box.positions, box.normals, box.indices);
      break;
    }
    case 'Message': {
      // 橄榄梭形
      g = new THREE.SphereGeometry(1, 18, 12);
      g.scale(0.62, 1.45, 0.62);
      break;
    }
    case 'Interface': {
      // 三角晶体
      g = new THREE.ConeGeometry(1, 1.9, 3, 1);
      break;
    }
    case 'Security':
      g = shieldPlate();
      break;
    case 'Agent': {
      // 五角晶体
      g = new THREE.CylinderGeometry(0.9, 0.9, 1.5, 5, 1);
      break;
    }
    case 'Composition':
      g = crownProfile();
      break;
    case 'Output': {
      // 梯形切割
      g = new THREE.CylinderGeometry(0.5, 1, 1.6, 4, 1);
      break;
    }
  }

  // 统一: 一次镜像 (位置 + 法线 + 绕序) —— 手性转换的唯一出口
  adapter.mirrorGeometry(g);
  // 硬边着色由材质 flatShading 提供, 这里去掉 smooth 法线残留
  g.computeBoundingSphere();
  g.computeBoundingBox();
  cache.set(key, g);
  return g;
}

/** 运行状态环 (水平放置的扁环, 颜色 = RING_STATE_COLOR) */
export function ringGeometry(adapter: LeftHandAdapter): THREE.BufferGeometry {
  const key = '__ring__';
  const hit = cache.get(key);
  if (hit) return hit;
  const g = new THREE.TorusGeometry(1.45, 0.055, 6, 28);
  g.rotateX(-Math.PI / 2);   // 摊平在 XZ 平面
  adapter.mirrorGeometry(g);
  cache.set(key, g);
  return g;
}

/** 房间地板 (薄盒, 既是视觉底板也是房间拾取靶) */
export function slabGeometry(adapter: LeftHandAdapter): THREE.BufferGeometry {
  const key = '__slab__';
  const hit = cache.get(key);
  if (hit) return hit;
  const g = new THREE.BoxGeometry(1, 1, 1);
  adapter.mirrorGeometry(g);
  cache.set(key, g);
  return g;
}

/** 房间墙 (薄盒, 三面墙 + 可切面的前墙) */
export function wallGeometry(adapter: LeftHandAdapter): THREE.BufferGeometry {
  const key = '__wall__';
  const hit = cache.get(key);
  if (hit) return hit;
  const g = new THREE.BoxGeometry(1, 1, 1);
  adapter.mirrorGeometry(g);
  cache.set(key, g);
  return g;
}

/** 端口凸台 (小八面体) */
export function portGeometry(adapter: LeftHandAdapter): THREE.BufferGeometry {
  const key = '__port__';
  const hit = cache.get(key);
  if (hit) return hit;
  const g = new THREE.OctahedronGeometry(1, 0);
  g.scale(1, 0.7, 1);
  adapter.mirrorGeometry(g);
  cache.set(key, g);
  return g;
}

/** 箭头 (边方向指示) */
export function arrowGeometry(adapter: LeftHandAdapter): THREE.BufferGeometry {
  const key = '__arrow__';
  const hit = cache.get(key);
  if (hit) return hit;
  const g = new THREE.ConeGeometry(0.42, 1.1, 8, 1);
  // Object3D.lookAt 让物体局部 -Z 指向目标 → 锥尖转到 -Z, 使 lookAt 后锥尖朝行进方向
  g.rotateX(-Math.PI / 2);
  adapter.mirrorGeometry(g);
  cache.set(key, g);
  return g;
}

/** 释放缓存 (维度卸载时调用) */
export function disposeGemCache(): void {
  for (const g of cache.values()) g.dispose();
  cache.clear();
}
