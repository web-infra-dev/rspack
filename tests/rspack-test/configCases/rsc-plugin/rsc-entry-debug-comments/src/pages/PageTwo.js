'use server-entry';

import { PageTwoClient } from '../clients/PageTwoClient';
import { SharedAcrossPages } from '../clients/SharedAcrossPages';
import './PageTwo.css';

export const PageTwo = () => {
  return (
    <section className="page-two">
      <PageTwoClient />
      <SharedAcrossPages />
    </section>
  );
};
