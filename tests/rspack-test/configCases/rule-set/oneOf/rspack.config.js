/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	output: {
		assetModuleFilename: "[name][ext]"
	},
	experiments: {
		css: false
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				oneOf: [
					{
						use: ["./style-loader", "./css-loader"],
						issuer: /\.(js)$/
					},
					{
						// TODO: should not change source type when no pre/post loader
						// type: "asset/resource",
						issuer: /\.(css|scss|sass)$/
					}
				]
			}
		]
	}
};
