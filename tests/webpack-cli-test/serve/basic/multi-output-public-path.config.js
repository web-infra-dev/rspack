const WebpackCLITestPlugin = require("../../utils/webpack-cli-test-plugin");

module.exports = [
  {
    name: "one",
    mode: "development",
    devtool: false,
    output: {
      publicPath: "/my-public-path/",
      filename: "first-output/[name].js",
    },
    plugins: [new WebpackCLITestPlugin(["mode", "output"], false, "hooks.compilation.taps")],
    devServer: {
      client: {
        logging: "info",
      },
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
  },
];
