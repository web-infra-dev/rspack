/**@type {import("@rspack/cli").Configuration}*/
module.exports = {
	experiments: {
		css: true,
		rspackFuture: {
			newTreeshaking: true
		}
	},
	optimization: {
		minimize: false,
		moduleIds: "named"
	},
	module: {
		rules: [
			{
				test: /\.module\.css$/,
				type: "css/module"
			}
		]
	},
	entry: {
		main: "./index.js"
	}
};
