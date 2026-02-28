const path = require("path");

const config = {
	target: "web",
	node: false,
	module: {
		generator: {
			"css/auto": {
				exportsOnly: false
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
			},
			{
				test: /\.css$/,
				type: "css/auto"
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

};

module.exports = config;
