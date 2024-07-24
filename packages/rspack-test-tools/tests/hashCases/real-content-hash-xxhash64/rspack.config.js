/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	entry: "./src/index.js",
	devtool: false,
	output: {
		filename: "main.js",
		hashFunction: "xxhash64",
		assetModuleFilename: "[contenthash][ext]"
	},
	module: {
		rules: [
			{
				test: /\.(png|jpg|svg)$/,
				type: "asset/resource"
			}
		]
	},
	context: __dirname,
	optimization: {
		realContentHash: true
	}
};
