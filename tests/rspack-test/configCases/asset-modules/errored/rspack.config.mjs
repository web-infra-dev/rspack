import { fileURLToPath } from 'node:url';

/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  output: {
    hashDigestLength: 8,
  },
  module: {
    rules: [
      {
        test: /\.css$/i,
        type: 'asset/resource',
        generator: {
          filename: () => 'css/style.[contenthash].css',
        },
        use: [
          {
            loader: fileURLToPath(import.meta.resolve('./loader.js')),
          },
        ],
      },
    ],
  },
};
