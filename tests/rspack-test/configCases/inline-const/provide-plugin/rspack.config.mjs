import { ProvidePlugin } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  externals: {
    fs: 'node-commonjs fs',
  },
  optimization: {
    moduleIds: 'named',
    inlineExports: true,
  },
  plugins: [
    new ProvidePlugin({
      providedA: ['./constants.js', 'a'],
      providedDefault: ['./constants.js', 'default'],
    }),
  ],
};
