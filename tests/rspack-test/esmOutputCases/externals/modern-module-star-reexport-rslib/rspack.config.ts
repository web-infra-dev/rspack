import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

const {
  experiments: { RslibPlugin },
} = rspack;

export default defineConfig({
  externalsType: 'modern-module',
  externals: {
    externals: 'externals',
  },
  plugins: [new RslibPlugin()],
});
