const path = require("path");

/**
 * @type {import('webpack').Configuration | import('@rspack/cli').Configuration}
 */
module.exports = {
	devtool: false,
	entry: {
		main: {
			import: "./index.js"
		}
	},
	resolve: {
		mainFields: ["module", "main"],
		extensionAlias: {
			".js": [".ts", ".js"]
		}
	}
};
