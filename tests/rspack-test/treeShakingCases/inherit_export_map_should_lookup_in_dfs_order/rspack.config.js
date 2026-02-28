const { DefinePlugin } = require("@rspack/core");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		sideEffects: false
	},
	module: {
		parser: {
			javascript: {
				exportsPresence: 'auto',
			}
		}
	},
	plugins: [
		new DefinePlugin({
			"process.env.NODE_ENV": "'development'"
		})
	],
};
