import path from 'node:path';
import { pluginSass } from '@rsbuild/plugin-sass';
import { defineConfig } from '@rspress/core';
import { pluginAlgolia } from '@rspress/plugin-algolia';
import { pluginLlms } from '@rspress/plugin-llms';
import { pluginRss } from '@rspress/plugin-rss';
import { pluginSitemap } from '@rspress/plugin-sitemap';
import { transformerNotationHighlight } from '@shikijs/transformers';
import { pluginGoogleAnalytics } from 'rsbuild-plugin-google-analytics';
import { pluginOpenGraph } from 'rsbuild-plugin-open-graph';
import { pluginFontOpenSans } from 'rspress-plugin-font-open-sans';

const PUBLISH_URL = 'https://rspack.rs';

export default defineConfig({
  root: path.join(__dirname, 'docs'),
  title: 'Rspack',
  description:
    'Rspack is a high performance JavaScript bundler written in Rust. It offers strong compatibility with the webpack ecosystem, and lightning fast build speeds.',
  logo: {
    light: 'https://assets.rspack.rs/rspack/navbar-logo-light.png',
    dark: 'https://assets.rspack.rs/rspack/navbar-logo-dark.png',
  },
  icon: 'https://assets.rspack.rs/rspack/favicon-128x128.png',
  lang: 'en',
  globalStyles: path.join(__dirname, 'theme', 'index.css'),
  markdown: {
    shiki: {
      transformers: [transformerNotationHighlight()],
      langAlias: {
        ejs: 'js',
      },
    },
  },
  search: {
    codeBlocks: true,
  },
  route: {
    cleanUrls: true,
    exclude: ['**/types/*.mdx'],
  },
  plugins: [
    pluginAlgolia(),
    pluginLlms(),
    pluginSitemap({
      siteUrl: PUBLISH_URL,
    }),
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
        icon: 'x',
        mode: 'link',
        content: 'https://twitter.com/rspack_dev',
      },
      {
        icon: 'bluesky',
        mode: 'link',
        content: 'https://bsky.app/profile/rspack.dev',
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
  head: [
    ({ routePath }) => {
      const getOgImage = () => {
        if (routePath.includes('blog/announcing-')) {
          const version = routePath.split('announcing-')[1];
          return `assets/rspack-og-image-v${version}.png`;
        }
        if (routePath.endsWith('blog/rspack-next-partner')) {
          return 'assets/next-rspack-og-image.png';
        }
        // default
        return 'rspack-og-image.png';
      };
      return `<meta property="og:image" content="https://assets.rspack.rs/rspack/${getOgImage()}">`;
    },
  ],
  builderConfig: {
    dev: {
      lazyCompilation: true,
    },
    plugins: [
      pluginSass(),
      pluginGoogleAnalytics({ id: 'G-XKKCNZZNJD' }),
      pluginOpenGraph({
        url: PUBLISH_URL,
        description: 'Fast Rust-based web bundler',
        twitter: {
          site: '@rspack_dev',
          card: 'summary_large_image',
        },
      }),
    ],
    source: {
      preEntry: ['./theme/tailwind.css'],
    },
    resolve: {
      alias: {
        '@builtIns': path.join(__dirname, 'components', 'builtIns'),
        '@components': path.join(__dirname, 'components'),
        '@hooks': path.join(__dirname, 'hooks'),
      },
    },
    server: {
      open: true,
    },
    html: {
      tags: [
        // for baidu SEO verification
        {
          tag: 'meta',
          attrs: {
            name: 'baidu-site-verification',
            content: 'codeva-bE2dFTowhk',
          },
        },
      ],
    },
  },
});
