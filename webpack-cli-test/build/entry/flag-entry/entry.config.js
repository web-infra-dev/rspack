const WebpackCLITestPlugin = require("../../../utils/webpack-cli-test-plugin");

module.exports = {
  entry: "./src/index.cjs",
  plugins: [new WebpackCLITestPlugin(["entry"])],
};
