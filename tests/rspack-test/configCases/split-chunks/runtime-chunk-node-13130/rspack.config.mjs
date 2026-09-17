import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    'deep/path/a': './a',
    b: ['./shared?1', './shared?2', './b'],
    'somewhere/c': './c',
  },
  target: 'node',
  output: {
    filename: '[name].js',
  },
  optimization: {
    chunkIds: 'named',
    runtimeChunk: 'single',
    splitChunks: {
      cacheGroups: {
        dep: {
          chunks: 'initial',
          minChunks: 2,
          test: path.resolve(import.meta.dirname, 'shared.js'),
          enforce: true,
        },
      },
    },
  },
};
