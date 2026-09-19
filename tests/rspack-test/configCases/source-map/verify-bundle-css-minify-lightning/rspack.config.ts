import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

export default defineConfig({
  target: 'web',
  node: false,
  module: {
    generator: {
      'css/auto': {
        exportsOnly: false,
      },
    },
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
  devtool: 'source-map',
  optimization: {
    minimize: true,
    minimizer: [new rspack.LightningCssMinimizerRspackPlugin()],
  },
  externals: [
    {
      fs: 'node-commonjs fs',
      path: 'node-commonjs path',
    },
    {
      '@rspack/test-tools/helper/util/checkSourceMap':
        'commonjs @rspack/test-tools/helper/util/checkSourceMap',
    },
    'source-map',
  ],
  externalsType: 'commonjs',
});
