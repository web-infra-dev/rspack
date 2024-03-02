const WebpackCLITestPlugin = require("../../../utils/webpack-cli-test-plugin");

module.exports = () => {
  console.log("base.webpack.config.js");

  return {
    name: "base_config",
    mode: "development",
    plugins: [new WebpackCLITestPlugin()],

    experiments: {
      topLevelAwait: true,
    },
  };
};
