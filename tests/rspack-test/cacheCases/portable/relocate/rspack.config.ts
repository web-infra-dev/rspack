import { defineConfig } from '@rspack/cli';

export default defineConfig({
  devtool: 'source-map',
  module: {
    rules: [
      {
        test: /(?:plain(?:-two)?\.js|url\.mjs)$/,
        use: ['./count-loader.cjs'],
      },
    ],
  },
  cache: { type: 'persistent', portable: { root: '.' } },
});
