const rspack = require("@rspack/core");

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
