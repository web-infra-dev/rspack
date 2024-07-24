const WebpackCLITestPlugin = require("../../../utils/webpack-cli-test-plugin");

module.exports = () => {
  console.log("derived.webpack.config.js");

  return {
    extends: ["./base1.webpack.config.js"],
    plugins: [new WebpackCLITestPlugin()],
  };
};
