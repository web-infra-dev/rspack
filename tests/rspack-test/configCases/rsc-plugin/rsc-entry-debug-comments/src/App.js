import { RootClient } from './clients/RootClient';
import { SharedRootAndPage } from './clients/SharedRootAndPage';
import { PageTwo } from './pages/PageTwo';
import { PageOne } from './pages/PageOne';

export const App = () => {
  return (
    <>
      <RootClient />
      <SharedRootAndPage />
      <PageTwo />
      <PageOne />
    </>
  );
};
