import fs from 'node:fs';
import path from 'node:path';
import { rspack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration[]} */
export default [
  {
    optimization: {
      minimize: false,
    },
    module: {
      rules: [
        {
          test: /index.js$/,
          type: 'javascript/dynamic',
        },
        {
          test: /esm/,
          type: 'javascript/esm',
        },
      ],
    },
    plugins: [
      {
        apply(compiler) {
          compiler.hooks.compilation.tap('Test', (compilation) => {
            compilation.hooks.processAssets.tap(
              {
                name: 'copy-webpack-plugin',
                stage:
                  compiler.rspack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONAL,
              },
              () => {
                const data = fs.readFileSync(
                  path.resolve(import.meta.dirname, './test.js'),
                );

                compilation.emitAsset(
                  'test.js',
                  new rspack.sources.RawSource(data),
                );
              },
            );
          });
        },
      },
    ],
  },
];
