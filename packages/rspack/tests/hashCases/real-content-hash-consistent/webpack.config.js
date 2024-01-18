const path = require("path");
const base = {
	mode: "production",
	entry: "./src/index.js",
	devtool: false,
	output: {
		filename: "main.js",
		assetModuleFilename: "[contenthash][ext]"
	},
	module: {
		rules: [
			{
				test: /\.(png|jpg)$/,
				type: "asset/resource"
			}
		]
	},
	stats: "normal",
	context: __dirname
};

module.exports = [
	{
		...base,
		output: {
			...base.output,
			path: path.resolve(__dirname, "./dist/enable")
		},
		optimization: {
			realContentHash: true
		}
	},
	{
		...base,
		output: {
			...base.output,
			path: path.resolve(__dirname, "./dist/disable")
		},
		optimization: {
			realContentHash: false
		}
	}
];
