/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "node",
	optimization: {
		concatenateModules: true
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "css/auto"
			}
		]
	}
};
