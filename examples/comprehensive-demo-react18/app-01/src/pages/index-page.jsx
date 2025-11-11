import Markdown from '../Markdown';
import Page from '../Page';
import React from 'react';
import Welcome from '../docs/Welcome.md';

const IndexPage = () => (
  <Page title="Module Federation Demo">
    <Markdown>{Welcome}</Markdown>
  </Page>
);

export default IndexPage;
