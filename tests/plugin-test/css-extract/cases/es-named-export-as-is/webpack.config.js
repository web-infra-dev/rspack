const Self = require("@rspack/core").CssExtractRspackPlugin;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: "./index.js",
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [
          {
            loader: Self.loader,
          },
          {
            loader: "css-loader",
            options: {
              esModule: true,
              modules: {
                namedExport: true,
                exportLocalsConvention: "asIs",
              },
            },
          },
        ],
      },
    ],
  },
  plugins: [
    new Self({
      filename: "[name].css",
    }),
  ],
};
