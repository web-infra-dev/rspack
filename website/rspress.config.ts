import path from 'node:path';
import { defineConfig } from 'rspress/config';
import type { NavItem, Sidebar } from '@rspress/shared';
import { pluginRss } from '@rspress/plugin-rss';
import { pluginFontOpenSans } from 'rspress-plugin-font-open-sans';
import { pluginOpenGraph } from 'rsbuild-plugin-open-graph';
import { pluginGoogleAnalytics } from 'rsbuild-plugin-google-analytics';

const PUBLISH_URL = 'https://rspack.dev';
const COPYRIGHT = '© 2022-present ByteDance Inc. All Rights Reserved.';

function getI18nHelper(lang: 'zh' | 'en') {
  const isZh = lang === 'zh';
  const prefix = isZh ? '/zh' : '';
  const getLink = (str: string) => `${prefix}${str}`;
  const getText = (zhText: string, enText: string) => (isZh ? zhText : enText);
  return { getText, getLink };
}

function getNavConfig(lang: 'zh' | 'en'): NavItem[] {
  const { getText, getLink } = getI18nHelper(lang);
  return [
    {
      text: getText('指南', 'Guide'),
      link: getLink('/guide/introduction'),
      activeMatch: '/guide/',
    },
    {
      text: getText('配置', 'Config'),
      link: getLink('/config'),
      activeMatch: '/config',
    },
    {
      text: getText('插件', 'Plugin'),
      link: getLink('/plugins'),
      activeMatch: '^(/zh|/en)?/plugins',
    },
    {
      text: getText('API', 'API'),
      link: getLink('/api'),
      activeMatch: '/api',
    },
    {
      text: getText('博客', 'Blog'),
      link: getLink('/blog/announcing-0.6'),
      activeMatch: '/blog',
    },
    {
      text: getText('生态', 'Ecosystem'),
      items: [
        {
          text: 'Rsbuild',
          link: 'https://rsbuild.dev',
        },
        {
          text: 'Rspress',
          link: 'https://rspress.dev',
        },
        {
          text: 'Rsdoctor',
          link: 'https://rsdoctor.dev',
        },
        {
          text: 'Modern.js',
          link: 'https://modernjs.dev/en/',
        },
        {
          text: 'Nx Rspack plugin',
          link: 'https://nx.dev/packages/rspack/documents/overview',
        },
        {
          text: 'Awesome Rspack',
          link: 'https://github.com/web-infra-dev/awesome-rspack',
        },
        {
          text: 'Rspack Compat',
          link: 'https://github.com/web-infra-dev/rspack-compat',
        },
        {
          text: 'Rspack Examples',
          link: 'https://github.com/rspack-contrib/rspack-examples',
        },
        {
          text: 'Rsfamily Design Resources',
          link: 'https://github.com/rspack-contrib/rsfamily-design-resources',
        },
        {
          text: 'Rspack Community Packages',
          link: 'https://github.com/rspack-contrib',
        },
      ],
    },
    {
      text: getText('关于', 'About'),
      items: [
        {
          text: getText('加入我们', 'Join Us'),
          link: getLink('/misc/join-us'),
        },
        {
          text: getText('团队', 'Team'),
          link: getLink('/misc/meet-the-team'),
        },
        {
          text: getText('发布日志', 'Releases'),
          link: 'https://github.com/web-infra-dev/rspack/releases',
        },
        {
          text: getText(
            '未来默认行为与功能废弃',
            'Future behavior & Deprecation',
          ),
          link: getLink('/misc/future'),
        },
        {
          text: getText('功能规划', 'Roadmap'),
          link: getLink('/misc/roadmap'),
        },
        {
          text: getText('基准测试', 'Benchmark'),
          link: getLink('/misc/benchmark'),
        },
        {
          text: getText('贡献指南', 'Contributing Guide'),
          link: 'https://github.com/web-infra-dev/rspack/blob/main/CONTRIBUTING.md',
        },
        {
          text: getText('品牌指南', 'Branding Guideline'),
          link: getLink('/misc/branding'),
        },
      ],
    },
  ];
}

