/**@type {import("@rspack/cli").Configuration}*/
module.exports = {
	entry: {
		main: "./index.js"
	},
	optimization: {
		concatenateModules: true,
		minimize: false
	},
	builtins: {
		css: {
			namedExports: true
		}
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "css/module"
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
