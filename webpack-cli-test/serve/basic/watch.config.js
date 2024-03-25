const WebpackCLITestPlugin = require("../../utils/webpack-cli-test-plugin");

module.exports = {
  mode: "development",
  devtool: false,
  plugins: [new WebpackCLITestPlugin(["mode"], false, "hooks.compilation.taps")],
  watch: true,
  devServer: {
    client: {
      logging: "info",
    },
  },
};
