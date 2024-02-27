const path = require("path");

const config = {
  mode: "development",
  entry: "./main.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "foo.bundle.js",
  },
};

module.exports.default = config;
