const WebpackCLITestPlugin = require("../../../utils/webpack-cli-test-plugin");

module.exports = [
  {
    output: {
      filename: "./dist-amd.js",
      libraryTarget: "amd",
    },
    name: "amd",
    entry: "./index.js",
    mode: "development",
    devtool: "eval-cheap-module-source-map",
    plugins: [new WebpackCLITestPlugin()],
  },
  {
    output: {
      filename: "./dist-commonjs.js",
      libraryTarget: "commonjs",
    },
    name: "commonjs",
    entry: "./index.js",
    mode: "development",
    devtool: "source-map",
    target: "node",
    plugins: [new WebpackCLITestPlugin()],
  },
];
