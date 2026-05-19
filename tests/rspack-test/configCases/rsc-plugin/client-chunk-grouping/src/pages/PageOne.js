"use server-entry";

import { PageOneClientA } from '../clients/PageOneClientA';
import { PageOneClientB } from '../clients/PageOneClientB';
import { SharedAcrossPages } from '../clients/SharedAcrossPages';
import { SharedRootAndPage } from '../clients/SharedRootAndPage';
import { SharedServerChild } from '../server/SharedServerChild';
import './PageOne.css';

export const PageOne = async () => {
  const { PageOneDynamicClient } = await import('../clients/PageOneDynamicClient');
  const { SharedServerChild: DynamicSharedServerChild } = await import(
    '../server/SharedServerChild'
  );

  return (
    <section className="page-one-server-css">
      <PageOneClientA />
      <PageOneClientB />
      <PageOneDynamicClient />
      <SharedServerChild />
      <DynamicSharedServerChild />
      <SharedAcrossPages />
      <SharedRootAndPage />
    </section>
  );
};
