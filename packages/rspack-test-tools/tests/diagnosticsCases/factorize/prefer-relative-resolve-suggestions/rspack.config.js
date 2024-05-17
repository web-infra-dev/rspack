/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /\.js$/,
				type: "javascript/esm"
			}
		]
	}
}
