/** @type {import('@rspack/core').Configuration} */
module.exports = {
	entry: {
		main: "./index.js"
	},
	mode: 'production',
	experiments: {
		rspackFuture: {
			newTreeshaking: true
		}
	},
	stats: "verbose"
};
