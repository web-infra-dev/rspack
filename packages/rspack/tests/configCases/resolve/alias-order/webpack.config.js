const path = require("path");
module.exports = {
  entry: "./index.js",
  resolve: {
    alias: {
      "@foo/index": path.resolve(__dirname, "./b"),
      "@foo": path.resolve(__dirname, "./a"), // should not be used
    }
  },
};
