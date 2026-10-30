import type { RuleSetUseItem } from '@rspack/core';
import { defineConfig } from '@rspack/cli';
import { fileURLToPath } from 'node:url';

function createFunctionArrayFromUseArray(useArray: RuleSetUseItem[]) {
  return useArray.map(function (useItem) {
    return function () {
      return useItem;
    };
  });
}

var useArray = createFunctionArrayFromUseArray([
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
]);

export default defineConfig({
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
            // @ts-expect-error Rspack does not support functions inside a loader array yet.
            use: useArray,
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
});
