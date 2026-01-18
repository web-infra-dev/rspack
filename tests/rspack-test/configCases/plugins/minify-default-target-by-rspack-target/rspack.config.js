/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: ["web", "browserslist:chrome > 95"],
	node: {
		__dirname: false,
		__filename: false,
	},
	module: {
		rules: [
			{
				test: /\.css/,
				type: "css/auto"
			}
		]
	},
	optimization: {
		minimize: true,
	},
};
