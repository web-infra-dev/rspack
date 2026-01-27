/** @type {function(any, any): import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	mode: "development",
	module: {
		rules: [
			{
				test: /\.my-css$/i,
				type: "css/auto"
			},
			{
				test: /\.invalid$/i,
				type: "css/auto"
			},
			{
				test: /\.css$/,
				type: "css/auto"
			}
		]
	},
	node: {
		__dirname: false,
		__filename: false
	}
};
