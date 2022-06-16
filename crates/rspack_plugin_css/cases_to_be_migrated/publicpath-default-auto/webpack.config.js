import Self from "../../../src";

module.exports = {
  entry: "./index.js",
  output: {
    publicPath: "auto",
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
