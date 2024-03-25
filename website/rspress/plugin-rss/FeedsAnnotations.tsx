import { Helmet, usePageData } from 'rspress/runtime';

export default function FeedsAnnotations() {
  const { page } = usePageData();
  const feeds = (page.feeds as { href: string }[]) || [];

  return (
    <Helmet>
      {feeds.map(({ href }) => (
        <link
          rel="alternate"
          type="application/rss+xml"
          href={href}
          hrefLang={page.lang}
        />
      ))}
    </Helmet>
  );
}
