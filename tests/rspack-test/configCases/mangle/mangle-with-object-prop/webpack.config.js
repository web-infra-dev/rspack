const { DefinePlugin } = require("@rspack/core");
/** @type {import("@rspack/core").Configuration} */
module.exports = [
	{
		output: {
			filename: "deterministic.js"
		},
		optimization: {
			mangleExports: true,
			usedExports: true,
			providedExports: true
		},
		plugins: [
			new DefinePlugin({
				OPTIMIZATION: JSON.stringify("deterministic")
			})
		]
	},
	{
		output: {
			filename: "size.js"
		},
		optimization: {
			mangleExports: "size",
			usedExports: true,
			providedExports: true
		},
		plugins: [
			new DefinePlugin({
				OPTIMIZATION: JSON.stringify("size")
			})
		]
	}
];
