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
            './simple-loader.mjs',
            './pitching-loader.mjs',
            './simple-async-loader.mjs',
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
            './pitching-loader.mjs',
            './simple-async-loader.mjs',
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
            './simple-loader.mjs',
            'builtin:test-pitching-loader',
            './simple-async-loader.mjs',
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
            './simple-loader.mjs',
            './pitching-loader.mjs',
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
