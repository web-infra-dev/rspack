import path from 'node:path';

import { pluginRss } from '@rspress/plugin-rss';
import { pluginGoogleAnalytics } from 'rsbuild-plugin-google-analytics';
import { pluginOpenGraph } from 'rsbuild-plugin-open-graph';
import { pluginFontOpenSans } from 'rspress-plugin-font-open-sans';
import { defineConfig } from 'rspress/config';

const PUBLISH_URL = 'https://rspack.dev';
const COPYRIGHT = '© 2022-present ByteDance Inc. All Rights Reserved.';

export default defineConfig({
  root: path.join(__dirname, 'docs'),
  title: 'Rspack',
  description: 'The fast Rust-based web bundler',
  logo: {
    light: 'https://assets.rspack.dev/rspack/navbar-logo-light.png',
    dark: 'https://assets.rspack.dev/rspack/navbar-logo-dark.png',
  },
  icon: 'https://assets.rspack.dev/rspack/favicon-128x128.png',
  lang: 'en',
  globalStyles: path.join(__dirname, 'theme', 'index.css'),
  markdown: {
    checkDeadLinks: true,
    highlightLanguages: [['rs', 'rust']],
  },
  route: {
    cleanUrls: true,
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
        content: 'https://discord.gg/sYK4QjyZ4V',
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
        description: 'The fast Rust-based web bundler',
        label: 'English',
        editLink: {
          docRepoBaseUrl:
            'https://github.com/web-infra-dev/rspack/tree/main/website/docs',
          text: '📝 Edit this page on GitHub',
        },
      },
      {
        lang: 'zh',
        title: 'Rspack',
        description: '基于 Rust 的高性能 web 打包工具',
        label: '简体中文',
        editLink: {
          docRepoBaseUrl:
            'https://github.com/web-infra-dev/rspack/tree/main/website/docs',
          text: '📝 在 GitHub 上编辑此页',
        },
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
        image:
          'https://assets.rspack.dev/rspack/assets/rspack-og-image-v1-0-alpha.png',
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
