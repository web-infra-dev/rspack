import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

const {
  experiments: { VirtualModulesPlugin },
} = rspack;

export default defineConfig({
  entry: {
    main: './index.js',
  },
  output: {
    filename: '[name].js',
  },
  plugins: [
    new VirtualModulesPlugin({
      'foo.js': 'export const foo = "foo"',
    }),
    new VirtualModulesPlugin({
      'bar.js': 'export const bar = "bar"',
    }),
  ],
});
