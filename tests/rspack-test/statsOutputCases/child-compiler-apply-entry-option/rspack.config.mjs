import TestApplyEntryOptionPlugin from './TestApplyEntryOptionPlugin.js';

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    parent: './parent',
  },
  output: {
    filename: '[name].js',
  },
  plugins: [
    new TestApplyEntryOptionPlugin({
      entry: {
        child: './child',
      },
    }),
  ],
  stats: {
    assets: true,
    modules: true,
    children: true,
    entrypoints: true,
  },
};
