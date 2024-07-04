const WebpackCLITestPlugin = require("../../../utils/webpack-cli-test-plugin");

module.exports = {
  entry: "./first-entry.js",
  mode: "development",
  output: {
    filename: "first-output.js",
  },
  plugins: [new WebpackCLITestPlugin()],
};
