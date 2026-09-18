import path from 'node:path';
export default {
  module: {
    rules: [
      {
        test: path.resolve(import.meta.dirname, 'index.js'),
        loader: './loader.mjs',
      },
    ],
  },
};
