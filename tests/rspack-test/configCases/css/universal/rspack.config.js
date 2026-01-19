/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: ["web", "node"],
	mode: "development",
	experiments: {
		outputModule: true
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
