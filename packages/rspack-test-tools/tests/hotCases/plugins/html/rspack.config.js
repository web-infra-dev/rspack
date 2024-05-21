const rspack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  entry: "./index.js",
  plugins: [
    new rspack.HtmlRspackPlugin({
      template: "./index.html"
    }),
  ],
  node: {
    __dirname: false,
    __filename: false
  }
};
