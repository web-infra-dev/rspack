/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
	entry: {
		main: './example.js'
	},
	output: {
		webassemblyModuleFilename: "[hash].wasm",
		publicPath: 'dist/'
	},
	experiments: {
		asyncWebAssembly: true
	},
	builtins: {
		html: [
			{
				template: './index.html'
			}
		]
	}
};
