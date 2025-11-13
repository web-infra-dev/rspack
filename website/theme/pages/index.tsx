import { HomeBackground } from '@rspress/core/theme';
import { HomeFooter } from '../components/HomeFooter/index';
import LandingPage from '../components/Landingpage';

const CopyRight = () => {
  return (
    <footer className="bottom-0 mt-12 py-8 px-6 sm:p-8 w-full border-t border-solid rp-border-divider-light">
      <div className="m-auto w-full text-center">
        <div className="font-medium text-sm text-text-2">
          <p className="mb-2">
            Rspack is free and open source software released under the MIT
            license.
          </p>
          <p>© 2022-present ByteDance Inc.</p>
        </div>
      </div>
    </footer>
  );
};

export function HomeLayout() {
  return (
    <>
      {/* For transparent nav at top */}
      <HomeBackground style={{ background: 'none' }} />
      <LandingPage />
      <HomeFooter />
      <CopyRight />
    </>
  );
}
