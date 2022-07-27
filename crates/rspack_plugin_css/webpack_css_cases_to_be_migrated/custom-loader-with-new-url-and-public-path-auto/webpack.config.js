import path from "path";

import Self from "../../../src";

module.exports = {
  entry: "./index.js",
  context: path.resolve(__dirname, "app"),
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [
          {
            loader: Self.loader,
            options: {
              publicPath: "auto",
            },
          },
          "./mockLoader",
        ],
      },
      {
        test: /\.png$/,
        type: "asset/resource",
        generator: {
          filename: "[path][name][ext]",
        },
      },
    ],
  },
  plugins: [
    new Self({
      filename: "[name].css",
    }),
  ],
};
