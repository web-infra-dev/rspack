import { usePageData } from 'rspress/runtime';
import { HomeFooter } from '../components/HomeFooter/index';
import LandingPage from '../components/Landingpage';

const CopyRight = () => {
  const { siteData } = usePageData();
  const { message } = siteData.themeConfig.footer || {};

  if (!message) {
    return null;
  }

  return (
    <footer className="bottom-0 mt-12 py-8 px-6 sm:p-8 w-full border-t border-solid border-divider-light">
      <div className="m-auto w-full text-center">
        <div className="font-medium text-sm text-text-2">{message}</div>
      </div>
    </footer>
  );
};

export function HomeLayout() {
  return (
    <>
      <LandingPage />
      <HomeFooter />
      <CopyRight />
    </>
  );
}
