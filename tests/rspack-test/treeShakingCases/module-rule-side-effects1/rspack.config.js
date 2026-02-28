const { DefinePlugin } = require("@rspack/core");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		sideEffects: true
	},
	plugins: [
		new DefinePlugin({
			"process.env.NODE_ENV": "'development'"
		})
	],
	module: {
		rules: [
			{
				test: /b.js$/,
				sideEffects: false
			}
		]
	}
};
