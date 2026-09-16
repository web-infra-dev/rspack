import { rspack } from '@rspack/core';

/**
 * @type {import('@rspack/core').RspackOptions}
 */
export default [
  {
    module: {
      rules: [
        {
          test: /lib\.js$/,
          use: [
            './simple-loader.js',
            './pitching-loader.js',
            './simple-async-loader.js',
          ],
        },
      ],
    },
    plugins: [
      new rspack.DefinePlugin({
        CONTEXT: JSON.stringify(import.meta.dirname),
      }),
    ],
  },
  {
    module: {
      rules: [
        {
          test: /lib\.js$/,
          use: [
            'builtin:test-simple-loader',
            './pitching-loader.js',
            './simple-async-loader.js',
          ],
        },
      ],
    },
    plugins: [
      new rspack.DefinePlugin({
        CONTEXT: JSON.stringify(import.meta.dirname),
      }),
    ],
  },
  {
    module: {
      rules: [
        {
          test: /lib\.js$/,
          use: [
            './simple-loader.js',
            'builtin:test-pitching-loader',
            './simple-async-loader.js',
          ],
        },
      ],
    },
    plugins: [
      new rspack.DefinePlugin({
        CONTEXT: JSON.stringify(import.meta.dirname),
      }),
    ],
  },
  {
    module: {
      rules: [
        {
          test: /lib\.js$/,
          use: [
            './simple-loader.js',
            './pitching-loader.js',
            'builtin:test-simple-async-loader.js',
          ],
        },
      ],
    },
    plugins: [
      new rspack.DefinePlugin({
        CONTEXT: JSON.stringify(import.meta.dirname),
      }),
    ],
  },
];
