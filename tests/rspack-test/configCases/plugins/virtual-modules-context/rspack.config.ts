import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

const {
  experiments: { VirtualModulesPlugin },
} = rspack;

export default defineConfig({
  // Virtual directory timestamps change when each compiler recreates its files.
  snapshot: { contextModule: { hash: true } },
  entry: {
    main: './index.js',
  },
  output: {
    filename: '[name].js',
  },
  plugins: [
    new VirtualModulesPlugin({
      'translations/en.js': 'export const hello = "hello"',
      'translations/zh.js': 'export const hello = "你好"',
    }),
  ],
});
