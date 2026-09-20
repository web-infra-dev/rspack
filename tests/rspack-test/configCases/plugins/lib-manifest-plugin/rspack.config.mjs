import path from 'node:path';
import { LibManifestPlugin } from '@rspack/core';

/** @type {function(any, any): import("@rspack/core").Configuration} */
export default (env, { testPath }) => ({
  entry: {
    bundle0: ['./'],
  },
  plugins: [
    new LibManifestPlugin({
      path: path.resolve(testPath, '[name]-manifest.json'),
      name: '[name]_[fullhash]',
    }),
  ],
  node: {
    __dirname: false,
  },
});
