/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	mode: "development",
	entry: {
		main: "./index.js"
	},
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
					namedExports: false,
				},
				generator: {
					exportsOnly: false,
					localIdentName: "[path][name][ext]__[local]"
				}
			}
		]
	},
	experiments: {
		css: true,
	}
};
