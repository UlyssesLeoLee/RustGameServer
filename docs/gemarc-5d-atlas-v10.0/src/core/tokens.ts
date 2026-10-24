/**
 * tokens.ts — 设计令牌 (spec `html_design_system`)
 *
 * 直接对应 spec 给出的 base_tokens / layout_contract / typography / component / motion,
 * 数值一字不改地落成可消费的 token 对象 + CSS 变量。
 * 对比度在 QA_FINAL.md 中有 WCAG 实算记录, 不做"看起来够"的声明。
 */

export const TOKENS = {
  base: {
    background: '#0B1220',
    surface: '#121D2E',
    raised: '#1C2A3D',
    primaryText: '#E8EFF7',
    secondaryText: '#AAB9CB',
    focus: '#5AC7C3',
    information: '#82ADFA',
    success: '#74CA9D',
    warning: '#F0BA63',
    failure: '#F08886',
    mutedEdge: '#40536A',
  },
  /** spec `spatial_visual_standard.domain_palette` — 业务域色, 不复用为错误色 */
  domain: {
    Core: '#4C83FF',
    Interaction: '#19AFA3',
    Data: '#32B67A',
    Security: '#E8B34F',
    Intelligence: '#A276E8',
    Infrastructure: '#8397B7',
    External: '#38A6D6',
    Operations: '#D87792',
    Unknown: '#81909F',
  },
  spacing: { base: 4, scale: [4, 8, 12, 16, 24, 32] },
  sizing: {
    headerHeight: 60,
    leftSidebar: 256,
    rightInspector: 360,
    minCanvasWidth: 480,
    timelineHeight: 136,
    controlHeight: 36,
  },
  typography: {
    caption: { size: 12, line: 18 },
    small: { size: 13, line: 20 },
    body: { size: 14, line: 22 },
    subtitle: { size: 16, line: 24 },
    heading: { size: 20, line: 28 },
    title: { size: 28, line: 36 },
  },
  radius: { sm: 6, md: 10, lg: 14 },
  stroke: 1,
  focusRing: 2,
  motion: {
    micro: 140,
    standard: 220,
    spatial: 320,
    easing: 'ease-out',
    reducedMotion: '0ms or minimal fade',
  },
} as const;

/** spec `spatial_visual_standard.ring_states` */
export const RING_STATES = [
  'Idle', 'Running', 'Waiting', 'Completed', 'Failed', 'Recovered', 'Unknown',
] as const;

/** spec `spatial_visual_standard.gemstone_roles` */
export const GEMSTONE_ROLES = [
  'Actor', 'Input', 'Processor', 'Decision', 'Storage', 'Message',
  'Interface', 'Security', 'Agent', 'Composition', 'Output',
] as const;

export const RING_STATE_COLOR: Record<string, string> = {
  Idle: '#5A6C85',
  Running: '#5AC7C3',
  Waiting: '#F0BA63',
  Completed: '#74CA9D',
  Failed: '#F08886',
  Recovered: '#82ADFA',
  Unknown: '#81909F',
};

export const DOMAIN_COLOR = TOKENS.domain;

/** 生成 :root CSS 变量 */
export function cssVariables(): string {
  const b = TOKENS.base;
  const lines: string[] = [];
  lines.push(`--bg:${b.background};--surface:${b.surface};--raised:${b.raised};`);
  lines.push(`--text:${b.primaryText};--text-2:${b.secondaryText};--focus:${b.focus};`);
  lines.push(`--info:${b.information};--success:${b.success};--warn:${b.warning};--fail:${b.failure};--edge:${b.mutedEdge};`);
  for (const [k, v] of Object.entries(TOKENS.domain)) lines.push(`--dom-${k}:${v};`);
  lines.push(`--r-sm:${TOKENS.radius.sm}px;--r-md:${TOKENS.radius.md}px;--r-lg:${TOKENS.radius.lg}px;`);
  lines.push(`--h-header:${TOKENS.sizing.headerHeight}px;--w-left:${TOKENS.sizing.leftSidebar}px;--w-right:${TOKENS.sizing.rightInspector}px;`);
  lines.push(`--h-timeline:${TOKENS.sizing.timelineHeight}px;--h-control:${TOKENS.sizing.controlHeight}px;--min-canvas:${TOKENS.sizing.minCanvasWidth}px;`);
  lines.push(`--t-micro:${TOKENS.motion.micro}ms;--t-std:${TOKENS.motion.standard}ms;--t-spatial:${TOKENS.motion.spatial}ms;`);
  return lines.join('\n');
}

/** 响应式断点 (spec `responsive_targets`) */
export const VIEWPORTS = [
  { name: 'mobile', w: 390, h: 844 },
  { name: 'tablet', w: 768, h: 1024 },
  { name: 'laptop', w: 1024, h: 700 },
  { name: 'desktop', w: 1440, h: 900 },
  { name: 'wide', w: 1920, h: 1080 },
] as const;
