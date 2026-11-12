import { defineConfig } from '@rspack/cli';
import { sharing } from '@rspack/core';

const { ProvideSharedPlugin } = sharing;

export default defineConfig({
  plugins: [
    new ProvideSharedPlugin({
      provides: ['package'],
    }),
  ],
});
