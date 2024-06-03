const rspack = require("@rspack/core");
const path = require("path");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  // correct if "false"
  cache: true,
  mode: "development",
  entry: "./index",
  devServer: {
    devMiddleware: {
      writeToDisk: true
    }
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [
          {
            loader: rspack.CssExtractRspackPlugin.loader
          },
          "css-loader",
          {
            loader: path.resolve(
              __dirname,
              "fake-content.js"
            )
          }
        ]
      },
    ]
  },
  experiments: {
    css: false
  },
  plugins: [
    new rspack.CssExtractRspackPlugin({
      filename: "[name].css",
			runtime: false
    }),
    new rspack.HotModuleReplacementPlugin()
  ],
  optimization: {
    minimize: false,
  },
};
