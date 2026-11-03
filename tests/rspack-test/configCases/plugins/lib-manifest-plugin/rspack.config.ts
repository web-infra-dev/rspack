import { defineConfig } from '@rspack/cli';
import path from 'node:path';
// @ts-expect-error Rspack does not expose LibManifestPlugin yet.
import { LibManifestPlugin } from '@rspack/core';

export default defineConfig((_env, { testPath }) => ({
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
}));
