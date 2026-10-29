import { defineConfig } from '@rspack/cli';

export default defineConfig({
  devtool: false,
  module: {
    rules: [
      {
        test: /\.less$/,
        type: 'asset/source',
        use: [
          './assert-map-loader.mjs',
          {
            loader: 'builtin:less-loader',
            options: { sourceMap: true, additionalData: '@color: red;' },
          },
          './input-map-loader.mjs',
        ],
      },
    ],
  },
});
