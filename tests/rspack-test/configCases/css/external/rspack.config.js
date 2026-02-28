/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "css/auto"
			}
		]
	}
};
