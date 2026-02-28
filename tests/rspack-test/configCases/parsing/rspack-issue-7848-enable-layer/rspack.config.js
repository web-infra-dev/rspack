const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		bundle0: {
			import: "./index.js",
			layer: "main"
		}
	},
	plugins: [
		new rspack.DefinePlugin({
			__RUNTIME_TYPE__: "__webpack_layer__"
		})
	]
};
