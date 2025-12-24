import { NoSSR, useLang, usePageData } from '@rspress/core/runtime';
import {
  Layout as BaseLayout,
  getCustomMDXComponent as basicGetCustomMDXComponent,
} from '@rspress/core/theme-original';
import {
  Search as PluginAlgoliaSearch,
  ZH_LOCALES,
} from '@rspress/plugin-algolia/runtime';
import {
  LlmsContainer,
  LlmsCopyButton,
  LlmsViewOptions,
} from '@rspress/plugin-llms/runtime';
import { Announcement } from '@rstack-dev/doc-ui/announcement';
import { ConfigProvider } from '@rstack-dev/doc-ui/antd';
import { NavIcon } from '@rstack-dev/doc-ui/nav-icon';
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

function getCustomMDXComponent() {
  const { h1: H1, ...components } = basicGetCustomMDXComponent();

  const MyH1 = ({ ...props }) => {
    return (
      <>
        <H1 {...props} />
        <LlmsContainer>
          <LlmsCopyButton />
          <LlmsViewOptions />
        </LlmsContainer>
      </>
    );
  };
  return {
    ...components,
    h1: MyH1,
  };
}

export { Layout, HomeLayout, Search, getCustomMDXComponent };

export * from '@rspress/core/theme-original';
