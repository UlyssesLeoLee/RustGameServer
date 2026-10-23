/**
 * adapter.ts — 左手语义坐标 ↔ Three.js 右手系的 **唯一** 转换权威
 *
 * spec `3D.handedness`:
 *   "左手语义坐标 +X 右、+Y 上、+Z 前; Three.js 通过集中 Adapter 处理变换、法线、面绕序和拾取"
 *
 * ── 约定 (写在这里, 全工程只有这一处定义) ───────────────────────────────
 *   语义系 S (左手):  +X = 右, +Y = 上, +Z = 前 (= 朝场景纵深方向, 与 Unity/DirectX 左手系一致)
 *   Three 系 T (右手): +X = 右, +Y = 上, +Z = 指向观察者 (背离场景纵深)
 *
 *   映射 M: S → T 为**一次反射**  M(x, y, z) = (x, y, -z)
 *   —— 单一镜像, 保证:
 *      · 默认机位 (站在 S 的 -Z 侧, 沿 +Z 观察) 渲染出的画面**不左右翻转**,
 *        语义 +X 永远出现在屏幕右侧;
 *      · 一次镜像 = 一次手性翻转, 因此所有**面绕序**必须同步翻转 (CCW_S → CW_T),
 *        这一点由 mirrorGeometry() / buildGeometry() 集中处理, 业务代码不零散翻转。
 *
 *   反射是保距的, 因此:
 *      · 法线随 M 镜像 (normal.z 取反);
 *      · 四元数共轭: M·R(a,θ)·M⁻¹ = R(M(a), -θ)  —— 轴镜像且角取反;
 *      · 拾取: three 的射线结果统一用 fromThree() 还原回语义坐标再参与业务判断。
 *
 * 本文件不含任何业务知识 (不知道什么是 crate / floor / gem), 只做坐标与手性。
 */

import * as THREE from 'three';

/** 语义向量 (左手系) */
export interface SemanticVec3 {
  x: number;
  y: number;
  z: number;
}

export class LeftHandAdapter {
  readonly handedness = 'left' as const;

  /** 语义 → Three */
  toThree(x: number, y: number, z: number): THREE.Vector3 {
    return new THREE.Vector3(x, y, -z);
  }

  toThreeVec(v: SemanticVec3): THREE.Vector3 {
    return new THREE.Vector3(v.x, v.y, -v.z);
  }

  /** Three → 语义 (拾取命中点还原用) */
  fromThree(v: THREE.Vector3): THREE.Vector3 {
    return new THREE.Vector3(v.x, v.y, -v.z);
  }

  /** 法线镜像 (正交反射下 normal 同样按 M 变换) */
  toThreeNormal(x: number, y: number, z: number): THREE.Vector3 {
    return new THREE.Vector3(x, y, -z);
  }

  /**
   * 语义四元数 → Three 四元数。
   * 反射共轭: R_T = M·R_S·M⁻¹ = R(M(a), -θ) —— 轴取镜像, 角取反。
   */
  toThreeQuat(q: THREE.Quaternion): THREE.Quaternion {
    const w = THREE.MathUtils.clamp(q.w, -1, 1);
    const angle = 2 * Math.acos(w);
    if (angle < 1e-8) return new THREE.Quaternion();
    const s = Math.sqrt(Math.max(0, 1 - w * w));
    const axis = new THREE.Vector3(q.x / s, q.y / s, q.z / s).normalize();
    return new THREE.Quaternion().setFromAxisAngle(
      new THREE.Vector3(axis.x, axis.y, -axis.z),
      -angle,
    );
  }

  /**
   * 把一个在 **Three 空间** 里建好的几何体, 翻成它在 **语义空间** 中的对应体。
   * 做法: 顶点 z 取反 (反射) + 三角形绕序反转 (手性翻转) + 法线 z 取反。
   *
   * 为什么要翻绕序: 反射把 CCW 变成 CW; Three 默认正面是 CCW。
   * 若不翻转, 所有自建几何体会「里外翻转」——面被剔除, 看起来是空壳。
   */
  mirrorGeometry<T extends THREE.BufferGeometry>(geom: T): T {
    const pos = geom.getAttribute('position') as THREE.BufferAttribute | undefined;
    if (pos) {
      for (let i = 0; i < pos.count; i++) pos.setZ(i, -pos.getZ(i));
      pos.needsUpdate = true;
    }
    const nor = geom.getAttribute('normal') as THREE.BufferAttribute | undefined;
    if (nor) {
      for (let i = 0; i < nor.count; i++) nor.setZ(i, -nor.getZ(i));
      nor.needsUpdate = true;
    }
    reverseWinding(geom);
    geom.computeBoundingSphere();
    geom.computeBoundingBox();
    return geom;
  }

  /**
   * 直接用 **语义坐标** 构建几何体 (positions / normals 均为语义系, CCW 为语义正面)。
   * 内部做: z 取反 + 绕序反转 + 法线取反 → 得到可直接交给 Three 渲染的几何体。
   */
  buildGeometry(positions: number[], normals: number[], indices: number[]): THREE.BufferGeometry {
    const g = new THREE.BufferGeometry();
    const p = new Float32Array(positions.length);
    const n = new Float32Array(normals.length);
    for (let i = 0; i < positions.length; i += 3) {
      p[i] = positions[i];
      p[i + 1] = positions[i + 1];
      p[i + 2] = -positions[i + 2];
    }
    for (let i = 0; i < normals.length; i += 3) {
      n[i] = normals[i];
      n[i + 1] = normals[i + 1];
      n[i + 2] = -normals[i + 2];
    }
    g.setAttribute('position', new THREE.BufferAttribute(p, 3));
    g.setAttribute('normal', new THREE.BufferAttribute(n, 3));
    const idx = new Uint32Array(indices.length);
    for (let t = 0; t < indices.length; t += 3) {
      idx[t] = indices[t];
      idx[t + 1] = indices[t + 2];
      idx[t + 2] = indices[t + 1];
    }
    g.setIndex(new THREE.BufferAttribute(idx, 1));
    g.computeBoundingSphere();
    g.computeBoundingBox();
    return g;
  }
}

/** 三角形绕序反转 (索引 / 非索引几何体都支持) */
export function reverseWinding(geom: THREE.BufferGeometry): void {
  const idx = geom.getIndex();
  if (idx) {
    const arr = idx.array as unknown as ArrayLike<number> & { [i: number]: number };
    for (let i = 0; i + 2 < idx.count; i += 3) {
      const tmp = arr[i + 1];
      arr[i + 1] = arr[i + 2];
      arr[i + 2] = tmp;
    }
    idx.needsUpdate = true;
    return;
  }
  // 非索引: 逐三角形交换第 2/3 个顶点 (所有属性一起换)
  for (const name of Object.keys(geom.attributes)) {
    const attr = geom.getAttribute(name) as THREE.BufferAttribute;
    const arr = attr.array as unknown as ArrayLike<number> & { [i: number]: number };
    const size = attr.itemSize;
    for (let t = 0; t + 2 < attr.count; t += 3) {
      for (let k = 0; k < size; k++) {
        const i1 = (t + 1) * size + k;
        const i2 = (t + 2) * size + k;
        const tmp = arr[i1];
        arr[i1] = arr[i2];
        arr[i2] = tmp;
      }
    }
    attr.needsUpdate = true;
  }
}
