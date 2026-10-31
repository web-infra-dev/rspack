import { defineConfig } from '@rspack/cli';

export default defineConfig({
  entry: './index.js',
  mode: 'development',
  devtool: false,
  cache: { type: 'persistent' },
  experiments: {
    newCache: { codeGeneration: false, loader: true, minimize: false },
  },
  module: {
    rules: [
      {
        test: /entry\.js$/,
        use: [
          {
            loader: './transform-loader.mjs',
            options: { value: 'filtered' },
            codeFilter: 'marker',
            cache: true,
          },
        ],
      },
    ],
  },
});
