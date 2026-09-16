import { rspack } from '@rspack/core';

const {
  experiments: { VirtualModulesPlugin },
} = rspack;

/** @type {import("@rspack/core").Configuration} */
export default {
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
};
