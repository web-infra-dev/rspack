import Self from "../../../src";

module.exports = {
  entry: {
    entry1: "./entry1.js",
    entry2: "./entry2.js",
  },
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
      cacheGroups: {
        styles: {
          name: "styles",
          chunks: "all",
          test: /\.css$/,
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
