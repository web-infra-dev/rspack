import path from 'node:path';
import { rspack as webpack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    new webpack.ContextReplacementPlugin(
      /context-replacement/,
      path.resolve(import.meta.dirname, 'modules'),
      {
        a: './module-b',
      },
    ),
  ],
};
