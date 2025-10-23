/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		a: "./a",
		b: "./b",
	},
	output: {
		filename: "[name].js",
	},
	resolve: {
		extensions: [".ts", "..."]
	},
	module: {
		rules: [
			{
				test: /\.ts$/,
				use: [
					{
						loader: "builtin:swc-loader",
						options: {
							jsc: {
								parser: {
									syntax: "typescript"
								},
								target: "esnext"
							},
							rspackExperiments: {
								collectTypeScriptInfo: {
									exportedEnum: true
								}
							}
						}
					}
				]
			}
		]
	},
	optimization: {
		concatenateModules: false,
		splitChunks: {
			cacheGroups: {
				lib: {
					test: /lib/,
					name: 'lib',
					priority: 100,
					chunks: 'all',
					enforce: true,
					minSize: 0,
				},
			}
		}
	},
	experiments: {
		inlineEnum: true
	}
};
