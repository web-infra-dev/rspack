const rspack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  cache: true,
  mode: "development",
  entry: "./index",
  module: {
    rules: [
      {
        test: /\.module.css$/,
        use: [
          {
            loader: rspack.CssExtractRspackPlugin.loader,
						options: {
							emit: false,
							esModule: true,
						}
          },
          "css-loader",
					"./loader.js"
        ]
      },
    ]
  },
  experiments: {
    css: false,
  },
  plugins: [
    new rspack.CssExtractRspackPlugin({
      filename: "[name].css",
			runtime: false,
    }),
  ]
};
