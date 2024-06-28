/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /\.generate-json\.js$/,
				use: "./loader",
				type: "json"
			}
		]
	}
};
