const path = require("path");

module.exports = [
  {
    mode: "development",
    name: "cache-test-first",
    cache: {
      type: "filesystem",
      buildDependencies: {
        config: [__filename],
      },
    },
    infrastructureLogging: {
      debug: /cache/,
    },
    entry: {
      app: "./src/main.js",
    },
    output: {
      filename: "[name].bundle.js",
      chunkFilename: "[name].bundle.js",
      path: path.resolve(__dirname, "dist"),
      publicPath: "/",
    },
  },
  {
    mode: "development",
    name: "cache-test-second",
    cache: {
      type: "filesystem",
      buildDependencies: {
        config: [__filename],
      },
    },
    infrastructureLogging: {
      debug: /cache/,
    },
    entry: {
      app: "./src/main.js",
    },
    output: {
      filename: "[name].bundle.js",
      chunkFilename: "[name].bundle.js",
      path: path.resolve(__dirname, "dist"),
      publicPath: "/",
    },
  },
];
