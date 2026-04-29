"use server-entry";

import { PageOneClientA } from '../clients/PageOneClientA';
import { PageOneClientB } from '../clients/PageOneClientB';
import { SharedAcrossPages } from '../clients/SharedAcrossPages';
import { SharedRootAndPage } from '../clients/SharedRootAndPage';
import './PageOne.css';

export const PageOne = async () => {
  return (
    <section className="page-one-server-css">
      <PageOneClientA />
      <PageOneClientB />
      <SharedAcrossPages />
      <SharedRootAndPage />
    </section>
  );
};
