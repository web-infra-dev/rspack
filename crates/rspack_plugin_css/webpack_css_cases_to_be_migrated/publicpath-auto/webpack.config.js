import Self from "../../../src";

module.exports = {
  entry: "./index.js",
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [
          {
            loader: Self.loader,
            options: {
              publicPath: "auto",
            },
          },
          "css-loader",
        ],
      },
      {
        test: /\.(svg|png)$/,
        type: "asset/resource",
        generator: { filename: "assets/[name][ext]" },
      },
    ],
  },
  plugins: [
    new Self({
      filename: "styles/[contenthash]/[name].css",
    }),
  ],
};
