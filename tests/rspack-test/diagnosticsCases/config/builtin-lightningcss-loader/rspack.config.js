/** @type {import('@rspack/core').Configuration} */
module.exports = {
	context: __dirname,
	entry: {
		main: "./index.js"
	},
	experiments: {
		css: true
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [
					{
						loader: "builtin:lightningcss-loader",
						/** @type {import("@rspack/core").LightningcssLoaderOptions} */
						options: {
							unusedSymbols: ["unused"],
							targets: "> 0.2%",
							drafts: "xx"
						}
					}
				],
				type: "css/auto"
			}
		]
	}
};
