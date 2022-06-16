import Self from "../../../src";

module.exports = {
  entry: "./index.js",
  // Required to disable source maps in webpack@4
  devtool: false,
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
            options: {
              sourceMap: false,
            },
          },
        ],
      },
    ],
  },
  plugins: [
    new Self({
      filename: "[name].css",
    }),
  ],
};
