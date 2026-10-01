import { defineConfig } from '@rspack/cli';

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /\.png$/,
        use: [{ loader: './loader.mjs', parallel: true, options: {} }],
        type: 'asset/resource',
      },
    ],
  },
});
