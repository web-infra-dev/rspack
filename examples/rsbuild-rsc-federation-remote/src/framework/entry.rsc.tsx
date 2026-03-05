'use server-entry';

import { renderToReadableStream } from 'react-server-dom-rspack/server.node';
import {
  MixedClientBadge,
  SharedClientComponent,
  sharedValue,
} from 'rsbuild-rsc-federation-shared';
import { sharedAction } from 'rsbuild-rsc-federation-shared/server-actions';
import { consumeRemoteAndShared } from '../rsc-consumer';
import { consumeMixedPatterns } from '../server-mixed-consumer';
import { renderSsrShell } from './entry.ssr';

void renderToReadableStream;

export async function rscRenderSummary() {
  const consumed = await consumeRemoteAndShared();
  const mixedConsumed = await consumeMixedPatterns();
  await sharedAction();
  return `${consumed}:${mixedConsumed}:${sharedValue}:${typeof SharedClientComponent}:${typeof MixedClientBadge}`;
}

export default async function RscRoot() {
  const summary = await rscRenderSummary();
  const ssrShell = renderSsrShell();

  return (
    <main data-summary={summary} data-ssr-length={ssrShell.length}>
      <p>{summary}</p>
    </main>
  );
}
