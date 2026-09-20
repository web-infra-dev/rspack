import { defineConfig } from '@rspack/cli';
import { rspack as webpack } from '@rspack/core';

export default defineConfig(
  [1, 2, 3, 4].map((n) => ({
    name: `${n} chunks`,
    mode: 'production',
    entry: './index',
    output: {
      filename: `bundle${n}.js`,
    },
    plugins: [
      new webpack.optimize.LimitChunkCountPlugin({
        maxChunks: n,
      }),
    ],
    stats: {
      assets: true,
      chunkModules: true,
      dependentModules: true,
      chunkRelations: true,
      modules: false,
      chunks: true,
    },
  })),
);
