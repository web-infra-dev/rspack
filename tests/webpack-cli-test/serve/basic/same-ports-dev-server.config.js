module.exports = [
  {
    name: "one",
    mode: "development",
    devtool: false,
    output: {
      filename: "first-output/[name].js",
    },
    devServer: {
      port: 8081,
    },
  },
  {
    name: "two",
    mode: "development",
    devtool: false,
    entry: "./src/other.js",
    output: {
      filename: "second-output/[name].js",
    },
    devServer: {
      port: 8081,
    },
  },
];
