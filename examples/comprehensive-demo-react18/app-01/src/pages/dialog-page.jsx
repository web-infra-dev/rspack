import DialogMarkdown from '../docs/Dialog.md';
import Markdown from 'markdown-to-jsx';
import Page from '../Page';
import React from 'react';

const Dialog = React.lazy(() => import('app_02/Dialog'));

const DialogPage = () => (
  <Page title="Dialog Demo">
    <Markdown>{DialogMarkdown}</Markdown>
    <React.Suspense fallback="Loading Dialog...">
      <Dialog />
    </React.Suspense>
  </Page>
);

export default DialogPage;
