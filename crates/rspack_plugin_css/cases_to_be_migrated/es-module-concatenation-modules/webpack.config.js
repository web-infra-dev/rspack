import Self from "../../../src";

module.exports = {
  entry: "./index.js",
  optimization: {
    concatenateModules: true,
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [
          {
            loader: Self.loader,
            options: {
              esModule: true,
            },
          },
          {
            loader: "css-loader",
            options: {
              esModule: true,
              modules: {
                namedExport: true,
                localIdentName: "foo__[local]",
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
