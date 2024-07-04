// webpack.config.js
const { cli } = require("webpack");
const WebpackCLITestPlugin = require("../webpack-cli-test-plugin");

module.exports = {
  entry: "./main.js",
  mode: "development",
  target: "node",
  resolve: {
    alias:
      typeof cli !== "undefined"
        ? {
            alias: ["alias1", "alias2"],
          }
        : {},
  },
  plugins: [new WebpackCLITestPlugin(["resolve"])],
};
