import path from 'node:path';
import { rspack as webpack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: ['./a', './b', './_d', './_e', './f', './g.abc', './h'],
  resolve: {
    extensions: ['.js', '.jsx'],
  },
  output: {
    filename: 'dll.js',
    chunkFilename: '[id].dll.js',
    library: { type: 'commonjs2' },
  },
  module: {
    rules: [
      {
        test: /\.abc\.js$/,
        loader: './g-loader.js',
        options: {
          test: 1,
        },
      },
      {
        test: /0-create-dll.h/,
        sideEffects: false,
      },
    ],
  },
  optimization: {
    usedExports: true,
    sideEffects: true,
  },
  plugins: [
    new webpack.DllPlugin({
      path: path.resolve(
        import.meta.dirname,
        '../../../js/config/dll-plugin/manifest0.json',
      ),
      entryOnly: false,
    }),
  ],
};
