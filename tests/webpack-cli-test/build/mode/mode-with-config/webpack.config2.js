const WebpackCLITestPlugin = require("../../../utils/webpack-cli-test-plugin");

module.exports = {
  mode: "development",
  plugins: [new WebpackCLITestPlugin()],
};
