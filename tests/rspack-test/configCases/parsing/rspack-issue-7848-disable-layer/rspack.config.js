const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		bundle0: "./index.js"
	},
	experiments: {
		layers: false
	},
	plugins: [
		new rspack.DefinePlugin({
			__RUNTIME_TYPE__: "__webpack_layer__"
		})
	]
};
