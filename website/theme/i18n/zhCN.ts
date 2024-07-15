import type { EN_US } from './enUS';

export const ZH_CN: Record<keyof typeof EN_US, string> = {
  coldStart: '冷启动（dev）',
  coldBuild: '冷构建',
  hmr: '热更新',
  guide: '指南',
  quickStart: '快速开始',
  features: '核心特性',
  compatibility: 'webpack 兼容性',
  migration: '迁移指南',
  cli: 'CLI',
  ecosystem: '生态',
  community: '社区',
  benchmarkTitle: '极快的构建速度',
  benchmarkDesc:
    '基于 Rust 和 TypeScript 的高度并行、增量编译架构，构建速度非常快，带给你极致的开发体验。',
  benchmarkDetail: '参见 Benchmark 详情',
  recruit: 'Rspack 团队正在招聘中，欢迎加入👏🏻',
};
