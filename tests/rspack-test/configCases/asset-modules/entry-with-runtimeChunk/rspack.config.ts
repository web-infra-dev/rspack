import fs from 'node:fs';
import path from 'node:path';
import { defineConfig, definePlugin } from '@rspack/cli';
import { type Configuration, rspack } from '@rspack/core';

const common = defineConfig({
  module: {
    rules: [
      {
        test: /\.png$/,
        type: 'asset',
      },
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
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
  optimization: {
    runtimeChunk: {
      name: (entrypoint) => `runtime~${entrypoint.name}`,
    },
  },
});

const entry = (i: number): Configuration | undefined => {
  switch (i % 4) {
    case 0:
      return {
        entry: {
          app: {
            import: '../_images/file.png',
          },
        },
      };
    case 1:
      return {
        entry: {
          app: ['../_images/file.png', './entry.js'],
        },
      };
    case 2:
      return {
        entry: {
          app: ['../_images/file.png', './entry.css'],
        },
      };
    case 3:
      return {
        entry: {
          entry1: '../_images/file.png',
          entry2: './entry.js',
        },
      };
    default:
      break;
  }
};

const esm = (i: number) =>
  defineConfig({
    ...common,
    ...entry(i),
    output: {
      filename: `${i}/[name].mjs`,
      chunkFilename: `${i}/[name].mjs`,
      cssFilename: `${i}/[name].css`,
      cssChunkFilename: `${i}/[name].css`,
      assetModuleFilename: `${i}/[name][ext][query]`,
      module: true,
    },
  });

const node = (i: number) =>
  defineConfig({
    ...common,
    ...entry(i),
    output: {
      filename: `${i}/[name].js`,
      chunkFilename: `${i}/[name].js`,
      cssFilename: `${i}/[name].css`,
      cssChunkFilename: `${i}/[name].css`,
      assetModuleFilename: `${i}/[name][ext][query]`,
    },
    target: 'node',
  });

const web = (i: number) =>
  defineConfig({
    ...common,
    ...entry(i),
    output: {
      filename: `${i}/[name].js`,
      chunkFilename: `${i}/[name].js`,
      cssFilename: `${i}/[name].css`,
      cssChunkFilename: `${i}/[name].css`,
      assetModuleFilename: `${i}/[name][ext][query]`,
    },
    target: 'web',
  });

export default defineConfig([
  // web
  ...[0, 1, 2, 3].map((i) => web(i)),
  // node
  ...[4, 5, 6, 7].map((i) => node(i)),
  // ESM
  ...[8, 9, 10, 11].map((i) => esm(i)),
]);
