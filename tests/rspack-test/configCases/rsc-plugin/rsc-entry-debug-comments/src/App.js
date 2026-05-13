import { RootClient } from './clients/RootClient';
import { SharedRootAndPage } from './clients/SharedRootAndPage';
import { PageOne } from './pages/PageOne';
import { PageTwo } from './pages/PageTwo';

export const App = () => {
  return (
    <>
      <RootClient />
      <SharedRootAndPage />
      <PageOne />
      <PageTwo />
    </>
  );
};
