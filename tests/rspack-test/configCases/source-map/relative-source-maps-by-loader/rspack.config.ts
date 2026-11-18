import { defineConfig } from '@rspack/cli';
import { DefinePlugin } from '@rspack/core';

export default defineConfig({
  mode: 'development',
  node: {
    __dirname: false,
    __filename: false,
  },
  devtool: 'source-map',
  plugins: [
    new DefinePlugin({ TEST_CONTEXT: JSON.stringify(import.meta.dirname) }),
  ],
});
