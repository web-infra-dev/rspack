import { useEffect } from 'react';

type RedirectRule = {
  from: string | string[];
  to: string;
};

type LocalizedClientRedirectsProps = {
  redirects?: RedirectRule[];
};

export default function LocalizedClientRedirects({
  redirects = [],
}: LocalizedClientRedirectsProps) {
  useEffect(() => {
    const { pathname, hash } = window.location;

    // Localized 404 routes hide the original pathname from pluginClientRedirects.
    if (!pathname.startsWith('/zh/')) {
      return;
    }

    for (const { from, to } of redirects) {
      const patterns = Array.isArray(from) ? from : [from];

      for (const pattern of patterns) {
        const regexp = new RegExp(pattern);

        if (regexp.test(pathname)) {
          window.location.replace(pathname.replace(regexp, to) + hash);
          return;
        }
      }
    }
  }, [redirects]);

  return null;
}
