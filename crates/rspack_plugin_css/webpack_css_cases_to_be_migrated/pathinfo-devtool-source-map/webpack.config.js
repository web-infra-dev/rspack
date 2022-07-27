import Self from "../../../src";

module.exports = {
  entry: "./index.js",
  devtool: "source-map",
  output: {
    pathinfo: true,
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [Self.loader, "css-loader"],
      },
    ],
  },
  plugins: [
    new Self({
      filename: "[name].css",
    }),
  ],
};
