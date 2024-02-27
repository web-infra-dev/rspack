const { resolve } = require("path");

module.exports = {
  entry: "./a.js",
  mode: "development",
  output: {
    path: resolve(__dirname, "bin"),
    filename: "a.bundle.js",
  },
};
