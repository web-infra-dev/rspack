/**
 * @return {import("@rspack/core").Configuration}
 */
function config(index, { concatenateModules } = {}) {
	return {
		entry: "./index.js",
		output: {
			filename: `bundle.${index}.js`
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
		},
		experiments: {
			inlineConst: true
		}
	};
}

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	config(0, { concatenateModules: true }),
	config(1, { concatenateModules: false })
];
