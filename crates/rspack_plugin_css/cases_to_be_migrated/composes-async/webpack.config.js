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
            options: {
              modules: {
                localIdentName: "[local]",
              },
            },
          },
        ],
      },
    ],
  },
  optimization: {
    splitChunks: {
      cacheGroups: {
        cssDedupe: {
          test: /\.css$/,
          name: "dedupe",
          chunks: "all",
          minChunks: 2,
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
