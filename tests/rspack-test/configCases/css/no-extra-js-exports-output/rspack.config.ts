import fs from 'node:fs';
import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';
import { rspack } from '@rspack/core';

/**
 * @param i index
 * @returns entry
 */
const entry = (i: number) => {
  switch (i) {
    case 0:
      return {
        main: ['./main.css'],
      };
    case 1:
      return {
        main: ['./main1.js'],
      };
    case 2:
      return {
        main: ['./main2.js'],
      };
  }
};

/**
 * @param i param
 * @returns return
 */
const common = (i: number) =>
  defineConfig({
    entry: {
      ...entry(i),
    },
    target: 'web',
    devtool: false,
    module: {
      rules: [
        {
          test: /\.css$/,
          type: 'css/auto',
        },
      ],
    },
    output: {
      filename: `${i}/[name].js`,
      chunkFilename: `${i}/[name].js`,
      cssFilename: `${i}/[name].css`,
      cssChunkFilename: `${i}/[name].css`,
    },
    plugins: [
      definePlugin({
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
      }),
    ],
  });

export default defineConfig([0, 1].map((i: number) => common(i)));
