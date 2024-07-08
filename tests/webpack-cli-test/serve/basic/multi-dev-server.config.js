// eslint-disable-next-line node/no-unpublished-require
const getPort = require("get-port");

const WebpackCLITestPlugin = require("../../utils/webpack-cli-test-plugin");
const { devServerConfig } = require("./helper/base-dev-server.config");

module.exports = async () => [
  {
    name: "one",
    mode: "development",
    devtool: false,
    output: {
      filename: "first-output/[name].js",
    },
    devServer: {
      ...devServerConfig,
      port: await getPort(),
    },
    plugins: [new WebpackCLITestPlugin(["mode", "output"], false, "hooks.compilation.taps")],
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
      ...devServerConfig,
      port: await getPort(),
    },
  },
];
