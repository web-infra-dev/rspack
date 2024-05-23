import Theme from 'rspress/theme';
import { HomeLayout } from './pages';
import { RsfamilyNavIcon } from 'rsfamily-nav-icon';
import 'rsfamily-nav-icon/dist/index.css';
// enable this when we need a new announcement
// import { Announcement } from './components/Announcement';
// import { NoSSR } from 'rspress/runtime';

const Layout = () => (
  <Theme.Layout
    beforeNavTitle={<RsfamilyNavIcon />}
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
