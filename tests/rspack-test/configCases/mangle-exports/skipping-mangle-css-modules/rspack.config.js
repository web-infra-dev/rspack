/**@type {import("@rspack/core").Configuration}*/
module.exports = {
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
					exportsOnly: true
				},
				parser: {
					namedExports: false
				}
			}
		]
	},
	entry: {
		main: "./index.js"
	}
};
