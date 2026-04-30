"use server-entry";

import { PageTwoClient } from '../clients/PageTwoClient';
import { SharedAcrossPages } from '../clients/SharedAcrossPages';
import './PageTwo.css';

export const PageTwo = async () => {
  return (
    <section className="page-two-server-css">
      <PageTwoClient />
      <SharedAcrossPages />
    </section>
  );
};
