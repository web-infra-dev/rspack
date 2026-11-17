import path from 'node:path';

export default {
  resolve: {
    alias: {
      'strict-target': path.join(import.meta.dirname, 'target.custom'),
      'strict-ignored': false,
    },
    byDependency: {
      esm: {
        alias: {
          'strict-esm': path.join(import.meta.dirname, 'target.custom'),
        },
      },
    },
  },
  module: {
    rules: [
      {
        test: /value\.js$/,
        use: [{ loader: './loader.js', parallel: 'strict', options: {} }],
      },
    ],
  },
};
