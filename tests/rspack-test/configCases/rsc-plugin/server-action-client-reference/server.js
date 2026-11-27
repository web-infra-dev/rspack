import path from 'node:path';
import { ReadableStream } from 'node:stream/web';
import { createElement } from 'react';
import {
  loadServerAction,
  renderToReadableStream,
} from 'react-server-dom-rspack/server.node';
import { Outer } from './outer.js';

export const page = createElement(Outer);

it('follows successive client and action imports and terminates on a cycle', async () => {
  globalThis.ReadableStream ??= ReadableStream;
  const { clientManifest, serverManifest } = __rspack_rsc_manifest__;
  const actionIds = Object.keys(serverManifest);
  const steps = [];
  for (const id of actionIds) {
    const result = await loadServerAction(id)();
    steps.push(result.step);
    const errors = [];
    const stream = renderToReadableStream(result.element, {
      onError(error) {
        errors.push(error.message);
      },
    });
    const reader = stream.getReader();
    while (!(await reader.read()).done) {
      // Drain the Flight response.
    }
    expect(errors).toEqual([]);
  }
  expect(steps.sort()).toEqual(['inner', 'leaf', 'middle', 'outer']);
  expect(
    Object.keys(clientManifest).map((file) => path.basename(file)).sort(),
  ).toEqual(['inner.js', 'leaf.js', 'middle.js', 'outer.js']);
});
