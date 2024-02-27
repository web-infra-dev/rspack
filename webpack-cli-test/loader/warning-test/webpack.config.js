const path = require("path");

module.exports = {
  mode: "development",

  entry: {
    bundle: "./src/main.js",
  },

  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "[name].js",
  },

  module: {
    rules: [
      {
        test: /.(js|jsx)?$/,
        loader: "my-loader",
        include: [path.resolve(__dirname, "src")],
        exclude: [/node_modules/],
      },
    ],
  },
  resolveLoader: {
    alias: {
      "my-loader": require.resolve("./my-loader"),
    },
  },
  performance: {
    hints: "warning",
  },
};
