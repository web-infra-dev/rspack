/** @type {import('@rspack/core').Configuration} */
module.exports = {
	entry: {
		main: "./index.js"
	},
	mode: 'production',
	stats: {
		chunks: true,
		children: true,
		modules: true,
		optimizationBailout: true,
		reasons: true,
		ids: true,
		providedExports: true,
		usedExports: true
	}
};
