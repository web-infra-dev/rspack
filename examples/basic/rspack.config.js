const rspack = require("@rspack/core");
/** @type {import('@rspack/cli').Configuration} */
const config = {
	entry: "./src/index.js",
	experiments: {
		rspackFuture: {
			newTreeshaking: true
		},
	},
	plugins: [
		new rspack.HtmlRspackPlugin({
			template: "./index.html"
		})
	],
	builtins: {
		treeShaking: false
	},
	optimization: {
		providedExports: true,
		sideEffects: true,
		innerGraph: true,
		usedExports: true,
		moduleIds: "named",
		minimize: false,
	}
};
module.exports = config;
