import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
  },
  optimization: {
    sideEffects: true,
    moduleIds: 'named',
    concatenateModules: false,
  },
  plugins: [
    new rspack.DefinePlugin({
      'process.env.NODE_ENV': JSON.stringify('production'),
    }),
  ],
});
