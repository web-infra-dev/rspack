var webpack = require("../../../../");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [new webpack.WatchIgnorePlugin({ paths: [/file\.js$/, /foo$/] })]
};
