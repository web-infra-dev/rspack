import { defineConfig } from '@rspack/cli';
import type { Configuration } from '@rspack/core';

const config = (entry: string): Configuration => ({
  entry,
  mode: 'development',
  devtool: false,
  experiments: {
    deferImport: true,
  },
  optimization: {
    sideEffects: true,
    concatenateModules: false,
  },
});

export default defineConfig([config('./index.js'), config('./early.js')]);
