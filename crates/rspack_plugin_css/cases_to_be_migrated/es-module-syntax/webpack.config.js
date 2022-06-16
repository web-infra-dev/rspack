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
            options: { esModule: true },
          },
          {
            loader: "css-loader",
            options: {
              modules: {
                mode: "local",
                localIdentName: "foo__[name]__[local]",
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
