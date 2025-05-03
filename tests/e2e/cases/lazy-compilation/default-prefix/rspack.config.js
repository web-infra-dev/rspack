const { rspack } = require("@rspack/core");

/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
  context: __dirname,
  entry: {
    main: "./src/index.js"
  },
  stats: "none",
  mode: "development",
  plugins: [new rspack.HtmlRspackPlugin()],
  experiments: {
    lazyCompilation: {
      entries: true,
      imports: true
      // Using default prefix (not specifying prefix option)
    }
  },
  devServer: {
    hot: true
  }
};
