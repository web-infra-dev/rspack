const path = require("path");
const WebpackCLITestPlugin = require("../../../utils/webpack-cli-test-plugin");

module.exports = {
  output: {
    path: path.join(__dirname, "dist"),
    filename: "[name].js",
  },
  plugins: [new WebpackCLITestPlugin()],
};
