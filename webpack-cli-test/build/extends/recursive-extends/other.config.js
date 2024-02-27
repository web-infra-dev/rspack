const WebpackCLITestPlugin = require("../../../utils/webpack-cli-test-plugin");

module.exports = () => {
  return {
    extends: "./webpack.config.js",
    plugins: [new WebpackCLITestPlugin()],
  };
};
