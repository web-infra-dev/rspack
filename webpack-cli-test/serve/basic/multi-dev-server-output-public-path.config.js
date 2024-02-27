const WebpackCLITestPlugin = require("../../utils/webpack-cli-test-plugin");
const { devServerConfig } = require("./helper/base-dev-server.config");

module.exports = [
  {
    name: "one",
    mode: "development",
    devtool: false,
    entry: "./src/other.js",
    output: {
      filename: "first-output/[name].js",
    },
    devServer: {
      client: {
        logging: "info",
      },
    },
    plugins: [new WebpackCLITestPlugin(["mode", "output"], false, "hooks.compilation.taps")],
  },
  {
    name: "two",
    mode: "development",
    devtool: false,
    stats: "detailed",
    output: {
      publicPath: "/my-public-path/",
      filename: "second-output/[name].js",
    },
    devServer: devServerConfig,
    plugins: [new WebpackCLITestPlugin(["mode", "output"], false, "hooks.compilation.taps")],
  },
];
