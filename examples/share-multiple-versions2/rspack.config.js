// eslint-disable-next-line node/no-unpublished-require
const { SharePlugin } = require("@rspack/core").sharing;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	experiments: {
		rspackFuture: {
			newTreeshaking: true
		}
	},
	devtool: false,
	entry: {
		main: "./index.js"
	},
	optimization: {
		// mangleExports: false,
		minimize: false,
		moduleIds: "named",
		chunkIds: "named",
		sideEffects: false
	},
	builtins: {
		html: [{}]
	},
	plugins: [
		new SharePlugin({
			shared: ["shared"]
		})
	]
};
