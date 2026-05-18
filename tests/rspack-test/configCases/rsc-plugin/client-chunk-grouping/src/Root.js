import { RootOnlyA } from './clients/RootOnlyA';
import { RootOnlyB } from './clients/RootOnlyB';
import { SharedRootAndPage } from './clients/SharedRootAndPage';
import { PageOne } from './pages/PageOne';
import { PageTwo } from './pages/PageTwo';

export const Root = async () => {
  return (
    <main>
      <RootOnlyA />
      <RootOnlyB />
      <SharedRootAndPage />
      <PageOne />
      <PageTwo />
    </main>
  );
};
