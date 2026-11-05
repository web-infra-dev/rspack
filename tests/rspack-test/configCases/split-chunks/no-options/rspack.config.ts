import { defineConfig } from '@rspack/cli';
import { optimize } from '@rspack/core';

const { SplitChunksPlugin } = optimize;

export default defineConfig({
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
  // @ts-expect-error Rspack does not support resolver plugins yet.
  plugins: [new SplitChunksPlugin()],
});
