import { NoSSR, useLang, usePage } from '@rspress/core/runtime';
import { Layout as BaseLayout } from '@rspress/core/theme-original';
import {
  Search as PluginAlgoliaSearch,
  ZH_LOCALES,
} from '@rspress/plugin-algolia/runtime';
import { Announcement } from '@rstack-dev/doc-ui/announcement';
import { ConfigProvider } from '@rstack-dev/doc-ui/antd';
import { NavIcon } from '@rstack-dev/doc-ui/nav-icon';
import { HomeLayout } from './pages';

// Enable this when we need a new announcement
const ANNOUNCEMENT_URL = '';

const Layout = () => {
  const { page } = usePage();
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

const Search = () => {
  const lang = useLang();
  return (
    <PluginAlgoliaSearch
      docSearchProps={{
        appId: 'TQOGCXPBUD', // cspell:disable-line
        apiKey: '8c30f9d1f12e786a132af15ea30cf997', // cspell:disable-line
        indexName: 'rspack',
        searchParameters: {
          facetFilters: [`lang:${lang}`],
        },
      }}
      locales={ZH_LOCALES}
    />
  );
};

export { Layout, HomeLayout, Search };

export * from '@rspress/core/theme-original';
