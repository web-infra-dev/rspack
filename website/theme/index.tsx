import { NavIcon } from '@rstack-dev/doc-ui/nav-icon';
import Theme from 'rspress/theme';
import { HomeLayout } from './pages';
// enable this when we need a new announcement
// import { Announcement } from './components/Announcement';
// import { NoSSR } from 'rspress/runtime';

const Layout = () => (
  <Theme.Layout
    beforeNavTitle={<NavIcon />}
    // beforeNav={
    //   <NoSSR>
    //     <Announcement />
    //   </NoSSR>
    // }
  />
);

export * from 'rspress/theme';

export default {
  ...Theme,
  Layout,
  HomeLayout,
};
