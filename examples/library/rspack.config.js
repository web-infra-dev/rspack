const path = require('path')
const noNameLibraryTypes = ["amd-require", "module"];

/** @type {import('@rspack/cli').Configuration} */
const config = ["umd"].map((type) => ({
	context: __dirname,
	mode: "production",
	devtool: false,
	entry: {
		main: "./src/index.js",
	},
	optimization: {
		minimize: false,
	},
	output: {
		filename: `${type}.js`,
		library: {
			name: noNameLibraryTypes.includes(type) ? undefined : "myLib",
			type: type,
		},
	},
	builtins: {
		html: [{
			template: path.resolve(__dirname, "./index.html")
		}],
	},
	experiments: {
		rspackFuture: {
			newTreeshaking: true,
		},
	},
}));
module.exports = config;
