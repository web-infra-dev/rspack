/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	optimization: {
		concatenateModules: true,
		minimize: false
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "css/module",
				parser: {
					namedExports: false
				},
				generator: {
					exportsOnly: true,
					localIdentName: "[local]"
				}
			}
		]
	}
};
