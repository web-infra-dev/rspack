import Self from "../../../src";

module.exports = {
  entry: "./index.js",
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [
          Self.loader,
          {
            loader: "css-loader",
            // TODO Uncomment after `css-loader` release the `esModule` option
            // options: { esModule: false },
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
