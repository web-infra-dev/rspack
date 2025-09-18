const path = require("path");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: path.join(__dirname, "项目"),
  entry: "./src/index.js",
  mode: "development"
};
