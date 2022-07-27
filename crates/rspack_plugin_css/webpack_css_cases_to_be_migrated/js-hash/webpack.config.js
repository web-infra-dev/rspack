import Self from "../../../src";

module.exports = [1, 2].map((n) => {
  return {
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
                modules: true,
              },
            },
            {
              loader: "./loader",
              ident: "my-loader",
              options: {
                number: n,
              },
            },
          ],
        },
      ],
    },
    output: {
      filename: `[name].[contenthash].${n}.js`,
    },
    plugins: [
      new Self({
        filename: `[name].[contenthash].${n}.css`,
      }),
    ],
  };
});
