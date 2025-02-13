const { createHtmlPlugin, createIntegrityPlugin, getDist } = require("../wsi-test-helper");
const path = require("path");
const ScriptExtHtmlWebpackPlugin = require("script-ext-html-webpack-plugin");

module.exports = () => ({
  entry: {
    app: path.resolve(__dirname, "index"),
  },
  output: {
    path: path.join(getDist(__dirname), "inline"),
    crossOriginLoading: "anonymous",
  },
  optimization: {
    runtimeChunk: {
      // Put webpack runtime code in a single separate chunk called "runtime.js"
      name: "runtime",
    },
  },
  plugins: [
    createHtmlPlugin({
      inject: "body",
    }),
    new ScriptExtHtmlWebpackPlugin({
      inline: {
        // Inline "runtime.js" as a <script> tag in the HTML
        chunks: "initial",
        test: "runtime",
      },
    }),
    createIntegrityPlugin({
    }),
  ],
});
