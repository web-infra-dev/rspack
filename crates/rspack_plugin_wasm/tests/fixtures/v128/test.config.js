/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
	entry: {
		main: "./index.js"
	},
	optimization: {
		minimize: false
	},
	experiments: {
		asyncWebAssembly: true
	}
};
