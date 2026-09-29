import { defineConfig } from '@rspack/cli';

export default defineConfig({
  mode: 'development',
  devtool: false,
  target: 'node',
  output: {
    module: true,
    chunkFormat: 'module',
  },
  module: {
    rules: [
      {
        test: /empty-options\.js$/,
        parser: {
          importMeta: {},
        },
      },
      {
        test: /disabled-fields\.js$/,
        parser: {
          importMeta: {
            dirname: false,
            filename: false,
            main: false,
            url: false,
            webpack: false,
            webpackContext: false,
            glob: false,
            webpackHot: false,
          },
        },
      },
    ],
  },
});
