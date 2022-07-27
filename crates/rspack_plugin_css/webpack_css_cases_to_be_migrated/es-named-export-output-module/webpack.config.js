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
          {
            loader: "css-loader",
            options: {
              esModule: true,
              modules: {
                namedExport: true,
                localIdentName: "foo__[name]__[local]",
              },
            },
          },
        ],
      },
    ],
  },
  output: {
    module: true,
  },
  experiments: {
    outputModule: true,
  },
  plugins: [
    new Self({
      filename: "[name].css",
    }),
  ],
};
