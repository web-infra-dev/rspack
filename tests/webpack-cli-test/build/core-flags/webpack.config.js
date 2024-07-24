const WebpackCLITestPlugin = require("../../utils/webpack-cli-test-plugin");

module.exports = {
  entry: "./src/main.js",
  mode: "development",
  name: "compiler",
  plugins: [new WebpackCLITestPlugin(["module", "entry", "resolve", "resolveLoader", "cache"])],
};
