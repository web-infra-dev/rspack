const path = require("path");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: "./index.js",
  resolve: {
    alias: {
      "@foo/index": path.resolve(__dirname, "./b"),
      "@foo": path.resolve(__dirname, "./a"), // should not be used
    }
  },
};
