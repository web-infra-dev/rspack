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

function getSidebarConfig(lang: 'zh' | 'en'): Sidebar {
  const { getText, getLink } = getI18nHelper(lang);

  return {
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
        label: 'English',
      },
      {
        lang: 'zh',
        title: 'Rspack',
        description: '基于 Rust 的高性能 Web 构建工具',
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
