/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "node",
	mode: "development",
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "css/auto"
			}
		]
	},
	output: {
		module: true,
		chunkFormat: "module"
	}
};
