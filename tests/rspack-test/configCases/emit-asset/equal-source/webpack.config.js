/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /\.txt$/,
				use: {
					loader: "file-loader",
					options: {
						name: "same-name.txt"
					}
				}
			}
		]
	}
};
