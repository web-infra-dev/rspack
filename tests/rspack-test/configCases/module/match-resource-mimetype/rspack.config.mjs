import path from 'node:path';

/**
 * @type {import('@rspack/core').RspackOptions}
 */
export default {
  module: {
    rules: [
      {
        include: path.resolve(import.meta.dirname, 'a.js'),
        use: [
          './get-source.js',
          {
            loader: 'builtin:swc-loader',
            options: {
              jsc: {
                target: 'es3',
              },
            },
          },
        ],
      },
    ],
  },
};
