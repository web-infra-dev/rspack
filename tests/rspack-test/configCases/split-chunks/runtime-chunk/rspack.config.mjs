import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  externals: {
    './dep-shared_js.js': 'commonjs ./dep-shared_js.js',
  },
  entry: {
    a: './a',
    b: './b',
  },
  target: 'web',
  output: {
    filename: '[name].js',
  },
  optimization: {
    chunkIds: 'named',
    runtimeChunk: 'single',
    splitChunks: {
      cacheGroups: {
        dep: {
          chunks: 'all',
          test: path.resolve(import.meta.dirname, 'shared.js'),
          enforce: true,
        },
      },
    },
  },
};
