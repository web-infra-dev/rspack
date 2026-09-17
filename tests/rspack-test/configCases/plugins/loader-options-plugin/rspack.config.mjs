import { rspack as webpack } from '@rspack/core';

/** @type {import("@rspack/core").Configuration} */
export default {
  plugins: [
    new webpack.LoaderOptionsPlugin({
      minimize: true,
    }),
    new webpack.LoaderOptionsPlugin({
      test: /\.js$/,
      jsfile: true,
    }),
  ],
};
