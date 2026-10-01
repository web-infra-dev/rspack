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
        test: /context-field-enabled\.js$/,
        parser: {
          importMeta: {
            webpackContext: true,
          },
        },
      },
      {
        test: /context-field-disabled\.js$/,
        parser: {
          importMeta: {
            webpackContext: false,
          },
        },
      },
    ],
  },
});