function getSidebarConfig(lang: 'zh' | 'en'): Sidebar {
  const { getText, getLink } = getI18nHelper(lang);

  return {
    [getLink('/guide/')]: [
      {
        collapsible: false,
        text: getText('开始', 'Getting started'),
        items: [
          getLink('/guide/introduction'),
          getLink('/guide/quick-start'),
          getLink('/guide/migrate-from-webpack'),
          getLink('/guide/migrate-from-cra'),
          getLink('/guide/migrate-storybook'),
        ],
      },
      {
        collapsible: false,
        text: getText('特性', 'Features'),
        items: [
          getLink('/guide/language-support'),
          {
            link: getLink('/guide/asset-module'),
            text: getText('资源模块', 'Asset modules'),
          },
          {
            link: getLink('/guide/web-workers'),
            text: getText('Web Workers', 'Web Workers'),
          },
          getLink('/guide/loader'),
          getLink('/guide/builtin-swc-loader'),
          {
            link: getLink('/guide/plugin'),
            text: getText('Plugin', 'Plugin'),
          },
          getLink('/guide/module-resolution'),
          getLink('/guide/module-federation'),
          getLink('/guide/dev-server'),
        ],
      },
      {
        collapsible: false,
        text: getText('优化', 'Optimization'),
        items: [
          getLink('/guide/production'),
          getLink('/guide/code-splitting'),
          getLink('/guide/tree-shaking'),
          getLink('/guide/analysis'),
          getLink('/guide/profile'),
        ],
      },
      {
        collapsible: false,
        text: getText('框架支持', 'Framework support'),
        items: [
          getLink('/guide/react'),
          getLink('/guide/solid'),
          getLink('/guide/vue'),
          getLink('/guide/svelte'),
          getLink('/guide/nestjs'),
        ],
      },
      {
        collapsible: false,
        text: getText('兼容性', 'Compatibility'),
        items: [
          getLink('/guide/loader-compat'),
          getLink('/guide/plugin-compat'),
          getLink('/guide/compat-others'),
          getLink('/guide/config-diff'),
        ],
      },
      {
        collapsible: false,
        text: getText('其他', 'Misc'),
        items: [
          getLink('/misc/glossary'),
          getLink('/misc/faq'),
          getLink('/misc/roadmap'),
          getLink('/misc/join-us'),
          getLink('/misc/meet-the-team'),
          getLink('/misc/license'),
          getLink('/misc/branding'),
          getLink('/misc/benchmark'),
        ],
      },
    ],
    [getLink('/config/')]: [
      {
        text: getText('配置', 'Config'),
        link: getLink('/config'),
      },
      {
        text: getText('Entry 入口', 'Entry'),
        link: getLink('/config/entry'),
      },
      {
        text: getText('Context 基础目录', 'Context'),
        link: getLink('/config/context'),
      },
      {
        text: getText('Mode 模式', 'Mode'),
        link: getLink('/config/mode'),
      },
      {
        text: getText('Output 输出', 'Output'),
        link: getLink('/config/output'),
      },
      {
        text: getText('Module 模块', 'Module'),
        link: getLink('/config/module'),
      },
      {
        text: getText('Resolve 模块解析', 'Resolve'),
        link: getLink('/config/resolve'),
      },
      {
        text: getText('ResolveLoader Loader解析', 'ResolveLoader'),
        link: getLink('/config/resolve-loader'),
      },
      {
        text: getText('Node 全局变量', 'Node'),
        link: getLink('/config/node'),
      },
      {
        text: getText('Optimization 优化', 'Optimization'),
        link: getLink('/config/optimization'),
      },
      {
        text: getText('Plugins 插件', 'Plugins'),
        link: getLink('/config/plugins'),
      },
      {
        text: getText('DevServer 开发服务器', 'DevServer'),
        link: getLink('/config/dev-server'),
      },
      {
        text: getText('Cache 缓存', 'Cache'),
        link: getLink('/config/cache'),
      },
      {
        text: getText('Snapshot 缓存快照', 'Snapshot'),
        link: getLink('/config/snapshot'),
      },
      {
        text: getText('Devtool 调试', 'Devtool'),
        link: getLink('/config/devtool'),
      },
      {
        text: getText('Target 目标环境与兼容性', 'Target'),
        link: getLink('/config/target'),
      },
      {
        text: getText('Watch 监听变更', 'Watch'),
        link: getLink('/config/watch'),
      },
      {
        text: getText('Externals 外部依赖', 'Externals'),
        link: getLink('/config/externals'),
      },
      {
        text: getText('Stats 打包信息', 'Stats'),
        link: getLink('/config/stats'),
      },
      {
        text: getText('Experiments 实验功能', 'Experiments'),
        link: getLink('/config/experiments'),
      },
      {
        text: getText('Builtins 内置功能', 'Builtins'),
        link: getLink('/config/builtins'),
      },
      {
        text: getText('其他配置', 'Other Options'),
        link: getLink('/config/other-options'),
      },
    ],
    [getLink('/plugins/')]: [
      {
        text: getText('简介', 'Introduction'),
        link: getLink('/plugins'),
      },
      {
        text: getText(
          '同步自 webpack 的内置插件',
          'Webpack-aligned Built-in Plugins',
        ),
        items: [
          {
            text: 'EntryPlugin',
            link: getLink('/plugins/webpack/entry-plugin'),
          },
          {
            text: 'DefinePlugin',
            link: getLink('/plugins/webpack/define-plugin'),
          },
          {
            text: 'ProvidePlugin',
            link: getLink('/plugins/webpack/provide-plugin'),
          },
          {
            text: 'BannerPlugin',
            link: getLink('/plugins/webpack/banner-plugin'),
          },
          {
            text: 'HotModuleReplacementPlugin',
            link: getLink('/plugins/webpack/hot-module-replacement-plugin'),
          },
          {
            text: 'IgnorePlugin',
            link: getLink('/plugins/webpack/ignore-plugin'),
          },
          {
            text: 'ProgressPlugin',
            link: getLink('/plugins/webpack/progress-plugin'),
          },
          {
            text: 'ExternalsPlugin',
            link: getLink('/plugins/webpack/externals-plugin'),
          },
          {
            text: 'SourceMapDevToolPlugin',
            link: getLink('/plugins/webpack/source-map-dev-tool-plugin'),
          },
          {
            text: 'SplitChunksPlugin',
            link: getLink('/plugins/webpack/split-chunks-plugin'),
          },
          {
            text: 'NodeTargetPlugin',
            link: getLink('/plugins/webpack/node-target-plugin'),
          },
          {
            text: 'NodeTemplatePlugin',
            link: getLink('/plugins/webpack/node-template-plugin'),
          },
          {
            text: 'EnableChunkLoadingPlugin',
            link: getLink('/plugins/webpack/enable-chunk-loading-plugin'),
          },
          {
            text: 'EnableLibraryPlugin',
            link: getLink('/plugins/webpack/enable-library-plugin'),
          },
          {
            text: 'EnableWasmLoadingPlugin',
            link: getLink('/plugins/webpack/enable-wasm-loading-plugin'),
          },
          {
            text: 'ElectronTargetPlugin',
            link: getLink('/plugins/webpack/electron-target-plugin'),
          },
          {
            text: 'ModuleFederationPlugin',
            link: getLink('/plugins/webpack/module-federation-plugin'),
          },
          {
            text: 'ModuleFederationPluginV1',
            link: getLink('/plugins/webpack/module-federation-plugin-v1'),
          },
        ],
      },
      {
        text: getText('Rspack 独有的内置插件', 'Rspack-only Built-in Plugins'),
        items: [
          {
            text: 'HtmlRspackPlugin',
            link: getLink('/plugins/rspack/html-rspack-plugin'),
          },
          {
            text: 'SwcJsMinimizerRspackPlugin',
            link: getLink('/plugins/rspack/swc-js-minimizer-rspack-plugin'),
          },
          {
            text: 'SwcCssMinimizerRspackPlugin',
            link: getLink('/plugins/rspack/swc-css-minimizer-rspack-plugin'),
          },
          {
            text: 'CopyRspackPlugin',
            link: getLink('/plugins/rspack/copy-rspack-plugin'),
          },
          {
            text: 'CssExtractRspackPlugin',
            link: getLink('/plugins/rspack/css-extract-rspack-plugin'),
          },
        ],
      },
    ],
    [getLink('/api/')]: [
      {
        text: getText('简介', 'Introduction'),
        link: getLink('/api'),
      },
      {
        text: getText('CLI', 'CLI'),
        link: getLink('/api/cli'),
      },
      {
        text: getText('模块', 'Modules'),
        link: getLink('/api/modules'),
      },
      {
        text: getText('Node API', 'Node API'),
        link: getLink('/api/node-api'),
      },
      {
        text: getText('Hot Module Replacement', 'Hot Module Replacement'),
        link: getLink('/api/hmr'),
      },
      {
        text: getText('Loader API', 'Loader API'),
        link: getLink('/api/loader-api'),
      },
      {
        text: getText('插件 API', 'Plugin API'),
        link: getLink('/api/plugin-api'),
      },
    ],
    [getLink('/blog/')]: [
      {
        text: getText('0.6 发布公告', 'Announcing Rspack 0.6'),
        link: getLink('/blog/announcing-0.6'),
      },
      {
        text: getText('0.5 发布公告', 'Announcing Rspack 0.5'),
        link: getLink('/blog/announcing-0.5'),
      },
      {
        text: getText(
          'Rspack 支持模块联邦',
          'Module Federation added to Rspack',
        ),
        link: getLink('/blog/module-federation-added-to-rspack'),
      },
      {
        text: getText('0.4 发布公告', 'Announcing Rspack 0.4'),
        link: getLink('/blog/announcing-0.4'),
      },
      {
        text: getText('0.3 发布公告', 'Announcing Rspack 0.3'),
        link: getLink('/blog/announcing-0.3'),
      },
      {
        text: getText('0.2 发布公告', 'Announcing Rspack 0.2'),
        link: getLink('/blog/announcing-0.2'),
      },
      {
        text: getText('发布公告', 'Announcing Rspack'),
        link: getLink('/blog/announcement'),
      },
    ],
  };
}

