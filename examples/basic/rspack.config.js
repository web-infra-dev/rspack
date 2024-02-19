const rspack = require('@rspack/core')
/**@type {import("@rspack/cli").Configuration}*/
module.exports = {
  entry: {
    main: "./src/index.js",
  },
  optimization: {
    concatenateModules: true,
    minimize: false,
  },
  experiments: {
    rspackFuture: {
      newTreeshaking: true,
    },
  },
	plugins: [new rspack.HtmlRspackPlugin({
	})]
};
