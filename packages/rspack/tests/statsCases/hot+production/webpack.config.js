const rspack = require("@rspack/core")

module.exports = {
	entry: {
		main: "./index.js"
	},
	plugins: [
		new rspack.HotModuleReplacementPlugin(),
	],
	mode: "production"
};
