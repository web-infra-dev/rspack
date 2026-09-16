import { fileURLToPath } from 'node:url';

/** @type {import("@rspack/core").Configuration} */
export default {
  target: 'web',
  output: {
    assetModuleFilename: '[name][ext]',
  },
  module: {
    rules: [
      {
        test: /\.txt$/,
        loader: fileURLToPath(import.meta.resolve('./loader.js')),
      },
      {
        test: /\.text$/,
        type: 'asset/source',
      },
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
};
