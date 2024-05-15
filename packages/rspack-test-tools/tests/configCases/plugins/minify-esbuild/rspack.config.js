const minifyPlugin = require("@rspack/plugin-minify");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	target: "node",
	entry: {
		main: "./index.js"
	},
	optimization: {
		minimize: true,
		minimizer: [new minifyPlugin()]
	}
};
