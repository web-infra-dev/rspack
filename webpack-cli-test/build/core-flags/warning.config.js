const path = require("path");

module.exports = {
  mode: "development",
  entry: "./src/main.js",
  module: {
    rules: [
      {
        test: /.(js|jsx)?$/,
        loader: "my-warning-loader",
        include: [path.resolve(__dirname, "src")],
        exclude: [/node_modules/],
      },
    ],
  },
  resolveLoader: {
    alias: {
      "my-warning-loader": require.resolve("./my-warning-loader"),
    },
  },
  performance: {
    hints: "warning",
  },
};
