import { defineConfig } from '@rspack/cli';
import { fileURLToPath } from 'node:url';

export default defineConfig({
  module: {
    rules: [
      {
        oneOf: [
          {
            test: {
              and: [/a.\.js$/, /b\.js$/, { not: /not-/ }],
            },
            resourceQuery: { not: /not/ },
            loader: './loader.mjs',
            options: 'first',
          },
          {
            test: [
              fileURLToPath(import.meta.resolve('./a.js')),
              fileURLToPath(import.meta.resolve('./c.js')),
            ],
            issuer: fileURLToPath(import.meta.resolve('./b.js')),
            use: [
              './loader.mjs',
              {
                loader: './loader.mjs',
                options: 'second-2',
              },
              {
                loader: './loader.mjs',
                options: {
                  get: function () {
                    return 'second-3';
                  },
                },
              },
            ],
          },
          {
            test: {
              or: [
                fileURLToPath(import.meta.resolve('./a.js')),
                fileURLToPath(import.meta.resolve('./c.js')),
              ],
            },
            loader: './loader.mjs',
            options: 'third',
          },
        ],
      },
    ],
  },
});
