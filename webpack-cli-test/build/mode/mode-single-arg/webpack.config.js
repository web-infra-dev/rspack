const WebpackCLITestPlugin = require("../../../utils/webpack-cli-test-plugin");

module.exports = {
  entry: "./src/index.js",
  plugins: [new WebpackCLITestPlugin()],
};
