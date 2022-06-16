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
                mode: "local",
                localIdentName: "foo__[name]__[local]",
                exportOnlyLocals: true,
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
