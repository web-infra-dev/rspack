module.exports = {
	context: __dirname,
	mode: "development",
	devtool: false,
	entry: {
		main: './index.js'
	},
	externalsPresets: {
		web: false,
		webAsync: false
	},
	experiments: {
		buildHttp: {
			allowedUris: ["https://esm.sh/.*"]
		}
	}
}
