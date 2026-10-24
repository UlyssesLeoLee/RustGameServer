/**
 * edges.ts — 工程关系的 3D 表现 (spec `3D.picking` / `exclusive_semantics.edge_style`)
 *
 * 线型 = 工程关系 (排他语义: 形状/颜色/切面/环/大小 各占各的通道, 线型归关系):
 *   · Implemented (architecture.json edges)      → 实心管
 *   · Inferred    (architecture.json designedOnlyEdges) → 虚线管 (分段留缝)
 *   · 跨层连线 (source 楼层 ≠ target 楼层)         → 走"竖井"路径, 独立可拾取
 *   · 多边 (同 source→target 多条)                → 弧线侧向偏移, 不重叠
 *   · 自环 (source === target)                    → 显式回环
 *   · 方向                                       → 锥形箭头
 *
 * 坐标: 路径控制点先在 **语义系** 计算, 再由 LeftHandAdapter 统一转到 Three 系。
 *       管体几何由 TubeGeometry 在 Three 系内生成, 其绕序天然正确, 不需要二次镜像。
 */

import * as THREE from 'three';
import { LeftHandAdapter } from './adapter';
import type { EdgeInfo, NodeInfo, Vec3 } from './model';

export interface EdgeCurve {
  /** Three 系曲线 */
  curve: THREE.Curve<THREE.Vector3>;
  /** 中点 (Three 系) */
  mid: THREE.Vector3;
  /** 终点切向 (Three 系, 归一化) */
  tangent: THREE.Vector3;
  /** 语义系中点 (标签/距离计算用) */
  midSemantic: Vec3;
  /** 是否跨层 */
  crossFloor: boolean;
}

const UP = new THREE.Vector3(0, 1, 0);

/** 语义点 → Three 点 */
function s2t(a: LeftHandAdapter, v: Vec3): THREE.Vector3 {
  return a.toThree(v.x, v.y, v.z);
}

/**
 * 构造一条边的路径 (语义控制点)。
 *  · 跨层: 走 5 点竖井折线 (源点抬升 → 中段 → 目标点抬升)
 *  · 同层: 三点弧线, 多边按 multiIndex 侧向偏移
 *  · 自环: 四点回环
 */
export function buildEdgeCurve(
  a: LeftHandAdapter,
  info: EdgeInfo,
  src: NodeInfo,
  dst: NodeInfo,
): EdgeCurve {
  const s = src.position;
  const d = dst.position;
  const ctrl: THREE.Vector3[] = [];

  if (info.edge.sourceId === info.edge.targetId) {
    // 自环: 向 +X 侧外扩再折回
    const outX = 6 + info.multiIndex * 2.5;
    ctrl.push(
      s2t(a, { x: s.x, y: s.y, z: s.z }),
      s2t(a, { x: s.x + outX, y: s.y + 4, z: s.z }),
      s2t(a, { x: s.x + outX, y: s.y - 4, z: s.z }),
      s2t(a, { x: s.x, y: s.y, z: s.z }),
    );
  } else if (info.crossFloor) {
    // 跨层: 竖井 (先抬升, 中段横移, 再落到目标)
    const midX = (s.x + d.x) / 2;
    const midZ = (s.z + d.z) / 2;
    const off = info.multiIndex * 3.2;
    ctrl.push(
      s2t(a, { x: s.x, y: s.y, z: s.z }),
      s2t(a, { x: s.x + off, y: s.y + 7, z: s.z }),
      s2t(a, { x: midX, y: (s.y + d.y) / 2 + 10, z: midZ + off }),
      s2t(a, { x: d.x, y: d.y + 7, z: d.z }),
      s2t(a, { x: d.x, y: d.y, z: d.z }),
    );
  } else {
    // 同层弧线
    const dx = d.x - s.x;
    const dz = d.z - s.z;
    const len = Math.hypot(dx, dz) || 1;
    // 侧向单位向量 (在 XZ 平面内), 供多边错开
    const px = -dz / len;
    const pz = dx / len;
    const lateral = (info.multiIndex - (info.multiCount - 1) / 2) * 7.5;
    const bulge = 6 + Math.abs(lateral) * 0.4;
    ctrl.push(
      s2t(a, { x: s.x, y: s.y, z: s.z }),
      s2t(a, {
        x: (s.x + d.x) / 2 + px * lateral,
        y: (s.y + d.y) / 2 + bulge,
        z: (s.z + d.z) / 2 + pz * lateral,
      }),
      s2t(a, { x: d.x, y: d.y, z: d.z }),
    );
  }

  const curve = new THREE.CatmullRomCurve3(ctrl, false, 'catmullrom', 0.4);
  const mid = curve.getPointAt(0.5);
  const tangent = curve.getTangentAt(0.72).normalize();
  const midT = curve.getPointAt(0.5);
  const midSemantic = a.fromThree(midT);
  return { curve, mid, tangent, midSemantic, crossFloor: info.crossFloor };
}

/** 实心管几何 */
export function solidTube(curve: THREE.Curve<THREE.Vector3>, radius: number, segments = 48): THREE.BufferGeometry {
  return new THREE.TubeGeometry(curve, segments, radius, 8, false);
}

/**
 * 虚线管几何: 沿曲线切成 count 段, 每段留 duty 比例的实体。
 * 用分段管而不是 LineDashedMaterial —— 保持 3D 管体的立体感与可拾取性。
 */
export function dashedTube(
  curve: THREE.Curve<THREE.Vector3>,
  radius: number,
  count = 9,
  duty = 0.55,
): THREE.BufferGeometry[] {
  const out: THREE.BufferGeometry[] = [];
  for (let i = 0; i < count; i++) {
    const t0 = i / count;
    const t1 = t0 + duty / count;
    const pts: THREE.Vector3[] = [];
    const steps = 6;
    for (let k = 0; k <= steps; k++) pts.push(curve.getPointAt(t0 + ((t1 - t0) * k) / steps));
    const sub = new THREE.CatmullRomCurve3(pts, false, 'catmullrom', 0.2);
    out.push(new THREE.TubeGeometry(sub, 8, radius, 6, false));
  }
  return out;
}

/** 箭头朝向: 锥体尖端在局部 -Z (见 gems.arrowGeometry), lookAt 后即朝行进方向 */
export function orientArrow(mesh: THREE.Object3D, tangent: THREE.Vector3, at: THREE.Vector3): void {
  mesh.position.copy(at);
  const dir = tangent.clone().normalize();
  mesh.lookAt(at.clone().add(dir));
}

export { UP };
