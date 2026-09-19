import { defineConfig } from '@rspack/cli';

import { DefinePlugin } from '@rspack/core';

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
    '@rspack/test-tools/helper/util/checkSourceMap':
      'commonjs @rspack/test-tools/helper/util/checkSourceMap',
  },
  mode: 'development',
  devtool: 'source-map',
  resolve: {
    extensions: ['...', '.jsx'],
  },
  module: {
    rules: [
      {
        test: /a\.jsx$/,
        use: [
          {
            loader: 'builtin:swc-loader',
            options: {
              sourceMaps: true,
            },
          },
          './prev-loader.mjs',
        ],
      },
    ],
  },
  plugins: [
    new DefinePlugin({
      CONTEXT: JSON.stringify(import.meta.dirname),
    }),
  ],
});
