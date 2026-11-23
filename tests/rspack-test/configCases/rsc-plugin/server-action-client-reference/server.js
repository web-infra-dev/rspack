import { ReadableStream } from 'node:stream/web';
import { createElement } from 'react';
import { loadServerAction, renderToReadableStream } from 'react-server-dom-rspack/server.node';
import { Outer } from './outer.js';

export const page = createElement(Outer);

it('serializes a client component reached only through a client-imported action', async () => {
  globalThis.ReadableStream ??= ReadableStream;
  const { clientManifest, serverManifest } = __rspack_rsc_manifest__;
  const [actionId] = Object.keys(serverManifest);
  const action = loadServerAction(actionId);
  const errors = [];
  const stream = renderToReadableStream(await action(), {
    onError(error) {
      errors.push(error.message);
    },
  });
  const reader = stream.getReader();
  while (!(await reader.read()).done) {
    // Drain the Flight response.
  }
  expect(errors).toEqual([]);
  expect(Object.keys(clientManifest).some((file) => file.endsWith('inner.js'))).toBe(true);
});
