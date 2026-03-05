'use server-entry';

import { renderToReadableStream } from 'react-server-dom-rspack/server.node';
import {
  SharedClientComponent,
  sharedAction,
  sharedValue,
} from 'rsbuild-rsc-federation-shared';
import ExposedButton from '../exposed-client';
import { consumeRemoteAndShared } from '../rsc-consumer';
import { renderSsrShell } from './entry.ssr';

void renderToReadableStream;

export async function rscRenderSummary() {
  const consumed = await consumeRemoteAndShared();
  await sharedAction();
  return `${consumed}:${sharedValue}:${typeof SharedClientComponent}`;
}

export default async function RscRoot() {
  const summary = await rscRenderSummary();
  const ssrShell = renderSsrShell();

  return (
    <main data-summary={summary} data-ssr-length={ssrShell.length}>
      <ExposedButton />
      <p>{summary}</p>
    </main>
  );
}
