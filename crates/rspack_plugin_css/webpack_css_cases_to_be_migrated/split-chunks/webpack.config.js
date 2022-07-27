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
  optimization: {
    splitChunks: {
      chunks: "all",
      cacheGroups: {
        vendors: {
          name: "vendors",
          test: /node_modules/,
          enforce: true,
        },
      },
    },
  },
  plugins: [
    new Self({
      filename: "[name].css",
    }),
  ],
};
