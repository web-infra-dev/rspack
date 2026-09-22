import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'async-node',
  optimization: {
    concatenateModules: false,
  },
  experiments: {
    deferImport: true,
    sourceImport: true,
  },
  module: {
    rules: [
      {
        test: /module\.js$/,
        phase: 'defer',
        loader: './phase-loader.mjs',
        options: {
          phase: 'defer',
        },
      },
      {
        test: /module\.js$/,
        phase: 'source',
        loader: './phase-loader.mjs',
        options: {
          phase: 'source',
        },
      },
      {
        test: /module\.js$/,
        phase: 'evaluation',
        loader: './phase-loader.mjs',
        options: {
          phase: 'evaluation',
        },
      },
    ],
  },
});
