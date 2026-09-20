import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
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
});
