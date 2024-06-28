const WebpackCLITestPlugin = require("../../../utils/webpack-cli-test-plugin");

module.exports = {
  output: {
    filename: "./dist-amd.js",
    libraryTarget: "amd",
  },
  name: "amd",
  entry: "./index.js",
  mode: "development",
  devtool: "source-map",
  plugins: [new WebpackCLITestPlugin()],
};
