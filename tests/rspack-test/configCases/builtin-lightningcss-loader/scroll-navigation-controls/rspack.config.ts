import { defineConfig } from '@rspack/cli';
import { type LightningcssLoaderOptions, rspack } from '@rspack/core';

const targets = { chrome: '150' };
const drafts = { scrollNavigationControls: true };

export default defineConfig({
  target: 'web',
  externalsPresets: { node: true },
  node: false,
  module: {
    rules: [
      {
        test: /\.css$/,
        oneOf: [
          {
            resourceQuery: /loader/,
            type: 'asset/source',
            use: {
              loader: 'builtin:lightningcss-loader',
              options: {
                targets,
                drafts,
                minify: true,
              } satisfies LightningcssLoaderOptions,
            },
          },
          {
            type: 'css',
          },
        ],
      },
    ],
  },
  optimization: {
    minimize: true,
    minimizer: [
      new rspack.LightningCssMinimizerRspackPlugin({
        minimizerOptions: { targets, drafts },
      }),
    ],
  },
});
