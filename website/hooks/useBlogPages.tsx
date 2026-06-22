import type { BlogAvatarAuthor } from '@rstack-dev/doc-ui/blog-avatar';
import type { BlogListItem } from '@rstack-dev/doc-ui/blog-list';
import { useLang, usePages } from '@rspress/core/runtime';

const DEFAULT_AUTHOR: BlogAvatarAuthor = {
  name: 'Rspack Team',
  avatar: 'https://assets.rspack.rs/rspack/rspack-logo-with-background.png',
  github: 'https://github.com/web-infra-dev',
  x: 'https://x.com/rspack_dev',
  title: 'Rspack contributors',
};

type BlogFrontmatter = {
  description?: string;
  date?: string;
  authors?: BlogAvatarAuthor[];
};

// These legacy technical articles live in GitHub Discussions instead of local
// Rspress pages, but they still belong in the blog archive for both locales.
const EXTERNAL_BLOG_PAGES: Record<string, BlogListItem[]> = {
  en: [
    {
      title: 'Bundler tree shaking principles and differences',
      description:
        'Tree shaking has become an essential part of modern front-end bundling. This article provides a brief overview of tree shaking principles across different bundlers and explores their key differences.',
      date: '2025-07-31',
      href: 'https://github.com/orgs/web-infra-dev/discussions/29',
      authors: [
        {
          name: 'ahabhgk',
          avatar: 'https://github.com/ahabhgk.png',
        },
      ],
    },
    {
      title: 'How to embed a HashMap with lots of strings in program',
      description:
        'This article introduces techniques for constructing a completely static and efficient Map using MPHF and string packing, avoiding initialization, parsing, and memory allocation while improving binary size and compile speed.',
      date: '2025-07-02',
      href: 'https://github.com/orgs/web-infra-dev/discussions/27',
      authors: [
        {
          name: 'quininer',
          avatar: 'https://github.com/quininer.png',
        },
      ],
    },
    {
      title: 'Build systems and bundlers',
      description:
        'This article will briefly introduce the content of the "Build Systems à la Carte: Theory and Practice" paper and attempt to summarize bundlers from the perspective of build systems.',
      date: '2025-01-07',
      href: 'https://github.com/orgs/web-infra-dev/discussions/24',
      authors: [
        {
          name: 'ahabhgk',
          avatar: 'https://github.com/ahabhgk.png',
        },
      ],
    },
    {
      title: 'RSC and Server Action bundle practice',
      description:
        'This article introduces the construction practices of RSC (React Server Components) and Server Action in React, including their concepts, rendering methods, bundling process in webpack, and how Turbopack bundles multiple environment modules in a module diagram.',
      date: '2025-01-06',
      href: 'https://github.com/orgs/web-infra-dev/discussions/23',
      authors: [
        {
          name: 'ahabhgk',
          avatar: 'https://github.com/ahabhgk.png',
        },
      ],
    },
    {
      title: 'Deep dive into Rspack tree shaking',
      description:
        'This article primarily focuses on understanding the concept of Rspack & webpack tree shaking.',
      date: '2024-04-17',
      href: 'https://github.com/orgs/web-infra-dev/discussions/17',
      authors: [
        {
          name: 'hardfist',
          avatar: 'https://github.com/hardfist.png',
        },
      ],
    },
    {
      title: 'Webpack chunk graph algorithm',
      description:
        'This article introduces the chunk strategy of webpack. Through this article, you can understand when a chunk will be generated in the code and how to reduce the chunk size, etc.',
      date: '2024-01-12',
      href: 'https://github.com/orgs/web-infra-dev/discussions/15',
      authors: [
        {
          name: 'JSerFeng',
          avatar: 'https://github.com/JSerFeng.png',
        },
      ],
    },
    {
      title: 'Webpack CSS order issue',
      description:
        'This article shows how the CSS order problem occurs in webpack and how to solve it.',
      date: '2023-11-29',
      href: 'https://github.com/orgs/web-infra-dev/discussions/12',
      authors: [
        {
          name: 'JSerFeng',
          avatar: 'https://github.com/JSerFeng.png',
        },
      ],
    },
    {
      title: 'Deep dive into Top-level await',
      description:
        'In this article, we will take a closer look at aspects such as the specification, toolchain support, webpack runtime, and profiling of Top-level await.',
      date: '2023-10-26',
      href: 'https://github.com/orgs/web-infra-dev/discussions/9',
      authors: [
        {
          name: 'ulivz',
          avatar: 'https://github.com/ulivz.png',
        },
      ],
    },
    {
      title: 'Design trade-offs in bundler',
      description:
        'This article explains why we decided to develop Rspack and what trade-offs we made during the design process.',
      date: '2023-08-30',
      href: 'https://github.com/orgs/web-infra-dev/discussions/1',
      authors: [
        {
          name: 'hardfist',
          avatar: 'https://github.com/hardfist.png',
        },
      ],
    },
  ],
  zh: [
    {
      title: 'Bundler tree shaking 原理及差异',
      description:
        'Tree shaking 已经成为现代前端打包工具的重要组成部分。本文简要概述了不同打包工具中 tree shaking 的原理，并探讨了它们之间的主要差异。',
      date: '2025-07-31',
      href: 'https://github.com/orgs/web-infra-dev/discussions/28',
      authors: [
        {
          name: 'ahabhgk',
          avatar: 'https://github.com/ahabhgk.png',
        },
      ],
    },
    {
      title: '如何在程序中嵌入有大量字符串的 HashMap',
      description:
        '本文介绍了如何通过 MPHF 和字符串打包技术构造完全静态且高效的 Map，避免初始化、解析和内存分配，同时改善产物体积与编译速度。',
      date: '2025-07-02',
      href: 'https://github.com/orgs/web-infra-dev/discussions/26',
      authors: [
        {
          name: 'quininer',
          avatar: 'https://github.com/quininer.png',
        },
      ],
    },
    {
      title: '构建系统与前端打包工具',
      description:
        '本文会简单介绍 "Build Systems à la Carte: Theory and Practice" 这篇论文的内容，并尝试从 build system 的角度来概括 bundlers。',
      date: '2025-01-07',
      href: 'https://github.com/orgs/web-infra-dev/discussions/22',
      authors: [
        {
          name: 'ahabhgk',
          avatar: 'https://github.com/ahabhgk.png',
        },
      ],
    },
    {
      title: 'RSC 和 Server Action 构建实践',
      description:
        '本文介绍了 React 中 RSC（React Server Components）和 Server Action 的构建实践，包括它们的概念、渲染方式、在 webpack 中的打包流程，以及 Turbopack 是如何在一个模块图中完成打包多个环境模块的。',
      date: '2025-01-06',
      href: 'https://github.com/orgs/web-infra-dev/discussions/21',
      authors: [
        {
          name: 'ahabhgk',
          avatar: 'https://github.com/ahabhgk.png',
        },
      ],
    },
    {
      title: 'Deep dive into Rspack tree shaking',
      description:
        '本文主要侧重于理解 Rspack 和 webpack 中 tree shaking 的概念。',
      date: '2024-04-17',
      href: 'https://github.com/orgs/web-infra-dev/discussions/17',
      authors: [
        {
          name: 'hardfist',
          avatar: 'https://github.com/hardfist.png',
        },
      ],
    },
    {
      title: 'Webpack chunk graph 策略',
      description:
        '本文介绍了 webpack 的 chunk 策略，通过这篇文章，你可以理解代码中什么时候会产生 chunk，怎样减少 chunk 体积等。',
      date: '2024-01-12',
      href: 'https://github.com/orgs/web-infra-dev/discussions/16',
      authors: [
        {
          name: 'JSerFeng',
          avatar: 'https://github.com/JSerFeng.png',
        },
      ],
    },
    {
      title: 'Webpack CSS 顺序问题',
      description:
        '本文介绍了 webpack 中 CSS 顺序问题是怎样产生的，以及如何解决。',
      date: '2023-11-29',
      href: 'https://github.com/orgs/web-infra-dev/discussions/13',
      authors: [
        {
          name: 'JSerFeng',
          avatar: 'https://github.com/JSerFeng.png',
        },
      ],
    },
    {
      title: '深入了解 Top-level await',
      description:
        '在本文中，我们将对 Top-level await 的 specification、toolchain support、webpack runtime、profiling 等方面进行深入的分析。',
      date: '2023-10-26',
      href: 'https://github.com/orgs/web-infra-dev/discussions/10',
      authors: [
        {
          name: 'ulivz',
          avatar: 'https://github.com/ulivz.png',
        },
      ],
    },
    {
      title: 'Bundler 的设计取舍',
      description:
        '本文介绍了我们为什么要开发 Rspack，设计过程中进行了哪些取舍。',
      date: '2023-08-30',
      href: 'https://github.com/orgs/web-infra-dev/discussions/4',
      authors: [
        {
          name: 'hardfist',
          avatar: 'https://github.com/hardfist.png',
        },
      ],
    },
  ],
};

