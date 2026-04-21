import type { BlogAvatarAuthor } from '@rstack-dev/doc-ui/blog-avatar';
import type { BlogListItem } from '@rstack-dev/doc-ui/blog-list';
import { useLang, usePages } from '@rspress/core/runtime';

const DEFAULT_AUTHOR: BlogAvatarAuthor = {
  name: 'Rspack Team',
  avatar: 'https://assets.rspack.rs/rspack/rspack-logo.svg',
  github: 'https://github.com/web-infra-dev',
  x: 'https://x.com/rspack_dev',
  title: 'Rspack contributors',
};

type BlogFrontmatter = {
  description?: string;
  date?: string;
  authors?: BlogAvatarAuthor[];
};

export type BlogPage = BlogListItem & {
  filename?: string;
};

export const useBlogPages = (): BlogPage[] => {
  const { pages } = usePages();
  const lang = useLang();

  const blogPages = pages
    .filter((page) => page.lang === lang)
    .filter(
      (page) =>
        page.routePath.includes('/blog/') && !page.routePath.endsWith('/blog/'),
    )
    .sort((a, b) => {
      const frontmatterA = (a.frontmatter ?? {}) as BlogFrontmatter;
      const frontmatterB = (b.frontmatter ?? {}) as BlogFrontmatter;
      const dateA = frontmatterA.date
        ? new Date(frontmatterA.date)
        : new Date(0);
      const dateB = frontmatterB.date
        ? new Date(frontmatterB.date)
        : new Date(0);

      return dateB.getTime() - dateA.getTime();
    });

  return blogPages.map((page) => {
    const frontmatter = (page.frontmatter ?? {}) as BlogFrontmatter;

    return {
      id: page.routePath,
      title: page.title,
      description: frontmatter.description,
      date: frontmatter.date,
      href: page.routePath,
      authors: frontmatter.authors?.length
        ? frontmatter.authors
        : [DEFAULT_AUTHOR],
      filename: page.routePath.split('/').pop(),
    };
  });
};
