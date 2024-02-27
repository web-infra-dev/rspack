const WebpackCLITestPlugin = require("../../../utils/webpack-cli-test-plugin");

module.exports = () => {
  console.log("derived.webpack.config.js");

  return [
    {
      extends: "./base.webpack.config.js",
    },
    {
      name: "derived_config2",
      mode: "development",
      entry: "./src/index2.js",
      plugins: [new WebpackCLITestPlugin()],
    },
  ];
};
