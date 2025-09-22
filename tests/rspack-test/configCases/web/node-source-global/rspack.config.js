const { DefinePlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	node: {
		global: true
	},
	plugins: [
		new DefinePlugin({
			"global.test": "'test'"
		})
	]
};
