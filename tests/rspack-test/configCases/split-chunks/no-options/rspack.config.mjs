import { optimize } from '@rspack/core';

const { SplitChunksPlugin } = optimize;

/** @type {import("@rspack/core").Configuration} */
export default {
  entry: {
    vendor: ['./a'],
    main: './index',
  },
  target: 'web',
  output: {
    filename: '[name].js',
  },
  optimization: {
    splitChunks: false,
  },
  plugins: [new SplitChunksPlugin()],
};
