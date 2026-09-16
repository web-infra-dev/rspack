import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  output: {
    library: { type: 'commonjs2' },
  },
  externals: {
    external: ['webpack', 'version'],
  },
  plugins: [
    new rspack.DefinePlugin({
      NODE_VERSION: JSON.stringify(process.version),
      EXPECTED: JSON.stringify(rspack.version),
    }),
  ],
};
