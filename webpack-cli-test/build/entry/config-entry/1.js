const { resolve } = require("path");

module.exports = {
  entry: {
    index: "../a.js",
  },
  output: {
    path: resolve(process.cwd(), "binary"),
    filename: "[name].bundle.js",
  },
};
