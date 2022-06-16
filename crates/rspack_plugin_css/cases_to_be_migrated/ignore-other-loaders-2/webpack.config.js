import Self from "../../../src";

module.exports = {
  entry: "./index.js",
  module: {
    rules: [
      {
        oneOf: [
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
            exclude: /\.(js|mjs|jsx|ts|tsx)$/,
            type: "asset/resource",
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
