import { Announcement } from '@rstack-dev/doc-ui/announcement';
import { ConfigProvider } from '@rstack-dev/doc-ui/antd';
import { NavIcon } from '@rstack-dev/doc-ui/nav-icon';
import { NoSSR, useLang, usePageData } from 'rspress/runtime';
import { Layout as BaseLayout } from 'rspress/theme';
import { HomeLayout } from './pages';

// Enable this when we need a new announcement
const ANNOUNCEMENT_URL = '';

const Layout = () => {
  const { page } = usePageData();
  const lang = useLang();
  return (
    <ConfigProvider
      theme={{
        // Update tokens for Collapse in dark mode
        token: {
          colorBorder: 'var(--rp-c-divider)',
        },
        components: {
          Collapse: {
            contentBg: 'transparent',
          },
        },
      }}
    >
      <BaseLayout
        beforeNavTitle={<NavIcon />}
        beforeNav={
          ANNOUNCEMENT_URL && (
            <NoSSR>
              <Announcement
                href={
                  lang === 'en'
                    ? ANNOUNCEMENT_URL
                    : `/${lang}${ANNOUNCEMENT_URL}`
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
    </ConfigProvider>
  );
};

export { Layout, HomeLayout };

export * from 'rspress/theme';
