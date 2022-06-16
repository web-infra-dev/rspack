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
          },
          "css-loader",
        ],
      },
      {
        test: /\.svg$/,
        type: "asset/resource",
        generator: {
          filename: "static/[name][ext][query]",
        },
      },
    ],
  },
  plugins: [
    new Self({
      filename: "[name].css",
    }),
  ],
};
