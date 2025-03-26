const path = require("node:path");

module.exports = {
	context: __dirname,
	entry: {
		main: './index.js'
	},
	experiments: {
		buildHttp: {
			allowedUris: [
				"https://github.com/"
			],
			// cacheLocation: path.join(__dirname, "rspack-http-cache"),
			// lockfileLocation: path.join(__dirname, "rspack-http-lockfile.json"),
		}
	}
}
