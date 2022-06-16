import Self from "../../../src";

module.exports = {
  entry: "./index.css",
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
