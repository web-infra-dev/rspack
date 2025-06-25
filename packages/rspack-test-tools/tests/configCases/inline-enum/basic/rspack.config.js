/**
 * @return {import("@rspack/core").Configuration}
 */
function config(index, { concatenateModules } = {}) {
	return {
		entry: "./index.js",
		output: {
			filename: `bundle.${index}.js`
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
									}
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
		plugins: [
			function (compiler) {
				new compiler.webpack.DefinePlugin({
					CONCATENATED: JSON.stringify(concatenateModules)
				}).apply(compiler);
			}
		],
		optimization: {
			concatenateModules,
			moduleIds: "named"
		}
	};
}

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	config(0, { concatenateModules: true }),
	config(1, { concatenateModules: false })
];
