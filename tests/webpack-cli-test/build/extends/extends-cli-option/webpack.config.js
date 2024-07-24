const WebpackCLITestPlugin = require("../../../utils/webpack-cli-test-plugin");

module.exports = () => {
  console.log("derived.webpack.config.js");

  return {
    plugins: [new WebpackCLITestPlugin()],
  };
};
