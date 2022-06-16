/* global document */

import Self from "../../../src";

module.exports = {
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [
          {
            loader: Self.loader,
          },
          {
            loader: "css-loader",
          },
        ],
      },
    ],
  },
  plugins: [
    new Self({
      filename: "[name].css",
      chunkFilename: "[id].css",
      insert: "script[src='1.js']",
    }),
  ],
};
