// eslint-disable-next-line node/no-unpublished-require
const MiniCssExtractPlugin = require("mini-css-extract-plugin");

module.exports = {
  mode: "development",
  entry: {
    home: ["./home.js", "./home.scss"],
  },
  output: {
    filename: "[name].js",
  },
  module: {
    rules: [
      {
        test: /\.scss$/,
        use: [
          // fallback to style-loader in development
          MiniCssExtractPlugin.loader,
          "css-loader",
          "sass-loader",
        ],
      },
    ],
  },
  plugins: [
    new MiniCssExtractPlugin({
      filename: "[name].css",
    }),
  ],
};
