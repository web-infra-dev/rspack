/** @type {import("@rspack/core").Configuration} */
module.exports = {
	parallelism: 1,
	mode: "development",
	module: {
		rules: [
			{
				test: /\.css$/i,
				type: "javascript/auto",
				use: [require.resolve("./loader"), "css-loader"]
			}
		]
	}
};
