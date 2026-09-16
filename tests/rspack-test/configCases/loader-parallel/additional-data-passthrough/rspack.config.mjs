import path from 'node:path';

/**
 * @type {import('@rspack/core').RspackOptions}
 */
export default {
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: path.join(import.meta.dirname, 'a.js'),
        use: [
          { loader: './loader-2.js', parallel: true, options: {} },
          {
            loader: 'builtin:test-passthrough-loader',
            parallel: true,
            options: {},
          },
          { loader: './loader-1.js', parallel: true, options: {} },
        ],
      },
    ],
  },
};
