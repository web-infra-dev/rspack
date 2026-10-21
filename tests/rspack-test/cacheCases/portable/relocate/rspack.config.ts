import { defineConfig } from '@rspack/cli';

export default defineConfig({
  devtool: 'source-map',
  module: {
    rules: [{ test: /plain(?:-two)?\.js$/, use: ['./count-loader.cjs'] }],
  },
  cache: { type: 'persistent', portable: { root: '.' } },
});
