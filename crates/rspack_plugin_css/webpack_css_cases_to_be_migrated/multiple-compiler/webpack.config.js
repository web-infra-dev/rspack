import Self from "../../../src";

module.exports = [
  {
    entry: "./index.js",
    output: {
      filename: "one-[name].js",
    },
    module: {
      rules: [
        {
          test: /\.css$/,
          use: [Self.loader, "css-loader"],
        },
      ],
    },
    plugins: [
      new Self({
        filename: "one/[name].css",
      }),
    ],
  },
  {
    entry: "./index.js",
    output: {
      filename: "two-[name].js",
    },
    module: {
      rules: [
        {
          test: /\.css$/,
          use: [Self.loader, "css-loader"],
        },
      ],
    },
    plugins: [
      new Self({
        filename: "two/[name].css",
      }),
    ],
  },
];
