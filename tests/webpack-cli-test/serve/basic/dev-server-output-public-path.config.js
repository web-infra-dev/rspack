const WebpackCLITestPlugin = require("../../utils/webpack-cli-test-plugin");
const { devServerConfig } = require("./helper/base-dev-server.config");

module.exports = {
  mode: "development",
  devtool: false,
  output: {
    publicPath: "/my-public-path/",
  },
  devServer: devServerConfig,
  plugins: [new WebpackCLITestPlugin(["mode", "output"], false, "hooks.compilation.taps")],
};
