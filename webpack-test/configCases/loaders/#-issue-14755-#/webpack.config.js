/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /\.my$/,
				loader: "regexp-#-loader"
			}
		]
	}
};
