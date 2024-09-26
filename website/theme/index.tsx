import { RsfamilyNavIcon } from 'rsfamily-nav-icon';
import Theme from 'rspress/theme';
import { HomeLayout } from './pages';
import 'rsfamily-nav-icon/dist/index.css';
import { NoSSR, useLang } from 'rspress/runtime';
import { Announcement } from './components/Announcement';

const ANNOUNCEMENT_EN_URL = 'https://rspack.dev/blog/announcing-1-0';
const ANNOUNCEMENT_ZH_URL = 'https://rspack.dev/zh/blog/announcing-1-0';

const ReleaseV1Announcement = () => {
  const lang = useLang();

  return (
    <NoSSR>
      <Announcement
        href={lang === 'en' ? ANNOUNCEMENT_EN_URL : ANNOUNCEMENT_ZH_URL}
        message={
          lang === 'en'
            ? 'Rspack 1.0 has been released!'
            : 'Rspack 1.0 正式发布！'
        }
        // always display announcement to recommend upgrade
        display={true}
      />
    </NoSSR>
  );
};

const Layout = () => (
  <Theme.Layout
    beforeNavTitle={<RsfamilyNavIcon />}
    beforeNav={<ReleaseV1Announcement />}
  />
);

export * from 'rspress/theme';

export default {
  ...Theme,
  Layout,
  HomeLayout,
};
