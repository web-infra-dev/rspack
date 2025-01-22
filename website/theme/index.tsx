import { Announcement } from '@rstack-dev/doc-ui/announcement';
import { NavIcon } from '@rstack-dev/doc-ui/nav-icon';
import { NoSSR, useLang, usePageData } from 'rspress/runtime';
import Theme from 'rspress/theme';
import { HomeLayout } from './pages';

// Enable this when we need a new announcement
const ANNOUNCEMENT_URL = '';

const Layout = () => {
  const { page } = usePageData();
  const lang = useLang();
  return (
    <Theme.Layout
      beforeNavTitle={<NavIcon />}
      beforeNav={
        ANNOUNCEMENT_URL && (
          <NoSSR>
            <Announcement
              href={
                lang === 'en' ? ANNOUNCEMENT_URL : `/${lang}${ANNOUNCEMENT_URL}`
              }
              message={
                lang === 'en'
                  ? 'Rspack 1.0 has been released!'
                  : 'Rspack 1.0 正式发布！'
              }
              localStorageKey="rspack-announcement-closed"
              display={page.pageType === 'home'}
            />
          </NoSSR>
        )
      }
    />
  );
};

export * from 'rspress/theme';

export default {
  ...Theme,
  Layout,
  HomeLayout,
};
