import { defineConfig } from '@rspack/cli';

import TerserPlugin from 'terser-webpack-plugin';

export default defineConfig({
  mode: 'production',
  entry: './index',
  output: {
    filename: 'bundle.js',
  },
  optimization: {
    minimize: true,
    minimizer: [
      new TerserPlugin({
        terserOptions: {
          mangle: false,
          output: {
            beautify: true,
            comments: false,
          },
        },
      }),
    ],
  },
  stats: {
    assets: true,
    chunkModules: false,
    modules: true,
    providedExports: true,
    usedExports: true,
  },
});
