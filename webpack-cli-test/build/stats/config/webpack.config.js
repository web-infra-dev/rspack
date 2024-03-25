const WebpackCLITestPlugin = require("../../../utils/webpack-cli-test-plugin");

module.exports = {
  mode: "development",
  entry: "./index.js",
  stats: "normal",
  plugins: [new WebpackCLITestPlugin()],
};