export default defineConfig({
  root: path.join(__dirname, 'docs'),
  title: 'Rspack',
  description: 'A fast Rust-based web bundler',
  logo: {
    light:
      'https://lf3-static.bytednsdoc.com/obj/eden-cn/rjhwzy/ljhwZthlaukjlkulzlp/navbar-logo-2027.png',
    dark: 'https://lf3-static.bytednsdoc.com/obj/eden-cn/rjhwzy/ljhwZthlaukjlkulzlp/navbar-logo-dark-2027.png',
  },
  icon: 'https://lf3-static.bytednsdoc.com/obj/eden-cn/rjhwzy/ljhwZthlaukjlkulzlp/favicon-1714.png',
  lang: 'en',
  globalStyles: path.join(__dirname, 'theme', 'index.css'),
  markdown: {
    checkDeadLinks: true,
  },
  plugins: [
    pluginFontOpenSans(),
    pluginRss({
      siteUrl: PUBLISH_URL,
      feed: [
        {
          id: 'blog-rss',
          test: '/blog',
          title: 'Rspack Blog',
          language: 'en',
          output: {
            type: 'rss',
            filename: 'blog-rss.xml',
          },
        },
        {
          id: 'blog-rss-zh',
          test: '/zh/blog',
          title: 'Rspack 博客',
          language: 'zh-CN',
          output: {
            type: 'rss',
            filename: 'blog-rss-zh.xml',
          },
        },
      ],
    }),
  ],
  themeConfig: {
    footer: {
      message: COPYRIGHT,
    },
    socialLinks: [
      {
        icon: 'github',
        mode: 'link',
        content: 'https://github.com/web-infra-dev/rspack',
      },
      {
        icon: 'discord',
        mode: 'link',
        content: 'https://discord.gg/79ZZ66GH9E',
      },
      {
        icon: 'twitter',
        mode: 'link',
        content: 'https://twitter.com/rspack_dev',
      },
      {
        icon: 'lark',
        mode: 'link',
        content:
          'https://applink.feishu.cn/client/chat/chatter/add_by_link?link_token=3c3vca77-bfc0-4ef5-b62b-9c5c9c92f1b4',
      },
    ],
    locales: [
      {
        lang: 'en',
        title: 'Rspack',
        description: 'A fast Rust-based web bundler',
        nav: getNavConfig('en'),
        sidebar: getSidebarConfig('en'),
        label: 'English',
      },
      {
        lang: 'zh',
        title: 'Rspack',
        description: '基于 Rust 的高性能 Web 构建工具',
        nav: getNavConfig('zh'),
        sidebar: getSidebarConfig('zh'),
        label: '简体中文',
      },
    ],
  },
  builderConfig: {
    plugins: [
      pluginGoogleAnalytics({ id: 'G-XKKCNZZNJD' }),
      pluginOpenGraph({
        title: 'Rspack',
        type: 'website',
        url: PUBLISH_URL,
        image: 'https://assets.rspack.dev/rspack/rspack-banner.png',
        description: 'Fast Rust-based Web Bundler',
        twitter: {
          site: '@rspack_dev',
          card: 'summary_large_image',
        },
      }),
    ],
    source: {
      alias: {
        '@builtIns': path.join(__dirname, 'components', 'builtIns'),
        '@components': path.join(__dirname, 'components'),
        '@hooks': path.join(__dirname, 'hooks'),
      },
    },
    dev: {
      startUrl: true,
    },
    output: {
      copy: {
        patterns: [
          {
            from: path.join(__dirname, 'docs', 'public', '_redirects'),
          },
          {
            from: path.join(__dirname, 'docs', 'public', '_headers'),
          },
        ],
      },
    },
  },
});