const getDateValue = (date?: BlogListItem['date']): number => {
  if (!date) {
    return 0;
  }

  const timestamp = new Date(date).getTime();

  return Number.isNaN(timestamp) ? 0 : timestamp;
};

const withDefaultAuthor = (page: BlogListItem): BlogListItem => ({
  ...page,
  authors: page.authors?.length ? page.authors : [DEFAULT_AUTHOR],
});

export const useBlogPages = (): BlogListItem[] => {
  const { pages } = usePages();
  const lang = useLang();

  const localBlogPages = pages
    .filter((page) => page.lang === lang)
    .filter(
      (page) =>
        page.routePath.includes('/blog/') && !page.routePath.endsWith('/blog/'),
    )
    .map((page) => {
      const frontmatter = (page.frontmatter ?? {}) as BlogFrontmatter;

      return withDefaultAuthor({
        title: page.title,
        description: frontmatter.description,
        date: frontmatter.date,
        href: page.routePath,
        authors: frontmatter.authors,
      });
    });

  const externalBlogPages = (EXTERNAL_BLOG_PAGES[lang] ?? []).map((page) =>
    withDefaultAuthor(page),
  );

  return [...localBlogPages, ...externalBlogPages].sort(
    (a, b) => getDateValue(b.date) - getDateValue(a.date),
  );
};
