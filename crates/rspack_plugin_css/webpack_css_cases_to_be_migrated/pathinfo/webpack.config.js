import Self from "../../../src";

module.exports = {
  entry: "./index.js",
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
