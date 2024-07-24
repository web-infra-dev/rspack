const WebpackCLITestPlugin = require("../../../utils/webpack-cli-test-plugin");

module.exports = {
  mode: "development",
  entry: "./main.js",
  stats: "detailed",
  plugins: [new WebpackCLITestPlugin()],
};
