const WebpackCLITestPlugin = require("../../utils/webpack-cli-test-plugin");

module.exports = {
  mode: process.env.NODE_ENV,
  plugins: [new WebpackCLITestPlugin()],
};
