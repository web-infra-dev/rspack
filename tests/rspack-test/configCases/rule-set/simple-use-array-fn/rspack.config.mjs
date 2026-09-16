import { fileURLToPath } from 'node:url';

/** @type {import("@rspack/core").Configuration} */
export default {
  module: {
    rules: [
      {
        oneOf: [
          {
            test: {
              and: [/a.\.js$/, /b\.js$/],
            },
            loader: './loader',
            options: 'first',
          },
          {
            test: [
              fileURLToPath(import.meta.resolve('./a.js')),
              fileURLToPath(import.meta.resolve('./c.js')),
            ],
            issuer: fileURLToPath(import.meta.resolve('./b.js')),
            use: (data) => [
              './loader',
              {
                loader: './loader',
                options: 'second-2',
              },
              {
                loader: './loader',
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
            loader: './loader',
            options: 'third',
          },
        ],
      },
    ],
  },
};
