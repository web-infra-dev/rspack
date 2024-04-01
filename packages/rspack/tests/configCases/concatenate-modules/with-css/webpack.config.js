/**@type {import("@rspack/cli").Configuration}*/
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
				}
			}
		]
	},
	experiments: {
		css: true,
		rspackFuture: {
			newTreeshaking: true
		}
	}
};
