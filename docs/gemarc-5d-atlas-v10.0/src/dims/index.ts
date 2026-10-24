/**
 * dims/index.ts — 维度模块注册表
 *
 * 0D 是内部门禁 (d0.ts 自挂顶栏指示器, 无导航页), 其余为可切换页面。
 * Prototype 是独立 Tab, 不是第六维 (spec `placement.prototype`)。
 */

import type { DimensionModule } from '../core/app';
import { d0 } from './d0';
import { d1 } from './d1';
import { d2 } from './d2';
import { d3 } from './d3';
import { d4 } from './d4';
import { d5 } from './d5';
import { prototype } from './prototype';

export const DIMENSIONS: DimensionModule[] = [d0, d1, d2, d3, d4, d5, prototype];
