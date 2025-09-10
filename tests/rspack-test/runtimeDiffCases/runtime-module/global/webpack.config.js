/** @type {import("webpack").Configuration} */
module.exports = {
	output: {
		publicPath: "auto",
		cssFilename: "css/[name].css"
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				use: ["css-loader"]
			}
		]
	}
};
