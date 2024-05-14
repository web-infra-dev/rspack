/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /\.js$/,
				resolve: {
					fullySpecified: true
				},
				type: "javascript/esm"
			}
		]
	}
}
