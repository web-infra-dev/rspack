const { ProgressPlugin } = require("webpack");
const WebpackCLITestPlugin = require("../../utils/webpack-cli-test-plugin");

module.exports = {
  plugins: [new ProgressPlugin({}), new WebpackCLITestPlugin()],
};
