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
      'foo.js': 'export const foo = "foo"',
      'bar.js': 'export const bar = "bar"',
    }),
  ],
};
