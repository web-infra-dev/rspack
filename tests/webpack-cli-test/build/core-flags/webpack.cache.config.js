const WebpackCLITestPlugin = require("../../utils/webpack-cli-test-plugin");

module.exports = {
  entry: "./src/main.js",
  mode: "development",
  cache: {
    type: "filesystem",
    name: "config-cache",
  },
  name: "compiler-cache",
  plugins: [new WebpackCLITestPlugin(["cache"])],
};
