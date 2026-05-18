'use server-entry';

import { PageOneClient } from '../clients/PageOneClient';
import { SharedAcrossPages } from '../clients/SharedAcrossPages';
import { SharedRootAndPage } from '../clients/SharedRootAndPage';
import './PageOne.css';

export const PageOne = () => {
  return (
    <section className="page-one">
      <PageOneClient />
      <SharedAcrossPages />
      <SharedRootAndPage />
    </section>
  );
};
