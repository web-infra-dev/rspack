import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig([
  {
    module: {
      rules: [
        {
          test: /lib\.js$/,
          use: [
            { loader: './simple-loader.mjs', parallel: true, options: {} },
            { loader: './pitching-loader.mjs', parallel: false, options: {} },
            {
              loader: './simple-async-loader.mjs',
              parallel: true,
              options: {},
            },
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
            {
              loader: 'builtin:test-simple-loader',
              parallel: false,
              options: {},
            },
            { loader: './pitching-loader.mjs', parallel: true, options: {} },
            {
              loader: './simple-async-loader.mjs',
              parallel: true,
              options: {},
            },
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
            { loader: './simple-loader.mjs', parallel: true, options: {} },
            {
              loader: 'builtin:test-pitching-loader',
              parallel: false,
              options: {},
            },
            {
              loader: './simple-async-loader.mjs',
              parallel: true,
              options: {},
            },
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
]);
