import path from 'node:path';

/** @type {import("@rspack/core").Configuration} */
export default {
  context: path.join(import.meta.dirname, '../external'),
  entry: '../external-in-node/index.js',
  target: 'node',
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
};
