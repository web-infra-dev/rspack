import { defineConfig } from '@rspack/cli';

export default defineConfig({
  target: 'node',
  devtool: 'source-map',
  module: {
    rules: [{ extractSourceMap: true }],
  },
});
