const path = require("path");

module.exports = {
  mode: "development",

  entry: {
    bundle: "./src/index.ts",
  },

  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "[name].js",
  },

  module: {
    rules: [
      {
        test: /.(ts|tsx)?$/,
        loader: "ts-loader",
        include: [path.resolve(__dirname, "src")],
        exclude: [/node_modules/],
      },
    ],
  },
};
