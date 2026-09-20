import { defineConfig } from '@rspack/cli';
import { sharing } from '@rspack/core';

const { ProvideSharedPlugin } = sharing;

export default defineConfig({
  output: {
    filename: '[name].js',
  },
  optimization: {
    runtimeChunk: 'single',
  },
  plugins: [
    new ProvideSharedPlugin({
      provides: ['x'],
    }),
  ],
});
