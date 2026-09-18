import { fileURLToPath } from 'node:url';

/** @type {import('webpack').Configuration} */
const config = {
  devtool: 'source-map',
  module: {
    rules: [
      {
        test: /\.js$/,
        use: [
          {
            loader: fileURLToPath(import.meta.resolve('./loader.mjs')),
          },
        ],
      },
    ],
  },
};

export default config;
