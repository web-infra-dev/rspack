const { DefinePlugin } = require("@rspack/core");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		sideEffects: true
	},
	module: {
		rules: [
			{
				test: /\.svg$/,
				type: "asset/inline"
			}
		]
	},
	plugins: [
		new DefinePlugin({
			"process.env.NODE_ENV": "'development'"
		})
	],
};
