const { DefinePlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		concatenateModules: true,
		sideEffects: true,
		providedExports: true,
		usedExports: "global"
	},
	plugins: [
		new DefinePlugin({
			"process.env.NODE_ENV": "'development'"
		})
	],
};
