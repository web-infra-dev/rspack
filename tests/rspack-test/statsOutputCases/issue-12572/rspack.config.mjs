import { rspack as webpack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'production',
  entry: './index',
  plugins: [
    new webpack.optimize.LimitChunkCountPlugin({
      maxChunks: 1,
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
};
