import { NavIcon } from '@rstack-dev/doc-ui/nav-icon';
import { NoSSR } from 'rspress/runtime';
import Theme from 'rspress/theme';
// enable this when we need a new announcement
import { Announcement } from './components/Announcement';
import { HomeLayout } from './pages';

const Layout = () => (
  <Theme.Layout
    beforeNavTitle={<NavIcon />}
    beforeNav={
      <NoSSR>
        <Announcement />
      </NoSSR>
    }
  />
);

export * from 'rspress/theme';

export default {
  ...Theme,
  Layout,
  HomeLayout,
};
