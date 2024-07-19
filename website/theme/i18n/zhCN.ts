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
    '基于 Rust 和 TypeScript 的高度并行、增量编译架构，构建性能极佳，带来极致的开发体验。',
  benchmarkDetail: '参见 Benchmark 详情',
  featuredTitle: '功能完备',
  featuredDesc: '作为 webpack 的升级替代品，带来更强大的功能和卓越的生产力。',
  featureCodeSplitting:
    '将代码拆分成更小的 bundles，实现按需加载并提高页面性能。',
  featurePlugins: '提供丰富的插件钩子，并与大多数 webpack 插件兼容。',
  featureHmr: '在运行阶段通过 HMR 更新模块，无需刷新整个页面。',
  featureSwc: '利用基于 Rust 的 SWC 来加速 JavaScript 和 TypeScript 的转译。',
  featureTreeShaking:
    '检测并消除最终 bundle 中未使用的代码，以减少构建产物的大小。',
  featureLoaders: '完全兼容 webpack 的 loaders，重用整个生态系统。',
  featureDevServer: '提供成熟、高性能的 dev server，用于快速本地开发。',
  featureLightningCss: '集成 Lightning CSS 以实现快速的 CSS 处理和深度优化。',
  featureModuleFederation:
    '在多个 web 应用之间共享模块代码，更高效地团队协作。',
  featureAssetManagement: '处理和优化静态资源，如图像、字体和 stylesheets。',
  featureParallelBuilds:
    '并发运行多个构建任务，为不同的构建目标或环境输出产物。',
  featureJavaScriptApi: '提供对构建 API 的编程接口，允许自定义整个构建流程。',
  toolStackTitle: '工具栈',
  toolStackDesc: '围绕 Rspack 打造的高性能工具栈，助力现代 Web 开发',
  recruit: 'Rspack 团队正在招聘中，欢迎加入👏🏻',
};
