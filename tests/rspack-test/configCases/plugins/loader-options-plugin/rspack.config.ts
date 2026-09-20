import { defineConfig } from '@rspack/cli';
import { rspack as webpack } from '@rspack/core';

export default defineConfig({
  plugins: [
    new webpack.LoaderOptionsPlugin({
      minimize: true,
    }),
    new webpack.LoaderOptionsPlugin({
      test: /\.js$/,
      jsfile: true,
    }),
  ],
});
