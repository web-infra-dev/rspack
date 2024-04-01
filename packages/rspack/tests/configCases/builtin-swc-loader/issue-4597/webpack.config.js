const path = require("path");

const config = {
	module: {
		generator: {
			"css/auto": {
				exportsOnly: false
			},
			"css": {
				exportsOnly: false,
			},
			"css/module": {
				exportsOnly: false,
			}
		},
		rules: [
			{
				test: /\.(js|mjs|cjs|jsx)$/,
				loader: path.join(__dirname, "diy.js")
			},
			{
				test: /\.(js|mjs|cjs|jsx)$/,
				use: [
					{
						loader: "builtin:swc-loader",
						options: {
							rspackExperiments: {
								import: [
									{
										libraryName: "aaaaa",
										libraryDirectory: "es",
										style: "css"
									}
								]
							}
						}
					}
				]
			}
		]
	},
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.make.tap("child", base => {
					const child = base.createChildCompiler("child", {}, []);
					child.runAsChild(() => {});
				});
			}
		}
	],
	experiments: {
		css: true
	}
};

module.exports = config;
