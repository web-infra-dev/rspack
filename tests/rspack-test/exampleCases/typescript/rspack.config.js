const { TsCheckerRspackPlugin } = require('ts-checker-rspack-plugin');

module.exports = {
	mode: "development",
	entry: {
		output: "./index.ts"
	},
	module: {
		rules: [
			{
				test: /\.tsx?$/,
				loader: "ts-loader",
				options: {
					transpileOnly: true
				}
			}
		]
	},
	resolve: {
		extensions: [".ts", ".js", ".json"]
	},
	plugins: [new TsCheckerRspackPlugin({ async: false })]
};
