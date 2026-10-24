/**
 * d0.ts — 0D 存在性门禁 (内部门禁, **不制作独立页面**)
 *
 * spec `placement.0D`: "内部存在性门禁，无导航页面"
 * spec `immutable_dimensions.0D.prohibited`: "将 Unknown 自动视为存在" / "给 0D 制作装饰架构图"
 *
 * 实现: 顶栏门禁指示器 → 模态窗展示存在性矩阵。Unknown 必须显式标出, 不得折叠为 Present。
 */

import type { DimensionModule, DimensionContext } from '../core/app';
import { esc } from '../core/app';
import type { ExistenceStatus } from '../core/types';

interface GateRow {
  key: string;
  label: string;
  status: ExistenceStatus;
  detail: string;
  evidenceCount: number;
}

export function buildGate(ctx: DimensionContext): GateRow[] {
  const rows: GateRow[] = [];
  const { index } = ctx;

  /* 1. crate 源码存在性 */
  const services = [...index.nodes.values()].filter((n) => n.kind === 'Service');
  const withSrc = services.filter((s) => Number(s.properties.rustFiles ?? 0) > 0);
  rows.push({
    key: 'crate-source', label: 'Rust crate 源码',
    status: withSrc.length === services.length ? 'Present' : withSrc.length ? 'Present' : 'Unknown',
    detail: `${withSrc.length}/${services.length} 个 crate 有 .rs 源码`,
    evidenceCount: services.length,
  });

  /* 2. 部署存在性 —— 无 manifest = Unknown, 不等于 Absent */
  const deployed = services.filter((s) => s.properties.deployed);
  rows.push({
    key: 'deployment', label: 'k3s Deployment 清单',
    status: deployed.length ? 'Present' : 'Unknown',
    detail: `${deployed.length}/${services.length} 个 crate 有 Deployment 证据; 其余为 Unknown (非 Absent — 可能由其它清单或集群管理)`,
    evidenceCount: deployed.length,
  });

  /* 3. gRPC 契约 */
  const apis = [...index.nodes.values()].filter((n) => n.kind === 'Api');
  rows.push({
    key: 'grpc-contract', label: 'gRPC 服务契约 (proto)',
    status: apis.length ? 'Present' : 'Absent',
    detail: `${apis.length} 个 service, ${apis.reduce((a, b) => a + Number(b.properties.rpcCount ?? 0), 0)} 个 method`,
    evidenceCount: apis.length,
  });

  /* 4. 持久化 schema */
  const tables = [...index.nodes.values()].filter((n) => n.kind === 'Table');
  rows.push({
    key: 'db-schema', label: '数据库 Schema (migrations)',
    status: tables.length ? 'Present' : 'Absent',
    detail: `${tables.length} 张表, PK=${tables.filter((t) => (t.properties.pkColumns as string[] | undefined)?.length).length}, FK=${tables.filter((t) => (t.properties.fkColumns as unknown[] | undefined)?.length).length}`,
    evidenceCount: tables.length,
  });

  /* 5. 真实跨服务调用 (含仅设计意图的区分) */
  const impl = index.edges.size;
  const designed = index.designedEdges.size;
  rows.push({
    key: 'cross-call', label: '跨服务调用实现',
    status: impl ? 'Present' : 'Unknown',
    detail: `${impl} 条真实调用点 (Implemented); ${designed} 条仅注释提及 (Inferred, 不计入实现)`,
    evidenceCount: impl + designed,
  });

  /* 6. 源需求/设计文档 —— 仓库有 RACI/OPEN-QA, 但无 per-service 需求规格 → Unknown */
  rows.push({
    key: 'source-req-doc', label: '源需求 / 详细设计文档',
    status: 'Unknown',
    detail: '仓库有 docs/14-项目治理/ (RACI, OPEN-QA, DDD Review), 但无 per-service 业务需求规格 → GAP_SOURCE_DOCUMENT, 不计入图集失败',
    evidenceCount: 0,
  });

  /* 7. 图集自身必需产物 */
  rows.push({
    key: 'atlas-artifacts', label: '本图集构建产物',
    status: 'Present',
    detail: 'index.html / dist/index.html / models/ / src/ / tests/ — 存在性由构建期自检门禁保证',
    evidenceCount: 0,
  });

  return rows;
}

const STATUS_CLASS: Record<ExistenceStatus, string> = {
  Present: 'badge--Present', Absent: 'badge--Absent', Unknown: 'badge--Unknown',
};

export const d0: DimensionModule = {
  id: '0D',
  title: '0D 存在性门禁',
  mount(ctx) {
    const rows = buildGate(ctx);
    const unknown = rows.filter((r) => r.status === 'Unknown').length;

    const open = () => {
      const root = document.getElementById('modal-root')!;
      root.hidden = false;
      root.innerHTML = `
        <div class="modal" role="dialog" aria-modal="true" aria-label="0D 存在性门禁">
          <div class="modal-head">
            <strong>0D 存在性门禁</strong>
            <button class="icon-btn" data-close aria-label="关闭">✕</button>
          </div>
          <div class="modal-body">
            <p class="empty-hint">门禁只回答"有 / 无 / 未知", 不做架构图。Unknown 一律显式保留, 不折叠为 Present。</p>
            <table class="data">
              <thead><tr><th>对象</th><th>状态</th><th>依据</th><th>证据数</th></tr></thead>
              <tbody>
                ${rows.map((r) => `<tr>
                  <td>${esc(r.label)}</td>
                  <td><span class="badge ${STATUS_CLASS[r.status]}">${r.status}</span></td>
                  <td>${esc(r.detail)}</td>
                  <td>${r.evidenceCount}</td>
                </tr>`).join('')}
              </tbody>
            </table>
            <p class="empty-hint" style="margin-top:10px">未知项 ${unknown} 个 — 未知不等于不存在。</p>
          </div>
        </div>`;
      const close = () => { root.hidden = true; root.innerHTML = ''; document.removeEventListener('keydown', onKey); };
      const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') close(); };
      document.addEventListener('keydown', onKey);
      root.querySelector('[data-close]')?.addEventListener('click', close);
      root.addEventListener('click', (e) => { if (e.target === root) close(); });
      (root.querySelector('[data-close]') as HTMLElement)?.focus();
    };

    // 顶栏指示器
    const btn = document.createElement('button');
    btn.className = 'icon-btn';
    btn.id = 'gate-indicator';
    btn.title = '0D 存在性门禁';
    btn.setAttribute('aria-label', '0D 存在性门禁');
    btn.innerHTML = `0D<span style="margin-left:4px;font-size:10px;color:${unknown ? 'var(--warn)' : 'var(--success)'}">${unknown ? `${unknown}?` : '✓'}</span>`;
    btn.addEventListener('click', open);
    document.querySelector('.header-actions')?.prepend(btn);
  },
};
