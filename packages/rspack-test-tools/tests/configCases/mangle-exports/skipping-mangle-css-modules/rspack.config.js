/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	experiments: {
		css: true,
	},
	optimization: {
		minimize: false,
		moduleIds: "named"
	},
	module: {
		rules: [
			{
				test: /\.module\.css$/,
				type: "css/module",
				generator: {
					exportsOnly: false,
				},
				parser: {
					namedExports: false,
				}
			}
		]
	},
	entry: {
		main: "./index.js"
	}
};
