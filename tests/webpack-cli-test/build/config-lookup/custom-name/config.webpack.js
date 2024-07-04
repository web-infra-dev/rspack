const { resolve } = require("path");

module.exports = {
  entry: resolve("./a.js"),
  output: {
    path: resolve(__dirname, "binary"),
    filename: "a.bundle.js",
  },
};
