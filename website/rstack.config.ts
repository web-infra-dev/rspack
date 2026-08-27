import path from 'node:path';
import { define } from 'rstack';

define.doc(async () => {
  const { pluginSass } = await import('@rsbuild/plugin-sass');
  const { pluginTailwindcss } = await import('@rsbuild/plugin-tailwindcss');
  const { pluginAlgolia } = await import('@rspress/plugin-algolia');
  const { pluginClientRedirects } =
    await import('@rspress/plugin-client-redirects');
  const { pluginRss } = await import('@rspress/plugin-rss');
  const { pluginSitemap } = await import('@rspress/plugin-sitemap');
  const { transformerNotationDiff, transformerNotationHighlight } =
    await import('@shikijs/transformers');
  const { pluginGoogleAnalytics } =
    await import('rsbuild-plugin-google-analytics');
  const { pluginOpenGraph } = await import('rsbuild-plugin-open-graph');
  const { pluginFontOpenSans } = await import('rspress-plugin-font-open-sans');

  const PUBLISH_URL = 'https://rspack.rs';
  const description =
    'Fast Rust-based bundler for the web with a modernized webpack API';

  return {
    root: path.join(import.meta.dirname, 'docs'),
    title: 'Rspack',
    description,
    logo: {
      light: 'https://assets.rspack.rs/rspack/navbar-logo-light.png',
      dark: 'https://assets.rspack.rs/rspack/navbar-logo-dark.png',
    },
    icon: 'https://assets.rspack.rs/rspack/favicon-128x128.png',
    lang: 'en',
    globalStyles: path.join(import.meta.dirname, 'theme', 'index.css'),
    markdown: {
      link: {
        checkAnchors: true,
      },
      shiki: {
        transformers: [
          transformerNotationHighlight(),
          transformerNotationDiff(),
        ],
        langAlias: {
          ejs: 'js',
        },
      },
    },
    llms: true,
    search: {
      codeBlocks: true,
    },
    route: {
      cleanUrls: true,
      exclude: ['**/types/*.mdx'],
    },
    plugins: [
      pluginClientRedirects({
        redirects: [
          {
            from: '^(/zh)?/plugins/compat-hashed-chunk-ids-plugin/?$',
            to: '$1/plugins/compact-hashed-chunk-ids-plugin',
          },
          {
            from: '^(/zh)?/plugins/compat-hashed-module-ids-plugin/?$',
            to: '$1/plugins/compact-hashed-module-ids-plugin',
          },
          {
            from: '^(/zh)?/plugins/webpack/warn-case-sensitive-modules-plugin/?$',
            to: '$1/plugins/case-sensitive-plugin',
          },
          {
            from: '^(/zh)?/plugins/webpack(?:/index)?/?$',
            to: '$1/plugins/webpack-built-in-plugin-support',
          },
          {
            from: '^(/zh)?/plugins/rspack/?$',
            to: '$1/plugins/',
          },
          {
            from: '^(/zh)?/plugins/(?:rspack|webpack)/([^/]+)/?$',
            to: '$1/plugins/$2',
          },
        ],
      }),
      pluginAlgolia(),
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
      llmsUI: {
        placement: 'outline',
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
      editLink: {
        docRepoBaseUrl:
          'https://github.com/web-infra-dev/rspack/tree/main/website/docs',
      },
      locales: [
        {
          lang: 'en',
          title: 'Rspack',
          description,
          label: 'English',
        },
        {
          lang: 'zh',
          title: 'Rspack',
          description:
            '基于 Rust 的高性能 Web 打包工具，提供现代化的 webpack API',
          label: '简体中文',
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
      plugins: [
        pluginSass(),
        pluginTailwindcss(),
        pluginGoogleAnalytics({ id: 'G-XKKCNZZNJD' }),
        pluginOpenGraph({
          url: PUBLISH_URL,
          description,
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
          '@builtIns': path.join(import.meta.dirname, 'components', 'builtIns'),
          '@components': path.join(import.meta.dirname, 'components'),
          '@hooks': path.join(import.meta.dirname, 'hooks'),
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
  };
});
