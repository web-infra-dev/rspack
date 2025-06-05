import type { EN_US } from './enUS';

export const ZH_CN: Record<keyof typeof EN_US, string> = {
  // hero
  heroSlogan: '基于 Rust 的高性能 Web 打包工具',
  heroSubSlogan: '使用兼容 API 无缝替换 webpack',
  getStarted: '快速开始',
  learnMore: '深入了解',

  // whyRspack
  FastStartup: '高性能',
  FastStartupDesc: '基于 Rust 的并行架构，带来极致的开发体验',
  LightningHMR: 'HMR 极快',
  LightningHMRDesc: '内置增量构建机制，HMR 速度极快，完美胜任大型项目开发',
  FrameworkAgnostic: '框架无关',
  FrameworkAgnosticDesc: '不和任何前端框架绑定，保证足够的灵活性',
  WebpackCompatible: '兼容 webpack',
  WebpackCompatibleDesc:
    '兼容 webpack 插件和 loader，无缝衔接社区中沉淀的优秀库',

  // benchmark
  benchmarkTitle: '极快的构建速度',
  benchmarkDesc:
    '基于 Rust 和 TypeScript 的高度并行、增量编译架构，构建性能极佳，带来极致的开发体验。',
  benchmarkDetail: '查看 Benchmark 详情',

  // fully featured
  fullyFeaturedTitle: '功能完备',
  fullyFeaturedDesc:
    '作为 webpack 的升级替代品，带来更强大的功能和卓越的生产力。',

  featureCodeSplitting:
    '将代码分割成更小的 bundles，实现按需加载并提高页面性能。',
  featureTreeShaking:
    '检测并消除最终 bundle 中未使用的代码，以减少构建产物的大小。',
  featurePlugins: '提供丰富的插件钩子，并与大多数 webpack 插件兼容。',
  featureModuleFederation:
    '在多个 web 应用之间共享模块代码，更高效地团队协作。',

  featureAssetManagement: '处理和优化静态资源，如图像、字体和 stylesheets。',
  featureLoaders: '完全兼容 webpack 的 loaders，重用整个生态系统。',
  featureHmr: '在运行阶段通过 HMR 更新模块，无需刷新整个页面。',
  featureDevServer: '提供成熟、高性能的 dev server，用于快速本地开发。',

  featureSwc: '利用基于 Rust 的 SWC 来加速 JavaScript 和 TypeScript 的转译。',
  featureLightningCss: '集成 Lightning CSS 以实现快速的 CSS 处理和深度优化。',
  featureParallelBuilds:
    '并发运行多个构建任务，为不同的构建目标或环境输出产物。',
  featureJavaScriptApi: '提供对构建 API 的编程接口，允许自定义整个构建流程。',

  // Who is using
  whoIsUsing: '谁在使用',

  // HomeFooter
  coldStart: '冷启动（dev）',
  coldBuild: '冷构建',
  hmr: '热更新',
  guide: '指南',
  quickStart: '快速开始',
  features: '核心特性',
  compatibility: 'webpack 兼容性',
  migration: '迁移指南',
  cli: 'CLI',
  toolchain: '工具链',
  community: '社区',
};
