module.exports = {
	context: __dirname,
	mode: 'development',
	devtool: false,
	entry: {
		main: './index.js'
	},
	externals: [],  // Don't treat any modules as external
	externalsPresets: {
		web: false,  // Disable web preset
		webAsync: false, // Disable webAsync preset
		node: false  // Disable node preset
	},
	experiments: {
		buildHttp: {
			allowedUris: [
				"https://esm.sh"
			]
		}
	}
}
