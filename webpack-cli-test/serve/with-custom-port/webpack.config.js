const WebpackCLITestPlugin = require("../../utils/webpack-cli-test-plugin");

module.exports = {
  mode: "development",
  devtool: false,
  stats: "detailed",
  devServer: {
    port: 1234,
    host: "0.0.0.0",
  },
  plugins: [new WebpackCLITestPlugin(["mode"], false, "hooks.compilation.taps")],
};
