import { Announcement } from '@rstack-dev/doc-ui/announcement';
import { ConfigProvider } from '@rstack-dev/doc-ui/antd';
import { NavIcon } from '@rstack-dev/doc-ui/nav-icon';
import { NoSSR, useLang, usePageData } from 'rspress/runtime';
import { Layout as BaseLayout } from 'rspress/theme';
import type { LangTypes } from './i18n';
import { HomeLayout } from './pages';

// Enable this when we need a new announcement
const ANNOUNCEMENT_URL = '';

const Layout = () => {
  const { page } = usePageData();
  const lang = useLang() as LangTypes;
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
                href={refByLanguage(lang)}
                message={announcementMessage(lang)}
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

function refByLanguage(lang: LangTypes) {
  return lang === 'en' ? ANNOUNCEMENT_URL : `/${lang}${ANNOUNCEMENT_URL}`;
}

function announcementMessage(lang: LangTypes) {
  const message: Record<LangTypes, string> = {
    en: 'Rspack 1.0 has been released!',
    zh: 'Rspack 1.0 正式发布！',
    ptBR: 'O Rspack 1.0 foi lançado!',
  };
  return message[lang] ?? 'Rspack 1.0 has been released!';
}

export { Layout, HomeLayout };

export * from 'rspress/theme';
