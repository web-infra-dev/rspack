import { defineConfig } from '@rspack/cli';

export default defineConfig({
  devtool: 'source-map',
  module: {
    rules: [
      {
        test: /\.less$/,
        type: 'asset/source',
        use: [
          './assert-map-loader.mjs',
          {
            loader: 'builtin:less-loader',
            options: { sourceMap: false },
          },
        ],
      },
    ],
  },
});
