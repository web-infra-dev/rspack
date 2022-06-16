import Self from "../../../src";

module.exports = {
  entry: "./index.js",
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [Self.loader, "css-loader"],
      },
    ],
  },
  output: {
    iife: false,
  },
  plugins: [
    new Self({
      filename: "[name].css",
    }),
  ],
};
