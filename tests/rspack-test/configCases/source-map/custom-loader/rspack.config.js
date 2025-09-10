/** @type {import("@rspack/core").Configuration} */
module.exports = {
	devtool: false,
	plugins: [
		compiler => {
			new compiler.webpack.SourceMapDevToolPlugin({}).apply(compiler);
		}
	],
	module: {
		rules: [
			{
				loader: "./loader.js"
			}
		]
	}
};
